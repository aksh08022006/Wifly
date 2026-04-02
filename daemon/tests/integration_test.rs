/// Integration tests for daemon
/// Tests the full system working together: registry + buckets + scheduler

use daemon::DeviceRegistry;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use std::thread;

#[test]
fn test_multiple_devices_with_different_rates() {
    let mut registry = DeviceRegistry::new();

    // Add 3 devices with different bandwidth limits
    let ip1: Ipv4Addr = "192.168.1.100".parse().unwrap();
    let ip2: Ipv4Addr = "192.168.1.101".parse().unwrap();
    let ip3: Ipv4Addr = "192.168.1.102".parse().unwrap();

    registry.insert_device(ip1, 1_000_000);  // 1 MB/s
    registry.insert_device(ip2, 5_000_000);  // 5 MB/s
    registry.insert_device(ip3, 100_000);    // 100 KB/s

    // Verify all are registered
    assert_eq!(registry.count(), 3);

    // Verify each has the correct bandwidth
    assert_eq!(registry.get_bucket(ip1).unwrap().allowed_bytes_per_sec, 1_000_000);
    assert_eq!(registry.get_bucket(ip2).unwrap().allowed_bytes_per_sec, 5_000_000);
    assert_eq!(registry.get_bucket(ip3).unwrap().allowed_bytes_per_sec, 100_000);
}

#[test]
fn test_update_bandwidth_dynamically() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    // Add device at 1 MB/s
    registry.insert_device(ip, 1_000_000);
    assert_eq!(registry.get_bucket(ip).unwrap().allowed_bytes_per_sec, 1_000_000);

    // Update to 5 MB/s
    registry.update_bandwidth(ip, 5_000_000);
    assert_eq!(registry.get_bucket(ip).unwrap().allowed_bytes_per_sec, 5_000_000);

    // Update to 0 (block the device)
    registry.update_bandwidth(ip, 0);
    assert_eq!(registry.get_bucket(ip).unwrap().allowed_bytes_per_sec, 0);
}

#[test]
fn test_token_bucket_with_real_timing() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    // 1 MB/s bandwidth
    registry.insert_device(ip, 1_000_000);

    // Consume all initial tokens (burst capacity = 2 MB)
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        bucket.current_tokens = 0.0; // Simulate empty bucket
    }

    let start = Instant::now();

    // Wait 100ms
    thread::sleep(Duration::from_millis(100));

    // Refill should have added ~100KB (1 MB/s * 0.1s)
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        bucket.refill();
        let expected = 1_000_000.0 * 0.1;
        assert!(bucket.current_tokens >= expected * 0.95); // Allow 5% timing variance
        assert!(bucket.current_tokens <= expected * 1.05);
    }

    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(95)); // Some timing tolerance
}

#[test]
fn test_consume_tokens_success_and_failure() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    registry.insert_device(ip, 1_000_000);

    // Set bucket to have exactly 1000 bytes
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        bucket.current_tokens = 1000.0;
    }

    // Try to consume 500 bytes - should succeed
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        assert!(bucket.try_consume(500));
        assert_eq!(bucket.current_tokens, 500.0);
    }

    // Try to consume 1000 bytes - should fail (only 500 available)
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        assert!(!bucket.try_consume(1000));
        assert_eq!(bucket.current_tokens, 500.0); // Unchanged
    }

    // Try to consume 500 bytes - should succeed
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        assert!(bucket.try_consume(500));
        assert_eq!(bucket.current_tokens, 0.0);
    }
}

#[test]
fn test_device_removal() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    registry.insert_device(ip, 1_000_000);
    assert_eq!(registry.count(), 1);

    // Remove device
    let removed = registry.remove_device(ip);
    assert!(removed.is_some());
    assert_eq!(registry.count(), 0);

    // Verify it's gone
    assert!(registry.get_bucket(ip).is_none());
}

#[test]
fn test_burst_capacity_enforcement() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    registry.insert_device(ip, 1_000_000);

    let bucket = registry.get_bucket(ip).unwrap();
    // Burst should be 2x the rate (2 MB)
    assert_eq!(bucket.max_burst_bytes, 2_000_000);
    // Starts full
    assert_eq!(bucket.current_tokens as u64, 2_000_000);
}

#[test]
fn test_queue_depth_tracking() {
    let mut registry = DeviceRegistry::new();
    let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

    registry.insert_device(ip, 100_000); // 100 KB/s

    // Drain all tokens
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        bucket.current_tokens = 0.0;
    }

    // Enqueue multiple packets
    {
        let bucket = registry.get_bucket(ip).unwrap();
        for i in 0..5 {
            bucket.enqueue(daemon::bucket::DeferredPacket {
                packet_id: i,
                byte_len: 1000,
                queued_at: Instant::now(),
            });
        }
    }

    // Check queue depth
    {
        let bucket = registry.get_bucket(ip).unwrap();
        assert_eq!(bucket.queue_depth(), 5);
    }

    // Drain ready packets (none should be ready since tokens = 0)
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        let ready = bucket.drain_ready();
        assert_eq!(ready.len(), 0);
        assert_eq!(bucket.queue_depth(), 5); // All still in queue
    }

    // Wait and refill
    thread::sleep(Duration::from_millis(50));
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        bucket.refill();
        // ~5 KB should now be available
        assert!(bucket.current_tokens >= 4_000.0);
    }

    // Drain ready packets
    {
        let bucket = registry.get_bucket_mut(ip).unwrap();
        let ready = bucket.drain_ready();
        // Should be able to drain at least 4 packets (4 KB of 5 packets at 1 KB each)
        assert!(ready.len() >= 4);
    }
}
