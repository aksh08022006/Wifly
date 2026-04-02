/// Token Bucket Rate Limiter
/// ==========================
/// Implements the token bucket algorithm for per-device bandwidth limiting.
/// Each device gets a bucket with a maximum capacity (burst) that refills at a constant rate.

use std::time::Instant;
use crossbeam_queue::SegQueue;

/// A packet waiting to be released or dropped
#[derive(Debug, Clone)]
pub struct DeferredPacket {
    pub packet_id: u64,
    pub byte_len: u32,
    pub queued_at: Instant,
}

/// Token bucket for one device
#[derive(Debug)]
pub struct DeviceBucket {
    pub allowed_bytes_per_sec: u64,   // configured bandwidth ceiling
    pub max_burst_bytes: u64,          // bucket capacity (usually 2x one-second allowance)
    pub current_tokens: f64,           // fractional bytes available right now
    pub last_refill: Instant,          // when we last added tokens
    pub queue: SegQueue<DeferredPacket>, // packets waiting for tokens
}

impl DeviceBucket {
    /// Create a new bucket with the given bandwidth ceiling
    pub fn new(bytes_per_sec: u64) -> Self {
        let max_burst = bytes_per_sec.saturating_mul(2);
        Self {
            allowed_bytes_per_sec: bytes_per_sec,
            max_burst_bytes: max_burst,
            current_tokens: max_burst as f64, // start full
            last_refill: Instant::now(),
            queue: SegQueue::new(),
        }
    }

    /// Refill tokens based on elapsed time
    pub fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.current_tokens = (self.current_tokens
            + elapsed * self.allowed_bytes_per_sec as f64)
            .min(self.max_burst_bytes as f64);
        self.last_refill = Instant::now();
    }

    /// Try to consume bytes from the bucket
    /// Returns true if successful (packet can go through)
    /// Returns false if insufficient tokens (packet must be queued)
    pub fn try_consume(&mut self, bytes: u32) -> bool {
        self.refill();
        let bytes_f64 = bytes as f64;
        if self.current_tokens >= bytes_f64 {
            self.current_tokens -= bytes_f64;
            true
        } else {
            false
        }
    }

    /// Enqueue a deferred packet
    pub fn enqueue(&self, packet: DeferredPacket) {
        self.queue.push(packet);
    }

    /// Get the current queue depth (number of waiting packets)
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    /// Drain all ready packets from the queue
    pub fn drain_ready(&mut self) -> Vec<DeferredPacket> {
        self.refill();
        let mut ready = Vec::new();

        while let Some(packet) = self.queue.pop() {
            let bytes_f64 = packet.byte_len as f64;
            if self.current_tokens >= bytes_f64 {
                self.current_tokens -= bytes_f64;
                ready.push(packet);
            } else {
                // Re-queue if not ready
                self.queue.push(packet);
                break;
            }
        }

        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_refill_adds_tokens() {
        let mut bucket = DeviceBucket::new(1_000_000); // 1 MB/s
        bucket.current_tokens = 0.0;
        bucket.last_refill = Instant::now();

        thread::sleep(Duration::from_millis(100));

        bucket.refill();
        let expected_tokens = 1_000_000.0 * 0.1; // 100ms * 1 MB/s
        assert!(bucket.current_tokens >= expected_tokens * 0.95); // 95% due to timing variance
    }

    #[test]
    fn test_try_consume_succeeds_when_available() {
        let mut bucket = DeviceBucket::new(1_000_000);
        bucket.current_tokens = 5000.0;

        assert!(bucket.try_consume(1000));
        assert_eq!(bucket.current_tokens, 4000.0);
    }

    #[test]
    fn test_try_consume_fails_when_empty() {
        let mut bucket = DeviceBucket::new(1_000_000);
        bucket.current_tokens = 0.0;

        assert!(!bucket.try_consume(1000));
        assert_eq!(bucket.current_tokens, 0.0);
    }

    #[test]
    fn test_burst_cap() {
        let mut bucket = DeviceBucket::new(1_000_000);
        assert_eq!(bucket.max_burst_bytes, 2_000_000);
        assert_eq!(bucket.current_tokens as u64, 2_000_000);
    }

    #[test]
    fn test_throttle_timing() {
        let mut bucket = DeviceBucket::new(100_000); // 100 KB/s
        bucket.current_tokens = 0.0;
        bucket.last_refill = Instant::now();

        let start = Instant::now();

        // Try to consume 10 KB 10 times with small delays
        for _ in 0..10 {
            while !bucket.try_consume(10_000) {
                thread::sleep(Duration::from_millis(10));
            }
        }

        let elapsed = start.elapsed();
        // Should take ~1 second to drain 100 KB at 100 KB/s
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed <= Duration::from_millis(1100));
    }
}
