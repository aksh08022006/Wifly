// UI Library - M5 Device Management
// ==================================
// Tauri commands and application state

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================
// Error Types
// ============================================

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "error")]
pub enum UiError {
    #[error("Daemon connection failed: {0}")]
    DaemonConnection(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Invalid bandwidth: {0}")]
    InvalidBandwidth(String),

    #[error("IPC error: {0}")]
    IpcError(String),
}

// ============================================
// Data Structures
// ============================================

/// Device information displayed in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub ip: String,
    pub hostname: Option<String>,
    pub approved: bool,
    pub enrolled_at: String, // ISO 8601 timestamp
    pub bandwidth_limit: u64, // bytes per second
    pub current_usage: u64,   // bytes per second
}

/// Bandwidth metrics for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub device_ip: String,
    pub current: u64,     // bytes/sec
    pub average: u64,     // last 60 seconds
    pub peak: u64,        // last 60 seconds
    pub timestamp: String, // ISO 8601
}

// ============================================
// Application State
// ============================================

/// Global application state
pub struct AppState {
    // TODO: M5 Phase 2 - Add daemon IPC client
    // For now, just a placeholder
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {}
    }
}

// ============================================
// Mock Data Functions (for testing)
// ============================================

pub fn get_mock_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            ip: "192.168.1.100".to_string(),
            hostname: Some("iPhone-12".to_string()),
            approved: true,
            enrolled_at: "2026-04-03T15:30:45Z".to_string(),
            bandwidth_limit: 10_000_000,
            current_usage: 2_500_000,
        },
        DeviceInfo {
            ip: "192.168.1.101".to_string(),
            hostname: Some("MacBook-Pro".to_string()),
            approved: false,
            enrolled_at: "2026-04-03T16:00:00Z".to_string(),
            bandwidth_limit: 0,
            current_usage: 0,
        },
    ]
}

// ============================================
// Unit Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        // State created successfully
        let _ = state;
    }

    #[test]
    fn test_device_info_serialization() {
        let device = DeviceInfo {
            ip: "192.168.1.100".to_string(),
            hostname: Some("test-device".to_string()),
            approved: true,
            enrolled_at: "2026-04-03T15:30:45Z".to_string(),
            bandwidth_limit: 10_000_000,
            current_usage: 5_000_000,
        };

        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("192.168.1.100"));
        assert!(json.contains("test-device"));
    }

    #[test]
    fn test_mock_devices() {
        let devices = get_mock_devices();
        assert_eq!(devices.len(), 2);
        assert!(devices[0].approved);
        assert!(!devices[1].approved);
    }
}
