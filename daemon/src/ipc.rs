use crate::DeviceRegistry;
use proto::{BandwidthUpdate, DaemonCommand, DeviceState, PacketMetadata, PacketDecision};
use std::net::Ipv4Addr;
use std::sync::Arc;
/// IPC Server
/// ===========
/// Handles communication with the WFP kernel callout and Tauri UI over named pipes
///
/// On Windows: Uses Windows Named Pipes (NETSHAPER_PIPE_NAME)
/// On Unix/Dev: Uses Unix Domain Socket (/tmp/netshaper.sock)
///
/// Packet Flow:
/// 1. WFP kernel callout sends PacketMetadata over named pipe
/// 2. Daemon checks device registry for bandwidth limits
/// 3. Token bucket scheduler decides: Permit or Drop
/// 4. Daemon sends PacketDecision back to kernel callout
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Encode error: {0}")]
    Encode(String),

    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
}

/// Run the IPC server that listens for messages from the kernel callout and UI
pub async fn run_pipe_server(registry: Arc<Mutex<DeviceRegistry>>) -> Result<(), DaemonError> {
    #[cfg(windows)]
    {
        run_windows_pipe_server(registry).await
    }

    #[cfg(unix)]
    {
        run_unix_socket_server(registry).await
    }
}

/// Windows-specific named pipe server implementation
#[cfg(windows)]
async fn run_windows_pipe_server(registry: Arc<Mutex<DeviceRegistry>>) -> Result<(), DaemonError> {
    use std::ffi::OsStr;
    use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};

    let pipe_name = proto::NETSHAPER_PIPE_NAME;
    tracing::info!("Starting IPC server on named pipe: {}", pipe_name);

    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(false)
            .access_mode(winapi::um::winnt::GENERIC_READ | winapi::um::winnt::GENERIC_WRITE)
            .create(pipe_name)
            .map_err(|e| DaemonError::Io(e))?;

        tracing::debug!("Named pipe server instance created");

        // Handle client connection
        if let Err(e) = handle_client(server, registry.clone()).await {
            tracing::warn!("Error handling client: {}", e);
        }
    }
}

/// Unix-specific domain socket server (for development on non-Windows)
#[cfg(unix)]
async fn run_unix_socket_server(registry: Arc<Mutex<DeviceRegistry>>) -> Result<(), DaemonError> {
    use std::path::Path;
    use tokio::net::UnixListener;

    let socket_path = "/tmp/netshaper.sock";

    // Remove old socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path).map_err(|e| DaemonError::Io(e))?;
    }

    let listener = UnixListener::bind(socket_path).map_err(|e| DaemonError::Io(e))?;

    tracing::info!("Starting IPC server on Unix socket: {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                let registry = registry.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_unix_client(socket, registry).await {
                        tracing::warn!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Error accepting connection: {}", e);
            }
        }
    }
}

/// Handle a single client connection (Windows named pipe)
#[cfg(windows)]
async fn handle_client(
    mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    let mut buffer = vec![0u8; 65536]; // Larger buffer for potential packets

    loop {
        match pipe.read(&mut buffer).await {
            Ok(0) => {
                // Client disconnected
                tracing::debug!("Client disconnected");
                break;
            }
            Ok(n) => {
                // Try to parse as PacketMetadata first, then DaemonCommand
                if let Ok(packet) = bincode::deserialize::<PacketMetadata>(&buffer[..n]) {
                    // This is a packet from WFP kernel callout
                    if let Err(e) = process_packet(packet, registry.clone(), &mut pipe).await {
                        tracing::error!("Error processing packet: {}", e);
                    }
                } else if let Ok(cmd) = bincode::deserialize::<DaemonCommand>(&buffer[..n]) {
                    // This is a command from UI or other client
                    if let Err(e) = process_command(cmd, registry.clone(), &mut pipe).await {
                        tracing::error!("Error processing command: {}", e);
                    }
                } else {
                    tracing::warn!("Failed to deserialize message (not packet or command)");
                }
            }
            Err(e) => {
                tracing::error!("Error reading from pipe: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Handle a single client connection (Unix socket)
#[cfg(unix)]
async fn handle_unix_client(
    mut socket: tokio::net::UnixStream,
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    let mut buffer = vec![0u8; 65536]; // Larger buffer for potential packets

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                // Client disconnected
                tracing::debug!("Client disconnected");
                break;
            }
            Ok(n) => {
                // Try to parse as PacketMetadata first, then DaemonCommand
                if let Ok(packet) = bincode::deserialize::<PacketMetadata>(&buffer[..n]) {
                    // This is a packet from WFP kernel callout
                    if let Err(e) = process_unix_packet(packet, registry.clone(), &mut socket).await {
                        tracing::error!("Error processing packet: {}", e);
                    }
                } else if let Ok(cmd) = bincode::deserialize::<DaemonCommand>(&buffer[..n]) {
                    // This is a command from UI or other client
                    if let Err(e) = process_unix_command(cmd, registry.clone(), &mut socket).await {
                        tracing::error!("Error processing command: {}", e);
                    }
                } else {
                    tracing::warn!("Failed to deserialize message (not packet or command)");
                }
            }
            Err(e) => {
                tracing::error!("Error reading from socket: {}", e);
                break;
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
            let states = build_device_states(&reg);
            
            // Convert DeviceState to DeviceInfo format for UI
            let devices: Vec<(String, Option<String>, bool, String, u64, u64)> = states
                .iter()
                .map(|state| {
                    let ip_str = state.ip.to_string();
                    let approved = !state.is_blocked;
                    let now = chrono::Utc::now().to_rfc3339();
                    
                    (
                        ip_str,
                        state.hostname.clone(),
                        approved,
                        now,
                        state.bytes_per_sec,
                        state.current_usage,
                    )
                })
                .collect();
            
            let response = bincode::serialize(&devices)?;
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
    let response = bincode::serialize(&decision)?;
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
    let response = bincode::serialize(&decision)?;
    socket.write_all(&response).await?;

    Ok(())
}

/// Build DeviceState snapshots for all devices
fn build_device_states(registry: &crate::DeviceRegistry) -> Vec<DeviceState> {
    registry
        .list_devices()
        .iter()
        .map(|&ip| {
            let bucket = registry.get_bucket(ip).unwrap();
            DeviceState {
                ip,
                hostname: None, // TODO: Resolve hostname
                bytes_per_sec: bucket.allowed_bytes_per_sec,
                current_usage: 0, // TODO: Track rolling average
                is_blocked: bucket.allowed_bytes_per_sec == 0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_device_states() {
        let mut registry = crate::DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 1_000_000);
        let states = build_device_states(&registry);

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].ip, ip);
        assert_eq!(states[0].bytes_per_sec, 1_000_000);
        assert!(!states[0].is_blocked);
    }

    #[tokio::test]
    async fn test_build_device_states_blocked() {
        let mut registry = crate::DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 0); // Blocked
        let states = build_device_states(&registry);

        assert_eq!(states.len(), 1);
        assert!(states[0].is_blocked);
    }

    #[test]
    fn test_packet_metadata_serialization() {
        let packet = PacketMetadata {
            src_ip: "192.168.1.100".parse().unwrap(),
            dst_ip: "8.8.8.8".parse().unwrap(),
            byte_len: 1500,
            packet_id: 12345,
        };

        let encoded = bincode::serialize(&packet).expect("encode failed");
        let decoded: PacketMetadata = bincode::deserialize(&encoded).expect("decode failed");

        assert_eq!(packet.src_ip, decoded.src_ip);
        assert_eq!(packet.dst_ip, decoded.dst_ip);
        assert_eq!(packet.byte_len, decoded.byte_len);
        assert_eq!(packet.packet_id, decoded.packet_id);
    }

    #[test]
    fn test_packet_decision_serialization() {
        let decision = PacketDecision::Permit { packet_id: 999 };
        let encoded = bincode::serialize(&decision).expect("encode failed");
        let decoded: PacketDecision = bincode::deserialize(&encoded).expect("decode failed");
        assert_eq!(decision, decoded);
    }

    #[test]
    fn test_mixed_message_types() {
        // Verify that both PacketMetadata and DaemonCommand can be serialized
        let packet = PacketMetadata {
            src_ip: "192.168.1.100".parse().unwrap(),
            dst_ip: "8.8.8.8".parse().unwrap(),
            byte_len: 1500,
            packet_id: 12345,
        };

        let cmd = DaemonCommand::ListDevices;

        let packet_encoded = bincode::serialize(&packet).expect("packet encode failed");
        let cmd_encoded = bincode::serialize(&cmd).expect("cmd encode failed");

        // Try to parse each one - packet should NOT decode as command and vice versa
        assert!(bincode::deserialize::<DaemonCommand>(&packet_encoded).is_err());
        assert!(bincode::deserialize::<PacketMetadata>(&cmd_encoded).is_err());
    }
}
