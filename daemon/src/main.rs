use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::Mutex;

mod bucket;
mod device_registry;
mod ipc;
mod scheduler;

pub use bucket::DeviceBucket;
pub use device_registry::DeviceRegistry;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("NetShaper daemon starting");

    let registry = Arc::new(Mutex::new(DeviceRegistry::new()));

    // Spawn IPC server
    let ipc_handle = tokio::spawn({
        let registry = registry.clone();
        async move {
            if let Err(e) = ipc::run_pipe_server(registry).await {
                error!("IPC server error: {}", e);
            }
        }
    });

    // Spawn scheduler task
    let scheduler_handle = tokio::spawn({
        let registry = registry.clone();
        async move {
            if let Err(e) = scheduler::run_scheduler(registry).await {
                error!("Scheduler error: {}", e);
            }
        }
    });

    // Wait for tasks
    let _ = tokio::join!(ipc_handle, scheduler_handle);
    info!("NetShaper daemon exiting");
}
