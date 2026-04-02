// Tauri Application Entry Point - M5 Phase 3
// ============================================
// NetShaper UI Dashboard with IPC Integration

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use ui::DeviceInfo;

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

#[cfg(windows)]
async fn send_command_to_daemon_windows(command: String) -> Result<(), String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    
    let mut pipe = ClientOptions::new()
        .open("\\\\.\\pipe\\netshaper")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let request = bincode::serialize(&command)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    pipe.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    Ok(())
}

#[cfg(unix)]
async fn send_command_to_daemon_unix(command: String) -> Result<(), String> {
    use tokio::net::UnixStream;
    
    let mut stream = UnixStream::connect("/tmp/netshaper.sock")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let request = bincode::serialize(&command)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    stream.write_all(&request)
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_devices, approve_device, deny_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
