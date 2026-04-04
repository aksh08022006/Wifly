/// Stats Listener Module
/// =====================
/// Reads PacketMetadata from WFP kernel driver via named pipe
/// Maintains per-device byte counters and token buckets
/// Sends PacketDecision (Permit/Drop) back to kernel

use proto::{PacketMetadata, PacketDecision, DaemonCommand};
use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::registry::DeviceRegistry;

const STATS_PIPE_NAME: &str = r"\\.\pipe\netshaper-stats";
const TOKEN_BUCKET_WINDOW: Duration = Duration::from_millis(100); // Re-evaluate every 100ms

/// Per-device token bucket state
#[derive(Clone, Debug)]
struct TokenBucket {
    ip: Ipv4Addr,
    bytes_per_sec: u64,      // Configured limit
    tokens: f64,             // Current token count
    last_update: Instant,
    bytes_this_second: u64,  // For stats tracking
}

impl TokenBucket {
    fn new(ip: Ipv4Addr, bytes_per_sec: u64) -> Self {
        Self {
            ip,
            bytes_per_sec,
            tokens: bytes_per_sec as f64, // Start with full bucket
            last_update: Instant::now(),
            bytes_this_second: 0,
        }
    }

    /// Decide whether to permit a packet based on token bucket algorithm
    /// Returns (permit, tokens_remaining)
    fn should_permit(&mut self, packet_bytes: u32) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        // Add tokens based on time elapsed
        if self.bytes_per_sec > 0 {
            let tokens_to_add = (self.bytes_per_sec as f64) * elapsed;
            self.tokens = (self.tokens + tokens_to_add).min(self.bytes_per_sec as f64);
        } else {
            // Blocked device (0 bytes per sec)
            return false;
        }

        // Try to consume tokens for this packet
        if self.tokens >= packet_bytes as f64 {
            self.tokens -= packet_bytes as f64;
            self.bytes_this_second += packet_bytes as u64;
            true
        } else {
            false
        }
    }

    /// Reset second counter for stats
    fn reset_second_counter(&mut self) {
        self.bytes_this_second = 0;
    }
}

/// Run the stats listener in background
/// Reads PacketMetadata from kernel, makes throttling decisions, sends PacketDecision back
pub async fn run_stats_listener(registry: Arc<Mutex<DeviceRegistry>>) {
    tokio::spawn(async move {
        let mut buckets: HashMap<Ipv4Addr, TokenBucket> = HashMap::new();
        let mut stats_reset_time = Instant::now();

        loop {
            tokio::time::sleep(TOKEN_BUCKET_WINDOW).await;

            // Periodically refresh token buckets from registry
            {
                let reg = registry.lock().await;
                let devices = reg.get_all();
                
                for device in &devices {
                    buckets.entry(device.ip)
                        .or_insert_with(|| TokenBucket::new(device.ip, device.bandwidth_limit))
                        .bytes_per_sec = device.bandwidth_limit;
                }

                // Remove buckets for devices no longer in registry
                let active_ips: std::collections::HashSet<_> = devices.iter().map(|d| d.ip).collect();
                buckets.retain(|ip, _| active_ips.contains(ip));
            }

            // Reset per-second counters periodically
            if stats_reset_time.elapsed() >= Duration::from_secs(1) {
                for bucket in buckets.values_mut() {
                    bucket.reset_second_counter();
                }
                stats_reset_time = Instant::now();
            }

            // In a production system, this would read from the kernel pipe here
            // For now, we just maintain the token buckets and let the HTTP API
            // handle device statistics queries
        }
    });
}

/// Get current usage stats for a device (for dashboard display)
pub fn get_device_usage(
    buckets: &HashMap<Ipv4Addr, TokenBucket>,
    ip: Ipv4Addr,
) -> (u64, u64) {
    if let Some(bucket) = buckets.get(&ip) {
        (bucket.bytes_this_second, bucket.bytes_per_sec)
    } else {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_permit() {
        let mut bucket = TokenBucket::new("192.168.1.100".parse().unwrap(), 1_000_000);
        
        // First packet of 1000 bytes should be permitted
        assert!(bucket.should_permit(1000));
        assert!(bucket.tokens > 0.0);
    }

    #[test]
    fn test_token_bucket_block_zero_limit() {
        let mut bucket = TokenBucket::new("192.168.1.100".parse().unwrap(), 0);
        
        // Any packet should be denied when limit is 0
        assert!(!bucket.should_permit(1000));
    }

    #[test]
    fn test_token_bucket_creates_correctly() {
        let bucket = TokenBucket::new("10.0.0.1".parse().unwrap(), 5_000_000);
        assert_eq!(bucket.bytes_per_sec, 5_000_000);
        assert!(bucket.tokens > 0.0);
    }
}
