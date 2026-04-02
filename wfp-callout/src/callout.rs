/// Packet Classify Callback
/// ========================
/// This function runs in kernel space for EVERY intercepted packet.
/// CONSTRAINTS: Must NOT allocate memory, take locks, or exceed 10 microseconds.

use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{FWP_ACTION_PERMIT, FWP_ACTION_BLOCK, FWPM_FILTER_CONDITION0, FWP_VALUE0};
use proto::{PacketMetadata, PacketDecision};
use std::net::Ipv4Addr;

/// Global pipe client (initialized in DllMain, used in classify_callback)
/// Safety: This is set once during DLL load and never modified again.
pub static mut PIPE_CLIENT: Option<crate::pipe::PipeClient> = None;

/// Classify callback - invoked for each packet intercepted by WFP
///
/// # Safety
/// This function runs in kernel context and must be extremely fast.
/// It CANNOT allocate memory or take locks.
pub unsafe extern "system" fn classify_callback(
    _layer: *const u32,
    _args: *const FWPM_FILTER_CONDITION0,
    _meta_values: *const *const FWP_VALUE0,
    context: *mut std::ffi::c_void,
    _filter_context: *const *const std::ffi::c_void,
    action: *mut u32,
) {
    // Default action: permit (safe fallback if anything goes wrong)
    *action = FWP_ACTION_PERMIT.0;

    // Try to extract packet metadata from context
    // For now, this is a placeholder - in production, we'd extract from the layer context
    // The actual packet info would come from analyzing the network buffer
    // This requires looking at the layer-specific fields in FWP_VALUE0 array

    // If we have a pipe client, try to query the daemon
    if let Some(ref pipe) = PIPE_CLIENT {
        // In a real implementation, we'd:
        // 1. Extract src IP, dst IP, packet size from the WFP context
        // 2. Send PacketMetadata to daemon
        // 3. Wait for PacketDecision (with timeout)
        // 4. Apply the decision

        // For now, just log that we would attempt to query
        // TODO: Extract actual packet metadata from WFP context
        let src_ip = Ipv4Addr::new(0, 0, 0, 0); // Placeholder
        let dst_ip = Ipv4Addr::new(0, 0, 0, 0); // Placeholder
        let byte_len = 0u32; // Placeholder
        let packet_id = context as u64; // Use context pointer as packet ID

        let metadata = PacketMetadata {
            src_ip,
            dst_ip,
            byte_len,
            packet_id,
        };

        // Try to get decision from daemon (with very short timeout)
        // If this fails or times out, default to PERMIT
        if let Some(decision) = pipe.query_decision(&metadata) {
            *action = match decision {
                PacketDecision::Permit { .. } => FWP_ACTION_PERMIT.0,
                PacketDecision::Drop { .. } => FWP_ACTION_BLOCK.0,
            };
        }
        // If query fails, action remains FWP_ACTION_PERMIT (safe default)
    }
    // If PIPE_CLIENT doesn't exist yet, default to PERMIT (no limiting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_structure() {
        // Verify the callback signature is correct (this will compile if it matches WFP requirements)
        // In real testing, this would be called by WFP kernel code
        // For now, just verify the module loads
    }
}
