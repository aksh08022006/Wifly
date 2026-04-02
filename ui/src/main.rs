// Tauri Application Entry Point - M5 Phase 3
// ============================================
// NetShaper UI Dashboard with IPC Integration

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::fs::File;
use ui::DeviceInfo;

// Default bandwidth limit for approved devices (10 MB/s)
const DEFAULT_BANDWIDTH_LIMIT: u64 = 10_000_000;

#[derive(Debug, Serialize, Deserialize)]
struct RawDeviceData {
    ip: String,
    hostname: Option<String>,
    approved: bool,
    enrolled_at: String,
    bandwidth_limit: u64,
    current_usage: u64,
}

/// Fetch list of devices from daemon IPC
#[tauri::command]
async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    tokio::task::spawn_blocking(fetch_devices_from_daemon_impl)
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

fn fetch_devices_from_daemon_impl() -> Result<Vec<DeviceInfo>, String> {
    let pipe_name = "\\\\.\\pipe\\netshaper";
    
    // Open named pipe (blocking I/O)
    let mut pipe = File::open(pipe_name)
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    // Send list_devices command
    let request = bincode::serialize(&"list_devices".to_string())
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut buffer = vec![0; 16384];
    let n = pipe.read(&mut buffer)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    buffer.truncate(n);

    // Deserialize and convert
    let raw_devices: Vec<(String, Option<String>, bool, String, u64, u64)> = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;

    let devices = raw_devices
        .into_iter()
        .map(|(ip, hostname, approved, enrolled_at, bandwidth_limit, current_usage)| DeviceInfo {
            ip,
            hostname,
            approved,
            enrolled_at,
            bandwidth_limit,
            current_usage,
        })
        .collect();

    Ok(devices)
}

/// Approve a device by IP address
#[tauri::command]
async fn approve_device(ip: String) -> Result<(), String> {
    let ip_clone = ip.clone();
    tokio::task::spawn_blocking(move || send_command_impl(format!("approve:{}", ip_clone)))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

/// Deny/block a device by IP address
#[tauri::command]
async fn deny_device(ip: String) -> Result<(), String> {
    let ip_clone = ip.clone();
    tokio::task::spawn_blocking(move || send_command_impl(format!("deny:{}", ip_clone)))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

/// M5 Phase 5: Get bandwidth stats for a device
#[tauri::command]
async fn get_device_stats(ip: String) -> Result<(u64, u64, u64), String> {
    let ip_clone = ip.clone();
    tokio::task::spawn_blocking(move || get_device_stats_impl(ip_clone))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

/// M5 Phase 5: Get bandwidth stats for all devices
#[tauri::command]
async fn get_all_device_stats() -> Result<Vec<(String, u64, u64, u64, u64)>, String> {
    tokio::task::spawn_blocking(get_all_device_stats_impl)
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

fn send_command_impl(command: String) -> Result<(), String> {
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    use proto::BandwidthUpdate;
    
    let pipe_name = "\\\\.\\pipe\\netshaper";
    let mut pipe = File::open(pipe_name)
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Parse command: "approve:192.168.1.100" or "deny:192.168.1.100"
    let (action, ip_str) = if let Some(pos) = command.find(':') {
        (&command[..pos], &command[pos+1..])
    } else {
        return Err("Invalid command format".to_string());
    };

    let ip: Ipv4Addr = ip_str.parse()
        .map_err(|_| format!("Invalid IP: {}", ip_str))?;

    let cmd = match action {
        "approve" => DaemonCommand::UpdateBandwidth(BandwidthUpdate {
            ip,
            bytes_per_sec: DEFAULT_BANDWIDTH_LIMIT,
        }),
        "deny" => DaemonCommand::UpdateBandwidth(BandwidthUpdate {
            ip,
            bytes_per_sec: 0, // 0 = blocked
        }),
        _ => return Err(format!("Unknown action: {}", action)),
    };

    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .map_err(|e| format!("Send error: {}", e))?;

    Ok(())
}

fn get_device_stats_impl(ip: String) -> Result<(u64, u64, u64), String> {
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    
    let parsed_ip: Ipv4Addr = ip.parse()
        .map_err(|_| format!("Invalid IP: {}", ip))?;

    let pipe_name = "\\\\.\\pipe\\netshaper";
    let mut pipe = File::open(pipe_name)
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetDeviceStats(parsed_ip);
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 1024];
    let n = pipe.read(&mut buffer)
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let stats: proto::DeviceStats = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok((stats.current_usage, stats.peak_usage, stats.total_consumption))
}

fn get_all_device_stats_impl() -> Result<Vec<(String, u64, u64, u64, u64)>, String> {
    use proto::DaemonCommand;
    
    let pipe_name = "\\\\.\\pipe\\netshaper";
    let mut pipe = File::open(pipe_name)
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetAllDeviceStats;
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 65536];
    let n = pipe.read(&mut buffer)
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let stats_list: Vec<proto::DeviceStats> = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    let result = stats_list
        .into_iter()
        .map(|stats| {
            (
                stats.ip.to_string(),
                stats.current_usage,
                stats.peak_usage,
                stats.total_consumption,
                stats.bandwidth_limit,
            )
        })
        .collect();
    
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile::app_entry)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_devices,
            approve_device,
            deny_device,
            get_device_stats,
            get_all_device_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(mobile))]
fn main() {
    run();
}
