use tracing::{info, error, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;

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
    let shutdown_signal = Arc::new(Notify::new());

    // Spawn IPC server
    let ipc_handle = tokio::spawn({
        let registry = registry.clone();
        let shutdown = shutdown_signal.clone();
        async move {
            if let Err(e) = ipc::run_pipe_server(registry, shutdown).await {
                error!("IPC server error: {}", e);
            }
        }
    });

    // Spawn scheduler task
    let scheduler_handle = tokio::spawn({
        let registry = registry.clone();
        let shutdown = shutdown_signal.clone();
        async move {
            if let Err(e) = scheduler::run_scheduler(registry, shutdown).await {
                error!("Scheduler error: {}", e);
            }
        }
    });

    // Wait for CTRL+C or other termination signal
    let shutdown_handle = tokio::spawn({
        let shutdown = shutdown_signal.clone();
        async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                warn!("Failed to listen for CTRL+C: {}", e);
            } else {
                info!("Received shutdown signal (CTRL+C)");
            }
            shutdown.notify_waiters();
        }
    });

    // Wait for first task to finish or shutdown signal
    let mut ipc_handle = ipc_handle;
    let mut scheduler_handle = scheduler_handle;
    let mut shutdown_handle = shutdown_handle;
    
    tokio::select! {
        res = &mut ipc_handle => {
            match res {
                Ok(_) => warn!("IPC server task exited"),
                Err(e) => error!("IPC server panicked: {}", e),
            }
            shutdown_signal.notify_waiters();
        }
        res = &mut scheduler_handle => {
            match res {
                Ok(_) => warn!("Scheduler task exited"),
                Err(e) => error!("Scheduler panicked: {}", e),
            }
            shutdown_signal.notify_waiters();
        }
        res = &mut shutdown_handle => {
            match res {
                Ok(_) => info!("Graceful shutdown initiated"),
                Err(e) => error!("Shutdown handler panicked: {}", e),
            }
        }
    }

    // Wait for all tasks to finish
    let _ = tokio::join!(ipc_handle, scheduler_handle, shutdown_handle);
    info!("NetShaper daemon exiting");
}
