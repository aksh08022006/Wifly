// Tauri Application Entry Point
// ==============================
// NetShaper UI Dashboard with HTTP API Integration
// All communication via HTTP to daemon on http://172.17.44.89:8080

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};

const DAEMON_URL: &str = "http://172.17.44.89:8080";
const HTTP_TIMEOUT: u64 = 5; // seconds

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub ip: String,
    pub platform: String,
    pub device_name: String,
    pub approved: bool,
    pub bandwidth_limit: u64,  // bytes per second
    pub enrolled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrResponse {
    pub qr_image: String,
    pub pairing_url: String,
}

// ============ Tauri Commands ============

/// GET /qr - Fetch QR code + pairing URL
#[tauri::command]
async fn http_get_qr() -> Result<QrResponse, String> {
    let url = format!("{}/qr", DAEMON_URL);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    response
        .json::<QrResponse>()
        .await
        .map_err(|e| format!("Failed to parse QR response: {}", e))
}

/// GET /devices - Fetch list of devices
#[tauri::command]
async fn http_get_devices() -> Result<Vec<Device>, String> {
    let url = format!("{}/devices", DAEMON_URL);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    #[derive(Deserialize)]
    struct DevicesResponse {
        devices: Vec<Device>,
    }
    
    let data = response
        .json::<DevicesResponse>()
        .await
        .map_err(|e| format!("Failed to parse devices response: {}", e))?;
    
    Ok(data.devices)
}

/// POST /devices/:id/approve - Approve device
#[tauri::command]
async fn http_approve_device(device_id: String, bandwidth_mb: u64) -> Result<(), String> {
    let url = format!("{}/devices/{}/approve", DAEMON_URL, device_id);
    let client = reqwest::Client::new();
    
    #[derive(Serialize)]
    struct ApproveBody {
        bandwidth_limit_mb: u64,
    }
    
    let body = ApproveBody {
        bandwidth_limit_mb: bandwidth_mb,
    };
    
    let response = client
        .post(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Approval failed: {}", response.status()))
    }
}

/// POST /devices/:id/deny - Deny device
#[tauri::command]
async fn http_deny_device(device_id: String) -> Result<(), String> {
    let url = format!("{}/devices/{}/deny", DAEMON_URL, device_id);
    let client = reqwest::Client::new();
    
    let response = client
        .post(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Denial failed: {}", response.status()))
    }
}

/// PUT /devices/:id/bandwidth - Set bandwidth limit
#[tauri::command]
async fn http_set_bandwidth(device_id: String, bytes_per_sec: u64) -> Result<(), String> {
    let url = format!("{}/devices/{}/bandwidth", DAEMON_URL, device_id);
    let client = reqwest::Client::new();
    
    #[derive(Serialize)]
    struct BandwidthBody {
        bytes_per_sec: u64,
    }
    
    let body = BandwidthBody { bytes_per_sec };
    
    let response = client
        .put(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Bandwidth update failed: {}", response.status()))
    }
}

/// GET /stats - Get statistics
#[tauri::command]
async fn http_get_stats() -> Result<serde_json::Value, String> {
    let url = format!("{}/stats", DAEMON_URL);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse stats response: {}", e))
}

// ============ Main Application ============

#[cfg_attr(mobile, tauri::mobile::app_entry)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            http_get_qr,
            http_get_devices,
            http_approve_device,
            http_deny_device,
            http_set_bandwidth,
            http_get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(mobile))]
fn main() {
    run();
}
