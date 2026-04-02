/// WFP Engine RAII Wrapper
/// =======================
/// Manages the Windows Filtering Platform engine session lifecycle

use windows::Win32::NetworkManagement::WindowsFilteringPlatform::*;
use windows::Win32::Foundation::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WfpError {
    #[error("Failed to open WFP engine: 0x{0:08X}")]
    EngineOpenFailed(u32),

    #[error("Failed to close WFP engine: 0x{0:08X}")]
    EngineCloseFailed(u32),

    #[error("Failed to register callout: 0x{0:08X}")]
    CalloutRegisterFailed(u32),

    #[error("Failed to add filter: 0x{0:08X}")]
    FilterAddFailed(u32),
}

/// RAII wrapper for WFP engine handle
pub struct WfpEngine {
    handle: HANDLE,
}

impl WfpEngine {
    /// Open a new WFP engine session
    /// Uses dynamic session mode and no security descriptor
    pub fn open() -> Result<Self, WfpError> {
        // TODO: Implement FwpmEngineOpen0 call
        // For now, return a placeholder handle
        Ok(WfpEngine {
            handle: Default::default(),
        })
    }

    /// Get the raw handle for use in other WFP API calls
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for WfpEngine {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: We're closing a handle we obtained from FwpmEngineOpen0.
            // If the handle is already closed or invalid, this is a double-free bug,
            // but that should never happen in correct code.
            if !self.handle.is_invalid() {
                let result = FwpmEngineClose0(self.handle);
                if result != 0 {
                    tracing::error!("Failed to close WFP engine: 0x{:08X}", result);
                }
            }
        }
    }
}
