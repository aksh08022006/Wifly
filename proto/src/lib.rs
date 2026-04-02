/// NetShaper IPC Protocol
/// =====================
/// This is the contract between all crates. CRITICAL: Any change requires both Aksh and Saksham to review.
use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};

/// Named pipe path for IPC communication
pub const NETSHAPER_PIPE_NAME: &str = r"\\.\pipe\netshaper";

/// Kernel callout → daemon: metadata for every intercepted packet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketMetadata {
    pub src_ip: Ipv4Addr,
    pub dst_ip: Ipv4Addr,
    pub byte_len: u32,
    pub packet_id: u64, // opaque handle, passed back in PacketDecision
}

/// Daemon → kernel callout: decision for a deferred packet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PacketDecision {
    Permit { packet_id: u64 },
    Drop { packet_id: u64 },
}

/// UI → daemon: update a device's bandwidth ceiling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BandwidthUpdate {
    pub ip: Ipv4Addr,
    pub bytes_per_sec: u64, // 0 = full block, u64::MAX = unlimited
}

/// Daemon → UI: current state snapshot of all managed devices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceState {
    pub ip: Ipv4Addr,
    pub hostname: Option<String>,
    pub bytes_per_sec: u64,
    pub current_usage: u64, // rolling 1s average in bytes
    pub is_blocked: bool,
}

/// Commands sent to the daemon over IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    UpdateBandwidth(BandwidthUpdate),
    ListDevices,
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_metadata_roundtrip() {
        let original = PacketMetadata {
            src_ip: "192.168.1.100".parse().unwrap(),
            dst_ip: "8.8.8.8".parse().unwrap(),
            byte_len: 1500,
            packet_id: 12345,
        };

        let encoded = bincode::serialize(&original).expect("encode failed");
        let decoded: PacketMetadata = bincode::deserialize(&encoded).expect("decode failed");

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_packet_decision_roundtrip() {
        let original = PacketDecision::Permit { packet_id: 999 };
        let encoded = bincode::serialize(&original).expect("encode failed");
        let decoded: PacketDecision = bincode::deserialize(&encoded).expect("decode failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bandwidth_update_roundtrip() {
        let original = BandwidthUpdate {
            ip: "192.168.1.50".parse().unwrap(),
            bytes_per_sec: 1_000_000, // 1 MB/s
        };

        let encoded = bincode::serialize(&original).expect("encode failed");
        let decoded: BandwidthUpdate = bincode::deserialize(&encoded).expect("decode failed");

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_device_state_roundtrip() {
        let original = DeviceState {
            ip: "192.168.1.75".parse().unwrap(),
            hostname: Some("john-phone".to_string()),
            bytes_per_sec: 5_000_000,
            current_usage: 2_500_000,
            is_blocked: false,
        };

        let encoded = bincode::serialize(&original).expect("encode failed");
        let decoded: DeviceState = bincode::deserialize(&encoded).expect("decode failed");

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_daemon_command_roundtrip() {
        let cmd = DaemonCommand::UpdateBandwidth(BandwidthUpdate {
            ip: "10.0.0.1".parse().unwrap(),
            bytes_per_sec: 500_000,
        });

        let encoded = bincode::serialize(&cmd).expect("encode failed");
        let decoded: DaemonCommand = bincode::deserialize(&encoded).expect("decode failed");

        match (cmd, decoded) {
            (
                DaemonCommand::UpdateBandwidth(a),
                DaemonCommand::UpdateBandwidth(b),
            ) => {
                assert_eq!(a, b);
            }
            _ => panic!("Command type mismatch"),
        }
    }
}
