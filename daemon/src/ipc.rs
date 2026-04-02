/// IPC Server
/// ===========
/// Handles communication with the WFP kernel callout and Tauri UI over named pipes

use thiserror::Error;
use proto::DaemonCommand;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::DeviceRegistry;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Encode error: {0}")]
    Encode(String),
}

/// Run the IPC server that listens for messages from the kernel callout and UI
pub async fn run_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    // TODO: Implement named pipe server
    // On Windows: use tokio::net::windows::named_pipe
    // On Unix (dev): use unix domain socket at /tmp/netshaper.sock

    tracing::info!("IPC server would start listening on {}", proto::NETSHAPER_PIPE_NAME);

    // For now, this is a placeholder
    tokio::signal::ctrl_c().await?;
    tracing::info!("IPC server shutting down");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipe_server_creation() {
        let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
        // Just verify it can be created without panic
        let _ = registry;
    }
}
