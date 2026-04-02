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
pub async fn run_scheduler(registry: Arc<Mutex<DeviceRegistry>>) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Scheduler started");

    loop {
        // TODO: Implement scheduler logic
        // 1. Lock registry
        // 2. For each device:
        //    - Call bucket.refill()
        //    - Call bucket.drain_ready()
        //    - Send PERMIT messages back to kernel for each ready packet
        // 3. Sleep for 1ms

        sleep(Duration::from_millis(1)).await;
    }
}
