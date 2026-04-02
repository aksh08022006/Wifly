/// Scheduler
/// ==========
/// Periodically refills token buckets and drains queues

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::DeviceRegistry;

/// Run the packet scheduler task
/// Wakes every 1ms to:
/// 1. Refill all buckets based on elapsed time
/// 2. Drain packets that can now be transmitted
/// 3. Send PacketDecision messages back to kernel
pub async fn run_scheduler(
    registry: Arc<Mutex<DeviceRegistry>>,
    shutdown: Arc<tokio::sync::Notify>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Scheduler started");

    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("Scheduler received shutdown signal");
                break;
            }
            _ = sleep(Duration::from_millis(1)) => {
                let mut reg = registry.lock().await;
                
                // For each device in registry
                let device_ips: Vec<_> = reg.list_devices();
                
                for ip in device_ips {
                    if let Some(bucket) = reg.get_bucket_mut(ip) {
                        // Refill tokens based on elapsed time
                        bucket.refill();
                        
                        // Drain all packets that are now ready
                        let ready_packets = bucket.drain_ready();
                        
                        // Log ready packets (in production, send PacketDecision back to kernel)
                        if !ready_packets.is_empty() {
                            tracing::debug!(
                                "Drained {} packets for device {}, remaining capacity: {}",
                                ready_packets.len(),
                                ip,
                                bucket.current_tokens
                            );
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
