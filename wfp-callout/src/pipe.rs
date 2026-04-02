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

    #[test]
    fn test_pipe_client_creation() {
        // This will fail if daemon isn't running, which is expected in tests
        let _client = PipeClient::connect();
    }
}
