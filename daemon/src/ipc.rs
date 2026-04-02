/// IPC Server
/// ===========
/// Handles communication with the WFP kernel callout and Tauri UI over named pipes

use thiserror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use crate::DeviceRegistry;
use proto::{PacketMetadata, PacketDecision, DaemonCommand};

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
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
    _registry: Arc<Mutex<DeviceRegistry>>,
    _shutdown: Arc<tokio::sync::Notify>,
) -> Result<(), DaemonError> {
    use tokio::io::AsyncReadExt;
    
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
    }

    Ok(())
}

/// Process a daemon command (Windows)
#[cfg(windows)]
async fn process_command(
    cmd: DaemonCommand,
    registry: Arc<Mutex<DeviceRegistry>>,
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeServer,
) -> Result<(), DaemonError> {
    match cmd {
        DaemonCommand::UpdateBandwidth(update) => {
            let mut reg = registry.lock().await;
            
            // If device doesn't exist and bandwidth > 0, create new bucket
            if !reg.list_devices().contains(&update.ip) && update.bytes_per_sec > 0 {
                reg.insert_device(update.ip, update.bytes_per_sec);
                tracing::info!(
                    "Enrolled new device {} with {} bytes/sec limit",
                    update.ip,
                    update.bytes_per_sec
                );
            } else {
                // Update existing device's bandwidth
                reg.update_bandwidth(update.ip, update.bytes_per_sec);
                let status = if update.bytes_per_sec == 0 { "blocked" } else { "approved" };
                tracing::info!(
                    "Device {} {}: {} bytes/sec",
                    update.ip,
                    status,
                    update.bytes_per_sec
                );
            }
            Ok(())
        }
        DaemonCommand::ListDevices => {
            let reg = registry.lock().await;
            
            // Convert devices to the expected format
            let devices: Vec<(String, Option<String>, bool, String, u64, u64)> = reg
                .list_devices()
                .iter()
                .map(|ip| {
                    let ip_str = ip.to_string();
                    // Device is approved if it has non-zero bandwidth limit
                    let bandwidth = reg.get_bucket(*ip).map(|b| b.allowed_bytes_per_sec).unwrap_or(0);
                    let approved = bandwidth > 0;
                    let now = format!("{:?}", std::time::SystemTime::now());
                    let current = reg.get_current_usage(*ip);
                    
                    (ip_str, None, approved, now, bandwidth, current)
                })
                .collect();
            
            let response = bincode::serialize(&devices)
                .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
            pipe.write_all(&response).await?;
            Ok(())
        }
        DaemonCommand::GetDeviceStats(ip) => {
            // M5 Phase 5: Return bandwidth stats for a single device
            let reg = registry.lock().await;
            let current_usage = reg.get_current_usage(ip);
            let peak_usage = reg.get_peak_usage(ip);
            let total_consumption = reg.get_total_consumption(ip);
            
            if let Some(bucket) = reg.get_bucket(ip) {
                let stats = proto::DeviceStats {
                    ip,
                    current_usage,
                    peak_usage,
                    total_consumption,
                    bandwidth_limit: bucket.allowed_bytes_per_sec,
                };
                let response = bincode::serialize(&stats)
                    .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
                pipe.write_all(&response).await?;
            }
            Ok(())
        }
        DaemonCommand::GetAllDeviceStats => {
            // M5 Phase 5: Return bandwidth stats for all devices
            let reg = registry.lock().await;
            let devices = reg.list_devices();
            
            let mut all_stats: Vec<proto::DeviceStats> = Vec::new();
            for ip in devices {
                let current_usage = reg.get_current_usage(ip);
                let peak_usage = reg.get_peak_usage(ip);
                let total_consumption = reg.get_total_consumption(ip);
                
                if let Some(bucket) = reg.get_bucket(ip) {
                    all_stats.push(proto::DeviceStats {
                        ip,
                        current_usage,
                        peak_usage,
                        total_consumption,
                        bandwidth_limit: bucket.allowed_bytes_per_sec,
                    });
                }
            }
            
            let response = bincode::serialize(&all_stats)
                .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
            pipe.write_all(&response).await?;
            Ok(())
        }
        DaemonCommand::Shutdown => {
            tracing::info!("Shutdown command received");
            std::process::exit(0);
        }
    }
}

/// Process a packet from WFP kernel callout (Windows)
#[cfg(windows)]
async fn process_packet(
    metadata: PacketMetadata,
    registry: Arc<Mutex<DeviceRegistry>>,
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeServer,
) -> Result<(), DaemonError> {
    let mut reg = registry.lock().await;

    // Check if source device is enrolled and get decision from token bucket
    let decision = if let Some(bucket) = reg.get_bucket_mut(metadata.src_ip) {
        if bucket.allowed_bytes_per_sec == 0 {
            // Device is completely blocked (denied)
            tracing::debug!("Dropping packet from blocked device: {}", metadata.src_ip);
            PacketDecision::Drop {
                packet_id: metadata.packet_id,
            }
        } else if bucket.try_consume(metadata.byte_len) {
            // Token bucket allowed the packet
            tracing::debug!(
                "Permitting packet from {}: {} bytes (tokens available)",
                metadata.src_ip,
                metadata.byte_len
            );
            PacketDecision::Permit {
                packet_id: metadata.packet_id,
            }
        } else {
            // Token bucket rejected (rate limited)
            tracing::debug!(
                "Dropping packet from {}: {} bytes (rate limited)",
                metadata.src_ip,
                metadata.byte_len
            );
            PacketDecision::Drop {
                packet_id: metadata.packet_id,
            }
        }
    } else {
        // Device not enrolled - block by default for security
        tracing::warn!("Dropping packet from unapproved device: {}", metadata.src_ip);
        PacketDecision::Drop {
            packet_id: metadata.packet_id,
        }
    };

    // Send decision back to kernel callout
    let response = bincode::serialize(&decision)
        .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
    pipe.write_all(&response).await?;

    Ok(())
}

/// Process a daemon command (Unix)
#[cfg(unix)]
async fn process_unix_command(
    cmd: DaemonCommand,
    registry: Arc<Mutex<DeviceRegistry>>,
    socket: &mut tokio::net::UnixStream,
) -> Result<(), DaemonError> {
    match cmd {
        DaemonCommand::UpdateBandwidth(update) => {
            let mut reg = registry.lock().await;
            
            // If device doesn't exist and bandwidth > 0, create new bucket
            if !reg.list_devices().contains(&update.ip) && update.bytes_per_sec > 0 {
                reg.insert_device(update.ip, update.bytes_per_sec);
                tracing::info!(
                    "Enrolled new device {} with {} bytes/sec limit",
                    update.ip,
                    update.bytes_per_sec
                );
            } else {
                // Update existing device's bandwidth
                reg.update_bandwidth(update.ip, update.bytes_per_sec);
                let status = if update.bytes_per_sec == 0 { "blocked" } else { "approved" };
                tracing::info!(
                    "Device {} {}: {} bytes/sec",
                    update.ip,
                    status,
                    update.bytes_per_sec
                );
            }
            Ok(())
        }
        DaemonCommand::ListDevices => {
            let reg = registry.lock().await;
            let devices = build_device_states(&reg);
            let response = bincode::serialize(&devices)?;
            socket.write_all(&response).await?;
            Ok(())
        }
        DaemonCommand::GetDeviceStats(ip) => {
            // M5 Phase 5: Return bandwidth stats for a single device
            let reg = registry.lock().await;
            let current_usage = reg.get_current_usage(ip);
            let peak_usage = reg.get_peak_usage(ip);
            let total_consumption = reg.get_total_consumption(ip);
            
            if let Some(bucket) = reg.get_bucket(ip) {
                let stats = proto::DeviceStats {
                    ip,
                    current_usage,
                    peak_usage,
                    total_consumption,
                    bandwidth_limit: bucket.allowed_bytes_per_sec,
                };
                let response = bincode::serialize(&stats)?;
                socket.write_all(&response).await?;
            }
            Ok(())
        }
        DaemonCommand::GetAllDeviceStats => {
            // M5 Phase 5: Return bandwidth stats for all devices
            let reg = registry.lock().await;
            let devices = reg.list_devices();
            
            let mut all_stats: Vec<proto::DeviceStats> = Vec::new();
            for ip in devices {
                let current_usage = reg.get_current_usage(ip);
                let peak_usage = reg.get_peak_usage(ip);
                let total_consumption = reg.get_total_consumption(ip);
                
                if let Some(bucket) = reg.get_bucket(ip) {
                    all_stats.push(proto::DeviceStats {
                        ip,
                        current_usage,
                        peak_usage,
                        total_consumption,
                        bandwidth_limit: bucket.allowed_bytes_per_sec,
                    });
                }
            }
            
            let response = bincode::serialize(&all_stats)
                .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
            socket.write_all(&response).await?;
            Ok(())
        }
        DaemonCommand::Shutdown => {
            tracing::info!("Shutdown command received");
            std::process::exit(0);
        }
    }
}

/// Process a packet from WFP kernel callout (Unix - for dev/testing)
#[cfg(unix)]
async fn process_unix_packet(
    metadata: PacketMetadata,
    registry: Arc<Mutex<DeviceRegistry>>,
    socket: &mut tokio::net::UnixStream,
) -> Result<(), DaemonError> {
    let mut reg = registry.lock().await;

    // Check if source device is enrolled and get decision from token bucket
    let decision = if let Some(bucket) = reg.get_bucket_mut(metadata.src_ip) {
        if bucket.allowed_bytes_per_sec == 0 {
            // Device is completely blocked (denied)
            tracing::debug!("Dropping packet from blocked device: {}", metadata.src_ip);
            PacketDecision::Drop {
                packet_id: metadata.packet_id,
            }
        } else if bucket.try_consume(metadata.byte_len) {
            // Token bucket allowed the packet
            tracing::debug!(
                "Permitting packet from {}: {} bytes (tokens available)",
                metadata.src_ip,
                metadata.byte_len
            );
            PacketDecision::Permit {
                packet_id: metadata.packet_id,
            }
        } else {
            // Token bucket rejected (rate limited)
            tracing::debug!(
                "Dropping packet from {}: {} bytes (rate limited)",
                metadata.src_ip,
                metadata.byte_len
            );
            PacketDecision::Drop {
                packet_id: metadata.packet_id,
            }
        }
    } else {
        // Device not enrolled - block by default for security
        tracing::warn!("Dropping packet from unapproved device: {}", metadata.src_ip);
        PacketDecision::Drop {
            packet_id: metadata.packet_id,
        }
    };

    // Send decision back to kernel callout
    let response = bincode::serialize(&decision)
        .map_err(|e| DaemonError::SerializationError(e.to_string()))?;
    socket.write_all(&response).await?;

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
