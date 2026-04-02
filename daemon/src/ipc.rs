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

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Encode error: {0}")]
    Encode(String),
}

/// Handle a single packet classification request
/// Returns PacketDecision based on device's token bucket state
#[allow(dead_code)]
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

/// Handle a single client connection (kernel callout or UI)
/// In production, this would handle a Windows named pipe connection
#[allow(dead_code)]
async fn handle_client_connection(
    stream: tokio::io::DuplexStream,
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    let mut stream = stream;
    loop {
        // Read message size (little-endian u32)
        let mut size_bytes = [0u8; 4];
        match stream.read_exact(&mut size_bytes).await {
            Ok(_) => {},
            Err(_) => break, // Connection closed or error
        }
        
        let size = u32::from_le_bytes(size_bytes) as usize;
        let mut buffer = vec![0u8; size];
        
        match stream.read_exact(&mut buffer).await {
            Ok(_) => {},
            Err(_) => break,
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
        
        // Serialize PacketDecision and send back
        let response = match bincode::serialize(&decision) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to serialize PacketDecision: {}", e);
                continue;
            }
        };
        
        // Write response size and data
        let size = (response.len() as u32).to_le_bytes();
        if let Err(e) = stream.write_all(&size).await {
            tracing::warn!("Failed to write response size: {}", e);
            break;
        }
        
        if let Err(e) = stream.write_all(&response).await {
            tracing::warn!("Failed to write response: {}", e);
            break;
        }
    }
    
    Ok(())
}

/// Run the IPC server that listens for messages from the kernel callout and UI
pub async fn run_pipe_server(
    _registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    // For development/testing, use a simple in-memory channel-based listener
    // In production, this would use Windows named pipes via tokio::net::windows::named_pipe
    
    tracing::info!("IPC server listening on {}", proto::NETSHAPER_PIPE_NAME);
    
    // Current implementation: wait for shutdown signal
    // TODO: Implement proper Windows named pipe listener
    // TODO: Accept multiple concurrent clients (one per kernel callout instance)
    
    tokio::signal::ctrl_c().await?;
    tracing::info!("IPC server shutting down");

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
