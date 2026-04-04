/// Token Bucket Rate Limiter
/// ===========================
/// Implements per-IPv4 bandwidth throttling using token bucket algorithm.
/// Tokens are added over time based on the configured bytes_per_sec rate.
/// Packets consume tokens; if insufficient tokens, packet is dropped.

use std::net::Ipv4Addr;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TokenBucket {
    pub ip: Ipv4Addr,
    pub allowed_bytes_per_sec: u64,
    current_tokens: f64,
    last_refill: Instant,
    max_burst: f64, // Allow 2x rate as burst for short traffic spikes
}

impl TokenBucket {
    /// Create a new token bucket with the specified rate limit
    pub fn new(ip: Ipv4Addr, bytes_per_sec: u64) -> Self {
        Self {
            ip,
            allowed_bytes_per_sec: bytes_per_sec,
            current_tokens: bytes_per_sec as f64, // Start with full burst capacity
            last_refill: Instant::now(),
            max_burst: bytes_per_sec as f64 * 2.0,
        }
    }

    /// Try to consume bytes from the bucket
    /// Returns true if allowed, false if rate limit exceeded
    pub fn consume(&mut self, bytes: u64) -> bool {
        // Unlimited traffic
        if self.allowed_bytes_per_sec == u64::MAX {
            return true;
        }

        // Blocked traffic
        if self.allowed_bytes_per_sec == 0 {
            return false;
        }

        // Refill tokens based on elapsed time
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.last_refill = Instant::now();
        
        // Add new tokens but cap at max burst capacity
        self.current_tokens = (
            self.current_tokens + elapsed * self.allowed_bytes_per_sec as f64
        ).min(self.max_burst);

        // Try to consume tokens
        if self.current_tokens >= bytes as f64 {
            self.current_tokens -= bytes as f64;
            true
        } else {
            false
        }
    }

    /// Update the rate limit for this bucket
    pub fn update_limit(&mut self, bytes_per_sec: u64) {
        self.allowed_bytes_per_sec = bytes_per_sec;
        self.max_burst = bytes_per_sec as f64 * 2.0;
        // Reset token balance to allow immediate traffic at new rate
        self.current_tokens = bytes_per_sec as f64;
        self.last_refill = Instant::now();
        tracing::debug!(
            "Token bucket for {} updated to {} bytes/sec",
            self.ip,
            bytes_per_sec
        );
    }

    /// Get current token count (for debugging)
    pub fn current_tokens(&self) -> f64 {
        self.current_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_unlimited() {
        let mut bucket = TokenBucket::new("192.168.1.1".parse().unwrap(), u64::MAX);
        assert!(bucket.consume(1_000_000));
        assert!(bucket.consume(u64::MAX - 1));
    }

    #[test]
    fn test_token_bucket_blocked() {
        let mut bucket = TokenBucket::new("192.168.1.1".parse().unwrap(), 0);
        assert!(!bucket.consume(1));
    }

    #[test]
    fn test_token_bucket_rate_limit() {
        let mut bucket = TokenBucket::new("192.168.1.1".parse().unwrap(), 1_000_000); // 1MB/s
        
        // Should have 1MB initially
        assert!(bucket.consume(500_000)); // 500KB allowed
        assert!(bucket.consume(500_000)); // Another 500KB allowed
        assert!(!bucket.consume(100));    // 3rd packet denied
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new("192.168.1.1".parse().unwrap(), 1_000_000);
        bucket.consume(900_000);           // Use up most tokens
        
        // Simulate time passing
        bucket.last_refill = Instant::now() - std::time::Duration::from_millis(500);
        
        // After 500ms at 1MB/s, should have ~500KB new tokens
        assert!(bucket.consume(400_000));  // Should succeed
    }
}
