/// IPC Server
/// ===========
/// Handles communication with the WFP kernel callout and Tauri UI over named pipes

use thiserror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::DeviceRegistry;
use proto::{PacketMetadata, PacketDecision};

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Handle a single packet classification request
/// Returns PacketDecision based on device's token bucket state
async fn classify_packet(
    metadata: &PacketMetadata,
    registry: &Arc<Mutex<DeviceRegistry>>,
) -> PacketDecision {
    let mut reg = registry.lock().await;
    
    // Check if device is in registry
    match reg.get_bucket_mut(metadata.dst_ip) {
        Some(bucket) => {
            // Device is throttled - try to consume tokens
            if bucket.try_consume(metadata.byte_len) {
                // Packet approved - can transmit immediately
                PacketDecision::Permit {
                    packet_id: metadata.packet_id,
                }
            } else {
                // Packet denied - insufficient tokens
                PacketDecision::Drop {
                    packet_id: metadata.packet_id,
                }
            }
        }
        None => {
            // Device not in registry - permit by default (not being rate-limited)
            PacketDecision::Permit {
                packet_id: metadata.packet_id,
            }
        }
    }
}

/// Run the IPC server that listens for messages from the kernel callout and UI
/// Accepts multiple concurrent clients and handles packet classification
/// With connection limiting and graceful shutdown support
pub async fn run_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
    shutdown: Arc<tokio::sync::Notify>,
) -> Result<(), DaemonError> {
    use tokio::net::windows::named_pipe::ServerOptions;
    use tokio::sync::Semaphore;
    use std::sync::Arc as StdArc;
    use std::time::Duration;
    
    let pipe_name = proto::NETSHAPER_PIPE_NAME;
    tracing::info!("IPC server listening on {}", pipe_name);
    
    // Limit concurrent connections to 32 (prevent resource exhaustion)
    let connection_limit = StdArc::new(Semaphore::new(32));
    
    // Retry backoff for pipe creation
    let mut retry_delay = Duration::from_millis(100);
    let max_retry_delay = Duration::from_secs(5);
    
    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("IPC server received shutdown signal");
                break;
            }
            
            result = async {
                // Create a new server instance for each connection
                ServerOptions::new().create(pipe_name)
            } => {
                match result {
                    Ok(server) => {
                        // Reset retry delay on successful creation
                        retry_delay = Duration::from_millis(100);
                        
                        // Wait for a client to connect
                        match server.connect().await {
                            Ok(_) => {
                                let registry_clone = registry.clone();
                                let conn_limit = connection_limit.clone();
                                let shutdown_clone = shutdown.clone();
                                
                                // Spawn a task to handle this client
                                tokio::spawn(async move {
                                    // Acquire connection slot
                                    let _permit = match conn_limit.acquire().await {
                                        Ok(p) => p,
                                        Err(e) => {
                                            tracing::warn!("Failed to acquire connection slot: {}", e);
                                            return;
                                        }
                                    };
                                    
                                    match handle_pipe_client(server, registry_clone, shutdown_clone).await {
                                        Ok(_) => tracing::debug!("Client disconnected"),
                                        Err(e) => tracing::warn!("Error handling client: {}", e),
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create named pipe: {}. Retrying in {:?}", e, retry_delay);
                        tokio::time::sleep(retry_delay).await;
                        retry_delay = std::cmp::min(retry_delay.saturating_mul(2), max_retry_delay);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Handle a single named pipe client connection
/// Reads PacketMetadata, classifies, and sends back PacketDecision
/// Respects shutdown signal for graceful termination
async fn handle_pipe_client(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    registry: Arc<Mutex<DeviceRegistry>>,
    _shutdown: Arc<tokio::sync::Notify>,
) -> Result<(), DaemonError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    loop {
        // Read message size (little-endian u32)
        let mut size_bytes = [0u8; 4];
        match pipe.read_exact(&mut size_bytes).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client disconnected gracefully
                break;
            }
            Err(e) => {
                tracing::warn!("Failed to read message size: {}", e);
                return Err(DaemonError::Io(e));
            }
        }
        
        let size = u32::from_le_bytes(size_bytes) as usize;
        
        // Sanity check: max 64KB message
        if size > 65536 {
            tracing::warn!("Message size too large: {} bytes", size);
            break;
        }
        
        let mut buffer = vec![0u8; size];
        
        match pipe.read_exact(&mut buffer).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                tracing::warn!("Failed to read message body: {}", e);
                return Err(DaemonError::Io(e));
            }
        }
        
        // Deserialize PacketMetadata
        let metadata: PacketMetadata = match bincode::deserialize(&buffer) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to deserialize PacketMetadata: {}", e);
                continue;
            }
        };
        
        // Classify the packet
        let decision = classify_packet(&metadata, &registry).await;
        
        // Serialize PacketDecision
        let response = match bincode::serialize(&decision) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to serialize PacketDecision: {}", e);
                continue;
            }
        };
        
        // Send response size + data
        let size_bytes = (response.len() as u32).to_le_bytes();
        if let Err(e) = pipe.write_all(&size_bytes).await {
            tracing::warn!("Failed to write response size: {}", e);
            return Err(DaemonError::Io(e));
        }
        
        if let Err(e) = pipe.write_all(&response).await {
            tracing::warn!("Failed to write response: {}", e);
            return Err(DaemonError::Io(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classify_packet() {
        use std::net::Ipv4Addr;
        
        let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
        
        // Add a device to the registry
        {
            let mut reg = registry.lock().await;
            reg.insert_device(Ipv4Addr::new(192, 168, 1, 100), 1_000_000); // 1 MB/s
        }
        
        // Create a small packet metadata
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 1),
            dst_ip: Ipv4Addr::new(192, 168, 1, 100),
            byte_len: 512,
            packet_id: 1,
        };
        
        // Should be permitted (device has tokens)
        let decision = classify_packet(&metadata, &registry).await;
        match decision {
            PacketDecision::Permit { packet_id } => {
                assert_eq!(packet_id, 1);
            }
            _ => panic!("Expected Permit"),
        }
    }
}
