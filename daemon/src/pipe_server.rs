/// Named Pipe Server
/// ==================
/// Daemon-side named pipe server that receives packet metadata from the WFP kernel driver.
/// For each packet, the WFP driver sends metadata, daemon consults token bucket,
/// and sends back a decision (permit/drop).

use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::token_bucket::TokenBucket;
use proto::{NETSHAPER_PIPE_NAME, PacketMetadata, PacketDecision};
use tracing::{info, debug, warn};

pub async fn run_pipe_server(
    token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
) {
    info!("Starting named pipe server at {}", NETSHAPER_PIPE_NAME);

    loop {
        // Create a new named pipe server instance
        match create_pipe_server().await {
            Ok(pipe) => {
                let buckets = token_buckets.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_pipe_client(pipe, buckets).await {
                        warn!("Pipe client error: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to create pipe server: {}. Retrying in 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn create_pipe_server() -> Result<tokio::net::windows::named_pipe::NamedPipeServer, String> {
    use tokio::net::windows::named_pipe::ServerOptions;

    let pipe = ServerOptions::new()
        .first_pipe_instance(false)
        .max_instances(10)
        .create(NETSHAPER_PIPE_NAME)
        .map_err(|e| e.to_string())?;

    Ok(pipe)
}

async fn handle_pipe_client(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
) -> Result<(), String> {
    info!("WFP kernel driver connected to pipe");

    let mut buf = vec![0u8; 4096];

    loop {
        // Wait for WFP driver to send packet metadata
        match pipe.read(&mut buf).await {
            Ok(0) => {
                // Connection closed
                info!("WFP driver disconnected");
                break;
            }
            Ok(n) => {
                // Try to deserialize packet metadata from WFP driver
                match bincode::deserialize::<PacketMetadata>(&buf[..n]) {
                    Ok(meta) => {
                        debug!(
                            "Packet from {}: {} bytes, packet_id={}",
                            meta.src_ip, meta.byte_len, meta.packet_id
                        );

                        // Consult token bucket for this IP
                        let decision = {
                            let mut map = token_buckets.lock().await;
                            let bucket = map
                                .entry(meta.src_ip)
                                .or_insert_with(|| TokenBucket::new(meta.src_ip, u64::MAX));

                            if bucket.consume(meta.byte_len as u64) {
                                PacketDecision::Permit {
                                    packet_id: meta.packet_id,
                                }
                            } else {
                                debug!(
                                    "Rate limit exceeded for {}: {} bytes",
                                    meta.src_ip, meta.byte_len
                                );
                                PacketDecision::Drop {
                                    packet_id: meta.packet_id,
                                }
                            }
                        };

                        // Send decision back to WFP driver
                        if let Ok(encoded) = bincode::serialize(&decision) {
                            if let Err(e) = pipe.write_all(&encoded).await {
                                warn!("Failed to send packet decision: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize packet metadata: {}", e);
                        // Continue reading despite serialization error
                    }
                }
            }
            Err(e) => {
                warn!("Pipe read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
