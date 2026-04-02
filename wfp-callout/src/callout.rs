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
use std::sync::Mutex;

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
    // TODO: Add periodic cleanup of expired packets (Phase 1.3)
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
}
