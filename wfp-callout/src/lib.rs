#![allow(non_snake_case)]

/// WFP Callout Library
/// ===================
/// Windows Filtering Platform kernel-mode callout for packet interception and rate limiting

mod engine;
mod pipe;
mod callout;

pub use engine::WfpEngine;
pub use pipe::PipeClient;
pub use callout::classify_callback;

// Re-export proto types for use by this crate
extern crate proto;

// Note: This crate runs in kernel mode with limited functionality
// All business logic happens in the daemon userspace process

#[no_mangle]
pub extern "system" fn DllMain(
    _module: isize,
    reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> i32 {
    match reason {
        1 => {
            // DLL_PROCESS_ATTACH
            // Initialize WFP engine and pipe client
            unsafe {
                // Try to connect to daemon's named pipe
                if let Some(pipe_client) = PipeClient::connect() {
                    callout::PIPE_CLIENT = Some(pipe_client);
                }
            }
            1 // Return TRUE (success)
        }
        0 => {
            // DLL_PROCESS_DETACH
            // Clean up - PIPE_CLIENT will be dropped automatically
            1
        }
        _ => 1,
    }
}
