/// Named Pipe Client
/// ==================
/// Communicates with the daemon via named pipe.
/// Sends PacketMetadata, receives PacketDecision.
/// Must be extremely fast since it's called from classify callback.

use proto::{PacketMetadata, PacketDecision};
use std::sync::Mutex;
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{CreateFileA, OPEN_EXISTING, FILE_SHARE_NONE};
use windows::Win32::Storage::FileSystem::{FILE_FLAG_NO_BUFFERING, FILE_GENERIC_READ, FILE_GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{WriteFile, ReadFile};
use windows::core::s;

/// Client for communicating with daemon over named pipe
pub struct PipeClient {
    /// Mutex-protected handle to avoid concurrent access
    handle: Mutex<HANDLE>,
}

impl PipeClient {
    /// Connect to the daemon's named pipe
    /// Returns None if connection fails (daemon not running)
    pub fn connect() -> Option<Self> {
        unsafe {
            // Use the NETSHAPER_PIPE_NAME constant from proto
            let handle = CreateFileA(
                s!(r"\\.\pipe\netshaper"),
                FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_FLAG_NO_BUFFERING,
                HANDLE::default(),
            );

            let handle = match handle {
                Ok(h) => h,
                Err(_) => return None,
            };

            if handle == INVALID_HANDLE_VALUE {
                return None;
            }

            Some(PipeClient {
                handle: Mutex::new(handle),
            })
        }
    }

    /// Send PacketMetadata and wait for PacketDecision
    /// Returns None if communication fails
    pub fn query_decision(&self, metadata: &PacketMetadata) -> Option<PacketDecision> {
        // Try to serialize the metadata
        let serialized = bincode::serialize(metadata).ok()?;

        // Try to acquire the mutex without blocking (fail if another thread is writing)
        let handle_guard = self.handle.try_lock().ok()?;
        let handle = *handle_guard;

        // Write metadata to pipe
        unsafe {
            let write_result = WriteFile(
                handle,
                Some(serialized.as_slice()),
                None,
                None,
            );

            if write_result.is_err() {
                return None;
            }
        }

        // Read decision from pipe (with fixed buffer size)
        let mut response_buf = vec![0u8; 256];

        unsafe {
            let read_result = ReadFile(
                handle,
                Some(&mut response_buf[..]),
                None,
                None,
            );

            if read_result.is_err() {
                return None;
            }
        }

        // Deserialize the decision
        bincode::deserialize(&response_buf).ok()
    }
}

impl Drop for PipeClient {
    fn drop(&mut self) {
        // Close the handle if we still have it
        if let Ok(handle) = self.handle.lock() {
            if *handle != INVALID_HANDLE_VALUE {
                unsafe {
                    let _ = windows::Win32::Foundation::CloseHandle(*handle);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_pipe_client_creation() {
        // This will fail if daemon isn't running, which is expected in tests
        // In a CI environment without the daemon, this test should gracefully fail
        let client = PipeClient::connect();
        // We don't assert anything - just verify it doesn't crash
        drop(client);
    }

    #[test]
    fn test_packet_metadata_serialization() {
        // Test that we can serialize PacketMetadata
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 1),
            dst_ip: Ipv4Addr::new(192, 168, 1, 2),
            byte_len: 1500,
            packet_id: 42,
        };

        let serialized = bincode::serialize(&metadata);
        assert!(serialized.is_ok(), "Failed to serialize PacketMetadata");

        let bytes = serialized.unwrap();
        assert!(!bytes.is_empty(), "Serialized data should not be empty");
    }

    #[test]
    fn test_packet_decision_serialization() {
        // Test that we can serialize PacketDecision::Permit
        let permit_decision = PacketDecision::Permit {
            packet_id: 42,
        };

        let serialized = bincode::serialize(&permit_decision);
        assert!(serialized.is_ok(), "Failed to serialize Permit decision");

        // Test PacketDecision::Drop
        let drop_decision = PacketDecision::Drop {
            packet_id: 43,
        };

        let serialized = bincode::serialize(&drop_decision);
        assert!(serialized.is_ok(), "Failed to serialize Drop decision");
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test serialize -> deserialize roundtrip
        let metadata = PacketMetadata {
            src_ip: Ipv4Addr::new(10, 0, 0, 1),
            dst_ip: Ipv4Addr::new(10, 0, 0, 2),
            byte_len: 512,
            packet_id: 999,
        };

        let serialized = bincode::serialize(&metadata).expect("serialize failed");
        let deserialized: PacketMetadata = bincode::deserialize(&serialized).expect("deserialize failed");

        assert_eq!(deserialized.src_ip, metadata.src_ip);
        assert_eq!(deserialized.dst_ip, metadata.dst_ip);
        assert_eq!(deserialized.byte_len, metadata.byte_len);
        assert_eq!(deserialized.packet_id, metadata.packet_id);
    }
}
