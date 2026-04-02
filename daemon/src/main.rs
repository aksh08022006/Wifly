use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

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

    // Load enrolled devices from ~/.netshaper/devices.json
    {
        let devices_path =
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
                .join(".netshaper/devices.json");

        match crypto::DeviceList::load_from_disk(&devices_path) {
            Ok(device_list) => {
                let mut reg = registry.lock().await;
                for ip in device_list.approved_devices() {
                    // Default bandwidth: 10 MB/s (80 Mbps)
                    reg.insert_device(ip, 10_000_000);
                    info!("Loaded enrolled device: {} with 10 MB/s limit", ip);
                }
            }
            Err(e) => {
                warn!("Failed to load enrolled devices: {}", e);
            }
        }
    }

    // Load or generate TLS certificates
    let cert_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
        .join(".netshaper");

    let cert = match crypto::CertBundle::load_or_generate(&cert_dir) {
        Ok(cert) => {
            info!("TLS certificates loaded/generated successfully");
            cert
        }
        Err(e) => {
            error!("Failed to load/generate certificates: {}", e);
            return;
        }
    };

    // Create EnrolledDevices tracker for consent server
    let enrolled_devices = Arc::new(Mutex::new(crypto::handshake::EnrolledDevices::new()));

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

    // Spawn enrollment server (TLS on :7979)
    let enrollment_handle = tokio::spawn({
        async move {
            if let Err(e) = crypto::run_consent_server(cert, enrolled_devices).await {
                error!("Enrollment server error: {}", e);
            }
        }
    });

    // Wait for tasks
    let _ = tokio::join!(ipc_handle, scheduler_handle, enrollment_handle);
    info!("NetShaper daemon exiting");
}
