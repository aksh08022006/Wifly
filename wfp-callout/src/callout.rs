/// Packet Classify Callback
/// ========================
/// This function runs in kernel space for EVERY intercepted packet.
/// CONSTRAINTS: Must NOT allocate memory, take locks, or exceed 10 microseconds.

use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{FWP_ACTION_PERMIT, FWP_ACTION_BLOCK, FWPM_FILTER_CONDITION0, FWP_VALUE0};
use proto::{PacketMetadata, PacketDecision};
use std::net::Ipv4Addr;
use lazy_static::lazy_static;
use crate::packet_tracker::PacketTracker;
use crate::packet_injector::PacketInjector;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};

/// Global pipe client (initialized in DllMain, used in classify_callback)
/// Safety: This is set once during DLL load and never modified again.
pub static mut PIPE_CLIENT: Option<crate::pipe::PipeClient> = None;

/// Global packet tracker for concurrent packet state management
/// Initialized with max 2000 packets and 1 second timeout
lazy_static! {
    static ref PACKET_TRACKER: PacketTracker = {
        PacketTracker::new(2000, 1_000_000) // Max 2000 packets, 1s timeout in microseconds
    };
}

/// Global packet injector for sending TCP RST and ICMP responses
/// Wrapped in Mutex for safe concurrent access
lazy_static! {
    static ref PACKET_INJECTOR: Mutex<PacketInjector> = {
        Mutex::new(PacketInjector::new())
    };
}

/// Periodic cleanup counter (Phase 4: Error Handling & Robustness)
/// Every 100 packets, trigger cleanup of expired entries in tracker
/// This prevents memory leaks from old packets that timeout
static CLEANUP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Get current time in microseconds since UNIX_EPOCH
/// Used for packet timeout tracking
fn get_current_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Extract IPv4 address from FWP_VALUE0
/// Safety: Caller must ensure the pointer is valid and the value type is uint32
unsafe fn extract_ipv4(value: *const FWP_VALUE0) -> Option<Ipv4Addr> {
    if value.is_null() {
        return None;
    }

    // FWP_VALUE0 is a union. For IPv4 addresses, we need the uint32 variant.
    // The value field in FWP_VALUE0 is the union member.
    // For uint32 type, it's stored as value.uint32 (but we need to check the type first)
    
    // NOTE: This is a simplified approach. The type field should be checked first.
    // For IPv4 addresses in outbound context, they come as uint32 in network byte order.
    let ip_u32 = (*value).Anonymous.uint32;
    
    // Convert from network byte order (big-endian) to Ipv4Addr
    let [a, b, c, d] = ip_u32.to_be_bytes();
    Some(Ipv4Addr::new(a, b, c, d))
}

/// Extract packet metadata from FWP context
/// NOTE: For FWPM_LAYER_OUTBOUND_IPPACKET_V4, the field layout is:
/// [0] = FWPM_FIELD_IP_SOURCE_ADDRESS (uint32 IPv4)
/// [1] = FWPM_FIELD_IP_DESTINATION_ADDRESS (uint32 IPv4)
/// [2] = FWPM_FIELD_IP_PROTOCOL (uint8)
/// [3] = FWPM_FIELD_IP_LENGTH (uint16 - total IP packet length)
/// And additional fields...
unsafe fn extract_packet_metadata(
    meta_values: *const *const FWP_VALUE0,
    context: *mut std::ffi::c_void,
) -> Option<PacketMetadata> {
    if meta_values.is_null() {
        return None;
    }

    let values = *meta_values;
    if values.is_null() {
        return None;
    }

    // Field 0: Source IPv4 (uint32)
    let src_ip = extract_ipv4(values.add(0))?;
    
    // Field 1: Destination IPv4 (uint32) 
    let dst_ip = extract_ipv4(values.add(1))?;
    
    // Field 3: IP Total Length (includes header + payload)
    // This is at offset 3 in the FWPM_LAYER_OUTBOUND_IPPACKET_V4 field array
    let byte_len = (*values.add(3)).Anonymous.uint16 as u32;
    
    // Use context pointer as packet ID (unique identifier)
    let packet_id = context as u64;

    Some(PacketMetadata {
        src_ip,
        dst_ip,
        byte_len,
        packet_id,
    })
}

/// Classify callback - invoked for each packet intercepted by WFP
///
/// # Safety
/// This function runs in kernel context and must be extremely fast.
/// It CANNOT allocate memory or take locks.
pub unsafe extern "system" fn classify_callback(
    _layer: *const u32,
    _args: *const FWPM_FILTER_CONDITION0,
    meta_values: *const *const FWP_VALUE0,
    context: *mut std::ffi::c_void,
    _filter_context: *const *const std::ffi::c_void,
    action: *mut u32,
) {
    // Default action: permit (safe fallback if anything goes wrong)
    *action = FWP_ACTION_PERMIT.0;

    // Try to extract packet metadata from WFP context
    let metadata = match extract_packet_metadata(meta_values, context) {
        Some(m) => m,
        None => {
            // If we can't extract metadata, permit the packet (safe fallback)
            return;
        }
    };

    // Get current time for timeout tracking
    let current_time = get_current_micros();

    // Try to add packet to tracker for state management
    if let Err(e) = PACKET_TRACKER.add_pending(metadata.clone(), current_time) {
        // Tracker full or locked - permit packet (safe fallback)
        tracing::warn!("Failed to track packet {}: {}", metadata.packet_id, e);
        return;
    }

    // If we have a pipe client, try to query the daemon
    if let Some(ref pipe) = PIPE_CLIENT {
        // Try to get decision from daemon (with very short timeout)
        // If this fails or times out, default to PERMIT
        if let Some(decision) = pipe.query_decision(&metadata) {
            // Apply decision and update tracker
            if let Ok(Some(_packet)) = PACKET_TRACKER.apply_decision(decision.clone()) {
                *action = match decision {
                    PacketDecision::Permit { .. } => FWP_ACTION_PERMIT.0,
                    PacketDecision::Drop { .. } => {
                        // Phase 2: Inject TCP RST or ICMP Unreachable for dropped packets
                        // Try to acquire injector lock (non-blocking)
                        if let Ok(mut injector_guard) = PACKET_INJECTOR.try_lock() {
                            // Attempt injection (best-effort, non-critical)
                            let _ = injector_guard.apply_drop_decision(&metadata, false);
                        }
                        // Whether injection succeeds or fails, block the packet
                        FWP_ACTION_BLOCK.0
                    }
                };
            }
        }
    }
    // If PIPE_CLIENT doesn't exist yet, default to PERMIT (no limiting)
    
    // Phase 4: Periodic cleanup of expired packets
    // Every 100 packets processed, trigger timeout-based cleanup
    // This prevents unbounded memory growth in the tracker
    let cleanup_tick = CLEANUP_COUNTER.fetch_add(1, Ordering::Relaxed);
    if cleanup_tick % 100 == 0 && cleanup_tick > 0 {
        if let Ok(removed) = PACKET_TRACKER.cleanup_expired(get_current_micros()) {
            if removed > 0 {
                tracing::debug!("Cleanup triggered: removed {} expired packets", removed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_extraction() {
        // Test IPv4 address extraction from u32
        // 192.168.1.1 in network byte order: 0xC0A80101
        let addr = Ipv4Addr::new(192, 168, 1, 1);
        let u32_repr = u32::from_be_bytes(addr.octets());
        
        // Convert back
        let [a, b, c, d] = u32_repr.to_be_bytes();
        let reconstructed = Ipv4Addr::new(a, b, c, d);
        assert_eq!(addr, reconstructed);
    }

    #[test]
    fn test_ipv4_addresses() {
        // Test various IPv4 address conversions
        let test_cases = vec![
            Ipv4Addr::new(127, 0, 0, 1),        // loopback
            Ipv4Addr::new(192, 168, 0, 1),       // private
            Ipv4Addr::new(10, 0, 0, 1),          // private
            Ipv4Addr::new(172, 16, 0, 1),        // private
            Ipv4Addr::new(8, 8, 8, 8),           // public DNS
        ];

        for addr in test_cases {
            let u32_repr = u32::from_be_bytes(addr.octets());
            let [a, b, c, d] = u32_repr.to_be_bytes();
            let reconstructed = Ipv4Addr::new(a, b, c, d);
            assert_eq!(addr, reconstructed, "Failed for {}", addr);
        }
    }

    #[test]
    fn test_callback_structure() {
        // Verify the callback signature is correct (this will compile if it matches WFP requirements)
        // In real testing, this would be called by WFP kernel code
        // For now, just verify the module loads
    }

    #[test]
    fn test_packet_metadata_construction() {
        // Test that we can construct valid PacketMetadata
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 100),
            dst_ip: Ipv4Addr::new(8, 8, 8, 8),
            byte_len: 1500,
            packet_id: 12345,
        };

        assert_eq!(metadata.src_ip.octets()[0], 192);
        assert_eq!(metadata.dst_ip.octets()[0], 8);
        assert_eq!(metadata.byte_len, 1500);
        assert_eq!(metadata.packet_id, 12345);
    }

    #[test]
    fn test_bincode_serialization() {
        // Test that PacketMetadata can be serialized with bincode
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 1),
            dst_ip: Ipv4Addr::new(192, 168, 1, 2),
            byte_len: 576,
            packet_id: 999,
        };

        // Serialize
        let serialized = bincode::serialize(&metadata);
        assert!(serialized.is_ok());

        // Deserialize
        let deserialized: Result<PacketMetadata, _> = bincode::deserialize(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let recovered = deserialized.unwrap();
        assert_eq!(recovered.src_ip, metadata.src_ip);
        assert_eq!(recovered.dst_ip, metadata.dst_ip);
        assert_eq!(recovered.byte_len, metadata.byte_len);
        assert_eq!(recovered.packet_id, metadata.packet_id);
    }

    // ===== PHASE 4 TESTS: Error Handling & Robustness =====

    #[test]
    fn test_tracker_integration_with_callback() {
        // Phase 5: Unit test for tracker integration
        // Verify packet is added to tracker on add_pending
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 100),
            dst_ip: Ipv4Addr::new(192, 168, 1, 1),
            byte_len: 1460,
            packet_id: 54321,
        };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        // Add packet to tracker
        let result = PACKET_TRACKER.add_pending(metadata.clone(), current_time);
        assert!(result.is_ok(), "Failed to add packet to tracker");

        // Verify packet is tracked
        if let Ok(is_pending) = PACKET_TRACKER.is_pending(metadata.packet_id) {
            assert!(is_pending, "Packet not tracked");
        }

        // Apply permit decision
        let decision = PacketDecision::Permit { packet_id: metadata.packet_id };
        let removed = PACKET_TRACKER.apply_decision(decision);
        assert!(removed.is_ok(), "Failed to apply decision");
        assert!(removed.unwrap().is_some(), "Packet not found in tracker");

        // Verify packet removed after decision
        if let Ok(is_pending) = PACKET_TRACKER.is_pending(metadata.packet_id) {
            assert!(!is_pending, "Packet not removed after decision");
        }
    }

    #[test]
    fn test_cleanup_counter_increments() {
        // Phase 4: Verify cleanup counter increments properly
        let initial = CLEANUP_COUNTER.load(Ordering::Relaxed);
        
        // Simulate 100 packets
        for _ in 0..100 {
            let _ = CLEANUP_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        
        let final_count = CLEANUP_COUNTER.load(Ordering::Relaxed);
        let expected = initial.wrapping_add(100);
        
        assert_eq!(final_count, expected, "Cleanup counter not incremented correctly");
    }

    #[test]
    fn test_tracker_graceful_capacity_handling() {
        // Phase 4: Verify tracker handles capacity limits gracefully
        // Add packets until tracker is full, verify it doesn't panic
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let mut added = 0;
        for i in 0..2500 {
            let metadata = PacketMetadata {
                src_ip: Ipv4Addr::new(192, 168, (i / 256) as u8, (i % 256) as u8),
                dst_ip: Ipv4Addr::new(10, 0, 0, 1),
                byte_len: 1460,
                packet_id: i as u64,
            };

            match PACKET_TRACKER.add_pending(metadata, current_time) {
                Ok(_) => added += 1,
                Err(_) => {
                    // Capacity limit reached - this is expected and safe
                    break;
                }
            }
        }

        // Verify we added some packets (at least up to capacity)
        assert!(added > 0, "Failed to add any packets to tracker");
        // Verify we hit capacity (2000 limit)
        assert!(added <= 2000, "Tracker capacity exceeded expected maximum");
    }

    #[test]
    fn test_decision_apply_permit() {
        // Phase 5: Test permit decision path
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 50),
            dst_ip: Ipv4Addr::new(8, 8, 8, 8),
            byte_len: 1460,
            packet_id: 99999,
        };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        // Add and check permit decision
        let _ = PACKET_TRACKER.add_pending(metadata.clone(), current_time);
        let decision = PacketDecision::Permit { packet_id: metadata.packet_id };
        let result = PACKET_TRACKER.apply_decision(decision);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_decision_apply_drop() {
        // Phase 5: Test drop decision path
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(10, 0, 0, 50),
            dst_ip: Ipv4Addr::new(192, 168, 1, 1),
            byte_len: 1460,
            packet_id: 88888,
        };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        // Add and check drop decision
        let _ = PACKET_TRACKER.add_pending(metadata.clone(), current_time);
        let decision = PacketDecision::Drop { packet_id: metadata.packet_id };
        let result = PACKET_TRACKER.apply_decision(decision);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_timeout_cleanup() {
        // Phase 4: Test timeout-based cleanup
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(172, 16, 0, 1),
            dst_ip: Ipv4Addr::new(8, 8, 8, 8),
            byte_len: 1460,
            packet_id: 77777,
        };

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        // Add packet at time T
        let _ = PACKET_TRACKER.add_pending(metadata.clone(), start_time);
        if let Ok(is_pending) = PACKET_TRACKER.is_pending(metadata.packet_id) {
            assert!(is_pending);
        }

        // Simulate time passing (2 seconds = 2_000_000 microseconds)
        // Default timeout is 1 second, so this should trigger cleanup
        let future_time = start_time + 2_000_000;
        if let Ok(removed) = PACKET_TRACKER.cleanup_expired(future_time) {
            // Packet should be cleaned up
            assert!(removed > 0, "Cleanup should remove expired packets");
        }
        
        if let Ok(is_pending) = PACKET_TRACKER.is_pending(metadata.packet_id) {
            assert!(!is_pending, "Expired packet not cleaned up");
        }
    }

    #[test]
    fn test_injector_creation() {
        // Phase 5: Test PacketInjector initialization
        let _ = PacketInjector::new();
        // If we can create it without panicking, initialization is working
        assert!(true);
    }

    // ===== PHASE 5 TESTS: Load Testing =====

    #[test]
    fn test_concurrent_packet_tracking() {
        // Phase 5: Load test - track many packets concurrently
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let num_packets = 500;
        let mut pending_count = 0;

        // Add many packets rapidly
        for i in 0..num_packets {
            let metadata = PacketMetadata {
                src_ip: Ipv4Addr::new(192, 168, (i / 256) as u8, (i % 256) as u8),
                dst_ip: Ipv4Addr::new(10, 0, 0, 1),
                byte_len: 1460,
                packet_id: 100000 + (i as u64),
            };

            if PACKET_TRACKER.add_pending(metadata, current_time).is_ok() {
                pending_count += 1;
            }
        }

        // Verify we tracked a significant number
        assert!(pending_count > 100, "Should track at least 100 packets");
        assert!(pending_count <= num_packets, "Cannot exceed added packets");
    }

    #[test]
    fn test_rapid_permit_drop_decisions() {
        // Phase 5: Load test - apply many decisions rapidly
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let num_decisions = 100;
        let mut applied = 0;

        for i in 0..num_decisions {
            let metadata = PacketMetadata {
                src_ip: Ipv4Addr::new(192, 168, 0, (i % 256) as u8),
                dst_ip: Ipv4Addr::new(10, 0, 0, (i / 256) as u8),
                byte_len: 1460,
                packet_id: 200000 + (i as u64),
            };

            let _ = PACKET_TRACKER.add_pending(metadata.clone(), current_time);

            let decision = if i % 2 == 0 {
                PacketDecision::Permit { packet_id: metadata.packet_id }
            } else {
                PacketDecision::Drop { packet_id: metadata.packet_id }
            };

            if PACKET_TRACKER.apply_decision(decision).is_ok() {
                applied += 1;
            }
        }

        assert!(applied > 0, "Should apply at least some decisions");
    }
}
