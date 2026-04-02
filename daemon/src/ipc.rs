/// IPC Server
/// ===========
/// Handles communication with the WFP kernel callout and Tauri UI over named pipes
/// 
/// On Windows: Uses Windows Named Pipes (NETSHAPER_PIPE_NAME)
/// On Unix/Dev: Uses Unix Domain Socket (/tmp/netshaper.sock)

use thiserror::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::DeviceRegistry;
use proto::{DaemonCommand, DeviceState, BandwidthUpdate};
use std::net::Ipv4Addr;

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
pub async fn run_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
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
async fn run_windows_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};
    use std::ffi::OsStr;

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
async fn run_unix_socket_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    use tokio::net::UnixListener;
    use std::path::Path;

    let socket_path = "/tmp/netshaper.sock";
    
    // Remove old socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)
            .map_err(|e| DaemonError::Io(e))?;
    }

    let listener = UnixListener::bind(socket_path)
        .map_err(|e| DaemonError::Io(e))?;

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
    let mut buffer = vec![0u8; 4096];

    loop {
        match pipe.read(&mut buffer).await {
            Ok(0) => {
                // Client disconnected
                tracing::debug!("Client disconnected");
                break;
            }
            Ok(n) => {
                // Process command
                match bincode::deserialize::<DaemonCommand>(&buffer[..n]) {
                    Ok(cmd) => {
                        if let Err(e) = process_command(cmd, registry.clone(), &mut pipe).await {
                            tracing::error!("Error processing command: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize command: {}", e);
                    }
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
    let mut buffer = vec![0u8; 4096];

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                // Client disconnected
                tracing::debug!("Client disconnected");
                break;
            }
            Ok(n) => {
                // Process command
                match bincode::deserialize::<DaemonCommand>(&buffer[..n]) {
                    Ok(cmd) => {
                        if let Err(e) = process_unix_command(cmd, registry.clone(), &mut socket).await {
                            tracing::error!("Error processing command: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize command: {}", e);
                    }
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
            reg.update_bandwidth(update.ip, update.bytes_per_sec);
            tracing::info!(
                "Updated bandwidth for {}: {} bytes/sec",
                update.ip,
                update.bytes_per_sec
            );
            Ok(())
        }
        DaemonCommand::ListDevices => {
            let reg = registry.lock().await;
            let devices = build_device_states(&reg);
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
            reg.update_bandwidth(update.ip, update.bytes_per_sec);
            tracing::info!(
                "Updated bandwidth for {}: {} bytes/sec",
                update.ip,
                update.bytes_per_sec
            );
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
}
