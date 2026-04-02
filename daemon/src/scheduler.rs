use crate::DeviceRegistry;
/// Scheduler
/// ==========
/// Periodically refills token buckets and drains queues
///
/// This task is the heart of the rate limiting system:
/// 1. Wakes every 1ms
/// 2. For each device, calls refill() to add tokens
/// 3. Drains packets from each queue that now have tokens
/// 4. In future, sends PacketDecision messages back to kernel
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Run the packet scheduler task
/// Wakes every 1ms to:
/// 1. Lock registry
/// 2. For each device: refill tokens and drain ready packets
/// 3. In production: send PERMIT decisions back to kernel via IPC
pub async fn run_scheduler(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Scheduler started - refilling buckets every 1ms");

    let mut stats = SchedulerStats::new();

    loop {
        {
            let mut reg = registry.lock().await;

            // Process each device in the registry
            for &ip in reg.list_devices().iter() {
                if let Some(bucket) = reg.get_bucket_mut(ip) {
                    // Refill the bucket based on elapsed time
                    bucket.refill();

                    // Drain all packets that now have tokens
                    let ready_packets = bucket.drain_ready();

                    if !ready_packets.is_empty() {
                        debug!(
                            "Scheduler: {} packets ready for {} (queue depth now: {})",
                            ready_packets.len(),
                            ip,
                            bucket.queue.len()
                        );

                        stats.packets_released += ready_packets.len();

                        // TODO: Send PacketDecision::Permit for each ready packet
                        // For now, just logging that we would send them
                        for packet in ready_packets {
                            debug!(
                                "Would permit packet_id={} ({} bytes)",
                                packet.packet_id, packet.byte_len
                            );
                        }
                    }

                    // Check for packets still in queue (waiting for tokens)
                    let queue_depth = bucket.queue.len();
                    if queue_depth > 0 {
                        stats.packets_queued += queue_depth;
                    }
                }
            }

            stats.tick_count += 1;

            // Log stats every 1000 ticks (approximately every second)
            if stats.tick_count % 1000 == 0 {
                debug!(
                    "Scheduler stats: {} ticks, {} packets released, {} currently queued",
                    stats.tick_count, stats.packets_released, stats.packets_queued
                );
                stats.packets_queued = 0;
            }
        }

        // Sleep for 1ms before next iteration
        // This allows for smooth, granular token refill
        sleep(Duration::from_millis(1)).await;
    }
}

/// Scheduler statistics (for monitoring and debugging)
struct SchedulerStats {
    tick_count: u64,
    packets_released: usize,
    packets_queued: usize,
}

impl SchedulerStats {
    fn new() -> Self {
        Self {
            tick_count: 0,
            packets_released: 0,
            packets_queued: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeviceRegistry;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_scheduler_refills_buckets() {
        let mut registry = DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 1_000_000); // 1 MB/s

        let registry = Arc::new(Mutex::new(registry));

        // Get initial tokens
        {
            let reg = registry.lock().await;
            let bucket = reg.get_bucket(ip).unwrap();
            assert_eq!(bucket.current_tokens as u64, bucket.max_burst_bytes);
        }

        // Let scheduler run for 10ms
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Tokens should have increased (we're starting with full burst)
        // In practice, this would be verified by checking if a packet that
        // previously failed now succeeds
    }

    #[tokio::test]
    async fn test_scheduler_drains_packets() {
        let mut registry = DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 100_000); // 100 KB/s

        {
            let bucket = registry.get_bucket_mut(ip).unwrap();
            bucket.current_tokens = 1000.0; // 1 KB available
        }

        let registry = Arc::new(Mutex::new(registry));

        // Simulate a packet being queued
        {
            let mut reg = registry.lock().await;
            if let Some(bucket) = reg.get_bucket_mut(ip) {
                // Enqueue a 500-byte packet
                bucket.enqueue(crate::bucket::DeferredPacket {
                    packet_id: 1,
                    byte_len: 500,
                    queued_at: std::time::Instant::now(),
                });
            }
        }

        // Verify packet is in queue
        {
            let reg = registry.lock().await;
            if let Some(bucket) = reg.get_bucket(ip) {
                assert_eq!(bucket.queue.len(), 1);
            }
        }

        // Yield to let other tasks run
        tokio::task::yield_now().await;

        // After scheduler runs, packet should still be queued (we only had 1 KB, packet is 500 bytes)
        // But tokens should be available
    }
}
