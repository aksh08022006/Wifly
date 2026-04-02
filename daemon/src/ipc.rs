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

/// Run the IPC server that listens for messages from the kernel callout and UI
/// Accepts multiple concurrent clients and handles packet classification
pub async fn run_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    use tokio::net::windows::named_pipe::ServerOptions;
    
    let pipe_name = proto::NETSHAPER_PIPE_NAME;
    tracing::info!("IPC server listening on {}", pipe_name);
    
    loop {
        // Create a new server instance for each connection
        let server = ServerOptions::new()
            .create(pipe_name)
            .map_err(|e| {
                tracing::error!("Failed to create named pipe server: {}", e);
                DaemonError::Io(e)
            })?;
        
        // Wait for a client to connect (blocking until connection)
        server.connect().await.map_err(|e| {
            tracing::error!("Failed to accept client connection: {}", e);
            DaemonError::Io(e)
        })?;
        
        let registry_clone = registry.clone();
        
        // Spawn a task to handle this client connection
        // (allows multiple concurrent clients)
        tokio::spawn(async move {
            match handle_pipe_client(server, registry_clone).await {
                Ok(_) => tracing::debug!("Client disconnected"),
                Err(e) => tracing::warn!("Error handling client: {}", e),
            }
        });
    }
}

/// Handle a single named pipe client connection
/// Reads PacketMetadata, classifies, and sends back PacketDecision
async fn handle_pipe_client(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    registry: Arc<Mutex<DeviceRegistry>>,
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
