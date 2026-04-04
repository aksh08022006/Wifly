/// WFP Bridge Module
/// =================
/// Manages bandwidth limits for approved devices via token bucket algorithm.
/// When UI sets a bandwidth limit, this updates the token bucket for that device's IP.
/// The actual packet throttling happens in the named pipe server when WFP driver
/// sends packet metadata.

use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::token_bucket::TokenBucket;

/// Set bandwidth limit for a device by updating its token bucket
/// 
/// # Arguments
/// * `token_buckets` - Shared token bucket map
/// * `ip` - Device IP address to throttle
/// * `bytes_per_sec` - Bandwidth limit (0 = block, u64::MAX = unlimited)
pub async fn set_bandwidth(
    token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
    ip: Ipv4Addr,
    bytes_per_sec: u64,
) {
    let mut map = token_buckets.lock().await;
    
    match map.get_mut(&ip) {
        Some(bucket) => {
            bucket.update_limit(bytes_per_sec);
            tracing::info!("WFP: Updated bandwidth for {} to {} bytes/sec", ip, bytes_per_sec);
        }
        None => {
            let mut bucket = TokenBucket::new(ip, bytes_per_sec);
            tracing::info!("WFP: Created new token bucket for {} with limit {} bytes/sec", ip, bytes_per_sec);
            map.insert(ip, bucket);
        }
    }
}

/// Block all traffic from a device (set bandwidth to 0)
pub async fn block_device(
    token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
    ip: Ipv4Addr,
) {
    tracing::info!("WFP: Blocking device {}", ip);
    set_bandwidth(token_buckets, ip, 0).await
}

/// Allow unlimited traffic from a device
pub async fn unblock_device(
    token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
    ip: Ipv4Addr,
) {
    tracing::info!("WFP: Unblocking device {}", ip);
    set_bandwidth(token_buckets, ip, u64::MAX).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_bandwidth_creates_bucket() {
        let buckets = Arc::new(Mutex::new(HashMap::new()));
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();
        
        set_bandwidth(buckets.clone(), ip, 1_000_000).await;
        
        let map = buckets.lock().await;
        assert!(map.contains_key(&ip));
    }

    #[tokio::test]
    async fn test_block_device() {
        let buckets = Arc::new(Mutex::new(HashMap::new()));
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();
        
        block_device(buckets.clone(), ip).await;
        
        let map = buckets.lock().await;
        assert_eq!(map[&ip].allowed_bytes_per_sec, 0);
    }
}
