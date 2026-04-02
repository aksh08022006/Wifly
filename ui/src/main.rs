// Tauri Application Entry Point
// ==============================

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use tokio::sync::Mutex;

// TODO: Implement Tauri commands for:
// 1. list_devices() -> Vec<DeviceState>
// 2. set_bandwidth(ip, bytes_per_sec) -> Result<()>

/// Application state
pub struct AppState {
    // TODO: Add named pipe client connection
}

fn main() {
    // TODO: Scaffold Tauri application
    // - Initialize system tray
    // - Register Tauri commands
    // - Connect to named pipe
}
