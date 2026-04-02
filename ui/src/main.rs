// Tauri Application Entry Point - M5 Phase 3
// ============================================
// NetShaper UI Dashboard with IPC Integration

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    let devices = fetch_devices_from_daemon().await?;
    Ok(devices)
}

async fn fetch_devices_from_daemon() -> Result<Vec<DeviceInfo>, String> {
    #[cfg(windows)]
    {
        connect_to_daemon_windows().await
    }

    #[cfg(unix)]
    {
        connect_to_daemon_unix().await
    }
}

#[cfg(windows)]
async fn connect_to_daemon_windows() -> Result<Vec<DeviceInfo>, String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    
    let pipe_name = "\\\\.\\pipe\\netshaper";
    
    let mut pipe = ClientOptions::new()
        .open(pipe_name)
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    // Send list_devices command
    let request = bincode::serialize(&"list_devices".to_string())
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut buffer = vec![0; 16384];
    let n = pipe.read(&mut buffer)
        .await
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

#[cfg(unix)]
async fn connect_to_daemon_unix() -> Result<Vec<DeviceInfo>, String> {
    use tokio::net::UnixStream;
    
    let mut stream = UnixStream::connect("/tmp/netshaper.sock")
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    // Send list_devices command
    let request = bincode::serialize(&"list_devices".to_string())
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    stream.write_all(&request)
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut buffer = vec![0; 16384];
    let n = stream.read(&mut buffer)
        .await
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
    #[cfg(windows)]
    {
        send_command_to_daemon_windows(format!("approve:{}", ip)).await
    }

    #[cfg(unix)]
    {
        send_command_to_daemon_unix(format!("approve:{}", ip)).await
    }
}

/// Deny/block a device by IP address
#[tauri::command]
async fn deny_device(ip: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        send_command_to_daemon_windows(format!("deny:{}", ip)).await
    }

    #[cfg(unix)]
    {
        send_command_to_daemon_unix(format!("deny:{}", ip)).await
    }
}

/// M5 Phase 5: Get bandwidth stats for a device
#[tauri::command]
async fn get_device_stats(ip: String) -> Result<(u64, u64, u64), String> {
    // Returns (current_usage, peak_usage, total_consumption)
    #[cfg(windows)]
    {
        get_device_stats_windows(ip).await
    }

    #[cfg(unix)]
    {
        get_device_stats_unix(ip).await
    }
}

/// M5 Phase 5: Get bandwidth stats for all devices
#[tauri::command]
async fn get_all_device_stats() -> Result<Vec<(String, u64, u64, u64, u64)>, String> {
    // Returns Vec<(ip, current_usage, peak_usage, total_consumption, bandwidth_limit)>
    #[cfg(windows)]
    {
        get_all_device_stats_windows().await
    }

    #[cfg(unix)]
    {
        get_all_device_stats_unix().await
    }
}

#[cfg(windows)]
async fn send_command_to_daemon_windows(command: String) -> Result<(), String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    use proto::BandwidthUpdate;
    
    let mut pipe = ClientOptions::new()
        .open("\\\\.\\pipe\\netshaper")
        .await
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
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    Ok(())
}

#[cfg(unix)]
async fn send_command_to_daemon_unix(command: String) -> Result<(), String> {
    use tokio::net::UnixStream;
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    use proto::BandwidthUpdate;
    
    let mut stream = UnixStream::connect("/tmp/netshaper.sock")
        .await
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
    
    stream.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    Ok(())
}

#[cfg(windows)]
async fn get_device_stats_windows(ip: String) -> Result<(u64, u64, u64), String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    
    let parsed_ip: Ipv4Addr = ip.parse()
        .map_err(|_| format!("Invalid IP: {}", ip))?;

    let mut pipe = ClientOptions::new()
        .open("\\\\.\\pipe\\netshaper")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetDeviceStats(parsed_ip);
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 1024];
    let n = pipe.read(&mut buffer)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let stats: proto::DeviceStats = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok((stats.current_usage, stats.peak_usage, stats.total_consumption))
}

#[cfg(unix)]
async fn get_device_stats_unix(ip: String) -> Result<(u64, u64, u64), String> {
    use tokio::net::UnixStream;
    use std::net::Ipv4Addr;
    use proto::DaemonCommand;
    
    let parsed_ip: Ipv4Addr = ip.parse()
        .map_err(|_| format!("Invalid IP: {}", ip))?;

    let mut stream = UnixStream::connect("/tmp/netshaper.sock")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetDeviceStats(parsed_ip);
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    stream.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let stats: proto::DeviceStats = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok((stats.current_usage, stats.peak_usage, stats.total_consumption))
}

#[cfg(windows)]
async fn get_all_device_stats_windows() -> Result<Vec<(String, u64, u64, u64, u64)>, String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    use proto::DaemonCommand;
    
    let mut pipe = ClientOptions::new()
        .open("\\\\.\\pipe\\netshaper")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetAllDeviceStats;
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 65536];
    let n = pipe.read(&mut buffer)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let all_stats: Vec<proto::DeviceStats> = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok(all_stats.into_iter().map(|s| {
        (s.ip.to_string(), s.current_usage, s.peak_usage, s.total_consumption, s.bandwidth_limit)
    }).collect())
}

#[cfg(unix)]
async fn get_all_device_stats_unix() -> Result<Vec<(String, u64, u64, u64, u64)>, String> {
    use tokio::net::UnixStream;
    use proto::DaemonCommand;
    
    let mut stream = UnixStream::connect("/tmp/netshaper.sock")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let cmd = DaemonCommand::GetAllDeviceStats;
    let request = bincode::serialize(&cmd)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    stream.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    let mut buffer = vec![0; 65536];
    let n = stream.read(&mut buffer)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    
    buffer.truncate(n);
    
    let all_stats: Vec<proto::DeviceStats> = bincode::deserialize(&buffer)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok(all_stats.into_iter().map(|s| {
        (s.ip.to_string(), s.current_usage, s.peak_usage, s.total_consumption, s.bandwidth_limit)
    }).collect())
}

fn main() {
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
