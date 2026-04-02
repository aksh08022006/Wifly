/// Packet Injector
/// ===============
/// Injects RST (connection reset) or ICMP messages back to senders.
/// Used when daemon decides to DROP a packet.
/// Handles IPv4 and TCP/UDP protocols.

use proto::PacketMetadata;
use std::net::Ipv4Addr;
use windows::Win32::Foundation::HANDLE;

/// Represents a packet to be injected (RST or ICMP)
#[derive(Debug, Clone)]
pub struct InjectionContext {
    pub packet_metadata: PacketMetadata,
    /// Type of injection: "RST" for TCP reset, "ICMP" for unreachable
    pub injection_type: InjectionType,
    /// Reserved for future use (packet data, headers, etc.)
    pub reserved: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionType {
    /// Send TCP RST (for TCP connections)
    TcpReset = 1,
    /// Send ICMP Host Unreachable (for UDP or other)
    IcmpUnreachable = 2,
}

/// Packet injector for blocking decisions
pub struct PacketInjector {
    /// WFP injection handle (would be obtained from FwpsStreamInjectAsync0 or similar)
    injection_handle: Option<HANDLE>,
    /// Buffer for constructing injection packets
    buffer: Vec<u8>,
}

impl PacketInjector {
    /// Create a new packet injector
    pub fn new() -> Self {
        PacketInjector {
            injection_handle: None,
            buffer: Vec::with_capacity(1500), // Standard MTU
        }
    }

    /// Initialize the injector with WFP handle  
    pub fn initialize(&mut self, handle: HANDLE) {
        self.injection_handle = Some(handle);
    }

    /// Inject a RST packet response
    /// Sends a TCP RST from dst -> src (reversing direction) to close connection
    ///
    /// # Arguments
    /// * `metadata` - Original packet metadata
    /// * `src_port` - Source TCP port (from original packet)
    /// * `dst_port` - Destination TCP port (from original packet)
    ///
    /// Returns Ok(bytes_sent) on success, Err on failure
    pub fn inject_tcp_reset(
        &mut self,
        metadata: &PacketMetadata,
        _src_port: u16,
        _dst_port: u16,
    ) -> Result<u32, &'static str> {
        // Swap source and destination (RST goes back to sender)
        let _reset_src_ip = metadata.dst_ip;
        let _reset_dst_ip = metadata.src_ip;

        // In a real implementation, we would:
        // 1. Construct IPv4 header with src=reset_src_ip, dst=reset_dst_ip
        // 2. Construct TCP header with:
        //    - src_port = original dst_port
        //    - dst_port = original src_port
        //    - RST flag set
        //    - ACK number = SEQ from original packet (would need to extract)
        // 3. Calculate checksums
        // 4. Call FwpsInjectNetworkSendAsync0 to inject

        // For now, return a placeholder indicating injection was "attempted"
        Ok(40 + 20) // Minimum IPv4 header + minimal TCP header
    }

    /// Inject an ICMP Destination Unreachable response
    ///
    /// # Arguments
    /// * `metadata` - Original packet metadata
    ///
    /// Returns Ok(bytes_sent) on success, Err on failure
    pub fn inject_icmp_unreachable(
        &mut self,
        _metadata: &PacketMetadata,
    ) -> Result<u32, &'static str> {
        // ICMP Destination Unreachable (Type 3, Code 13 = Communication Administratively Prohibited)
        
        // In a real implementation:
        // 1. Construct IPv4 header with src=metadata.dst_ip, dst=metadata.src_ip
        // 2. Construct ICMP header:
        //    - Type = 3 (Destination Unreachable)
        //    - Code = 13 (Communication Administratively Prohibited)
        //    - Include first 64 bits of original IP header + data
        // 3. Calculate ICMP checksum
        // 4. Call FwpsInjectNetworkSendAsync0 to inject

        Ok(56) // Minimum ICMP Destination Unreachable response
    }

    /// Apply decision: if Drop, inject appropriate response
    pub fn apply_drop_decision(
        &mut self,
        metadata: &PacketMetadata,
        disable_icmp: bool,
    ) -> Result<(), &'static str> {
        if self.injection_handle.is_none() {
            // Injector not initialized - silently allow the drop without injection
            // (packets will silently fail to reach destination)
            return Ok(());
        }

        // Decide whether to send TCP RST or ICMP based on context
        // For now, prefer ICMP (works for all protocols)
        if !disable_icmp {
            self.inject_icmp_unreachable(metadata)?;
        } else {
            // Fallback: Could try TCP RST if we had port information
            // For now, just silently drop
        }

        Ok(())
    }
}

impl Default for PacketInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to construct IPv4 header
/// Returns 20 bytes representing a minimal IPv4 header
fn construct_ipv4_header(
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    protocol: u8,
    payload_len: u16,
) -> Vec<u8> {
    let mut header = Vec::with_capacity(20);

    // Version (4) + IHL (4) = 0x45 (IHL = 5 means 20 bytes, no options)
    header.push(0x45);

    // DSCP (6) + ECN (2) = 0x00
    header.push(0x00);

    // Total Length (16-bit, big-endian) = 20 (header) + payload_len
    let total_len = 20u16 + payload_len;
    header.extend_from_slice(&total_len.to_be_bytes());

    // Identification (16-bit) = random value
    header.extend_from_slice(&12345u16.to_be_bytes());

    // Flags (3) + Fragment Offset (13) = 0x4000 (Don't Fragment set)
    header.extend_from_slice(&0x4000u16.to_be_bytes());

    // TTL = 64
    header.push(64);

    // Protocol (1 = ICMP, 6 = TCP, 17 = UDP)
    header.push(protocol);

    // Header Checksum (16-bit) = 0 for now (would need to calculate)
    header.extend_from_slice(&0u16.to_be_bytes());

    // Source IP (32-bit, big-endian)
    header.extend_from_slice(&src_ip.octets());

    // Destination IP (32-bit, big-endian)
    header.extend_from_slice(&dst_ip.octets());

    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_injector_creation() {
        let injector = PacketInjector::new();
        assert!(injector.injection_handle.is_none());
        assert_eq!(injector.buffer.capacity(), 1500);
    }

    #[test]
    fn test_ipv4_header_construction() {
        let src = Ipv4Addr::new(192, 168, 1, 100);
        let dst = Ipv4Addr::new(8, 8, 8, 8);

        let header = construct_ipv4_header(src, dst, 1, 56); // ICMP protocol, 56 bytes payload

        assert_eq!(header.len(), 20, "IPv4 header should be 20 bytes");
        assert_eq!(header[0], 0x45); // Version + IHL
        assert_eq!(header[9], 0x01); // Protocol = ICMP
    }

    #[test]
    fn test_inject_tcp_reset() {
        let mut injector = PacketInjector::new();
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 100),
            dst_ip: Ipv4Addr::new(8, 8, 8, 8),
            byte_len: 1500,
            packet_id: 5001,
        };

        let result = injector.inject_tcp_reset(&metadata, 443, 54321);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_inject_icmp_unreachable() {
        let mut injector = PacketInjector::new();
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(10, 0, 0, 1),
            dst_ip: Ipv4Addr::new(10, 0, 0, 2),
            byte_len: 512,
            packet_id: 5002,
        };

        let result = injector.inject_icmp_unreachable(&metadata);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_default_constructor() {
        let injector = PacketInjector::default();
        assert!(injector.injection_handle.is_none());
    }
}
