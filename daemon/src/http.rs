/// HTTP Server for Device Pairing and Management
/// ===============================================
/// Provides REST API for:
/// - Device pairing requests with QR code
/// - Device approval/denial from ONCE browser page
/// - Real-time SSE event stream for dashboard
/// - List devices and bandwidth control

use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use crate::registry::{DashboardEvent, BandwidthCommand};
use crate::AppState;
use base64::{Engine as _, engine::general_purpose};
use std::net::Ipv4Addr;
use tracing::info;

// ============ Request/Response Types ============

/// Pair request from phone POST /pair
#[derive(Debug, Deserialize)]
pub struct PairRequest {
    pub token: String,
    pub platform: String,
    pub device_name: Option<String>,
    #[serde(default)]
    pub device_ip: Option<String>, // Phone can optionally send its IP
}

/// Pair response - minimal
#[derive(Debug, Serialize)]
pub struct PairResponse {
    pub status: String,
    pub message: String,
}

/// Approve device request
#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub bandwidth_limit_mb: u64,
}

/// Set bandwidth request
#[derive(Debug, Deserialize)]
pub struct SetBandwidthRequest {
    pub bytes_per_sec: u64,
}

/// QR response
#[derive(Debug, Serialize)]
pub struct QrResponse {
    pub qr_image: String,
    pub pairing_url: String,
}

// ============ HTTP Handlers ============

/// GET / - Root page with daemon info
pub async fn root_handler() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>NetShaper Daemon</title>
  <style>
    body { font-family: sans-serif; margin: 40px; background: #0f172a; color: #e2e8f0; }
    h1 { color: #3b82f6; }
    .endpoint { background: #1e293b; padding: 15px; margin: 10px 0; border-radius: 8px; border-left: 4px solid #3b82f6; }
    code { background: #475569; padding: 2px 6px; border-radius: 4px; }
  </style>
</head>
<body>
  <h1>🔒 NetShaper Daemon</h1>
  <p>The daemon is running and ready to pair devices!</p>
  
  <h2>Available Endpoints:</h2>
  <div class="endpoint">
    <strong>GET /qr</strong> - Get QR code image for pairing
  </div>
  <div class="endpoint">
    <strong>GET /pair?token=XXX</strong> - Pairing page (scan QR to open)
  </div>
  <div class="endpoint">
    <strong>GET /health</strong> - Health check
  </div>
  <div class="endpoint">
    <strong>GET /devices</strong> - List all paired devices
  </div>
  <div class="endpoint">
    <strong>GET /stats</strong> - Real-time bandwidth statistics
  </div>
  <div class="endpoint">
    <strong>GET /events</strong> - SSE event stream for dashboard
  </div>
</body>
</html>"#;
    
    (axum::http::StatusCode::OK, 
     axum::response::Html(html)).into_response()
}

/// GET /health - Health check
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

/// GET /check-ip - Return client's detected IP (for pairing page to auto-fill)
/// Returns the X-Forwarded-For header if present, otherwise tries to detect from connection
pub async fn check_ip() -> impl IntoResponse {
    // This is a helper endpoint for the pairing page
    // Browser-based IP detection via WebRTC is preferred
    Json(serde_json::json!({
        "note": "Use WebRTC-based detection in browser"
    }))
}

/// GET /qr - Get QR code image + pairing URL
pub async fn get_qr(
    State(state): State<std::sync::Arc<AppState>>,
) -> impl IntoResponse {
    let qr_data = state.qr_data.lock().await;
    
    if let Some(qr) = qr_data.as_ref() {
        // Generate QR code that encodes ONLY the URL
        let pairing_url = format!(
            "http://{}:8080/pair?token={}",
            qr.token, qr.token
        );
        
        match generate_qr_svg(&pairing_url) {
            Ok(qr_image) => {
                return Json(QrResponse {
                    qr_image,
                    pairing_url,
                }).into_response();
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("QR generation failed: {}", e)}))
                ).into_response();
            }
        }
    }
    
    (StatusCode::INTERNAL_SERVER_ERROR,
     Json(serde_json::json!({"error": "QR data not initialized"}))
    ).into_response()
}

/// GET /pair?token=... - Phone opens this in browser (ONCE)
pub async fn get_pair_page(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(t) => t.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing token parameter").into_response();
        }
    };
    
    // Verify token exists in QR data
    let qr_data = state.qr_data.lock().await;
    if !qr_data.as_ref().map(|q| q.token == token).unwrap_or(false) {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }
    drop(qr_data);
    
    // Return minimal HTML page for phone browser
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>NetShaper</title>
  <style>
    body {
      margin: 0;
      padding: 0;
      background: #0f172a;
      color: #e2e8f0;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
    }
    .container {
      text-align: center;
      padding: 30px;
      max-width: 400px;
    }
    h1 { font-size: 24px; margin: 0 0 20px 0; }
    .device-info {
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 8px;
      padding: 20px;
      margin: 20px 0;
      text-align: left;
      font-size: 14px;
    }
    .info-line {
      margin: 8px 0;
      display: flex;
      justify-content: space-between;
    }
    .info-label { color: #94a3b8; font-weight: 500; }
    .info-value { color: #e2e8f0; }
    button {
      background: #3b82f6;
      color: white;
      border: none;
      padding: 16px 32px;
      border-radius: 6px;
      font-size: 16px;
      font-weight: 500;
      cursor: pointer;
      width: 100%;
      margin-top: 20px;
    }
    button:hover { background: #2563eb; }
    .success {
      color: #10b981;
      margin-top: 20px;
      font-size: 18px;
    }
    .spinner {
      display: inline-block;
      width: 20px;
      height: 20px;
      border: 3px solid #334155;
      border-top-color: #3b82f6;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
    }
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔒 NetShaper</h1>
    <p>Waiting for your permission...</p>
    
    <div class="device-info">
      <div class="info-line">
        <span class="info-label">Your IP:</span>
        <span class="info-value" id="device-ip">Detecting...</span>
      </div>
      <div class="info-line">
        <span class="info-label">Device:</span>
        <span class="info-value" id="device-type">Mobile</span>
      </div>
      <div class="info-line" style="margin-top: 15px; flex-direction: column; align-items: flex-start;">
        <span class="info-label" style="margin-bottom: 8px;">Enter IP manually:</span>
        <input type="text" id="ip-input" placeholder="e.g., 192.168.1.50" 
               style="width: 100%; padding: 8px; border: 1px solid #475569; background: #0f172a; 
                      color: #e2e8f0; border-radius: 4px; font-size: 14px;">
      </div>
    </div>

    <div id="pending" style="display: block;">
      <p>Once you tap "Allow", check your laptop for approval.</p>
      <button onclick="pair()">Allow NetShaper to Manage Connection</button>
      <div id="error" style="color: #ef4444; margin-top: 10px; display: none;"></div>
    </div>

    <div id="success" class="success" style="display: none;">
      ✓ Request sent. Check your laptop.
      <br><br>
      <span class="spinner"></span>
      <p style="color: #94a3b8; font-size: 13px; margin-top: 20px;">
        This page will close automatically...
      </p>
    </div>
  </div>

  <script>
    const token = new URLSearchParams(window.location.search).get('token');
    let detectedIP = null;
    
    // Attempt to detect local IP using WebRTC (works on most modern browsers)
    async function detectLocalIP() {
      return new Promise((resolve) => {
        const pc = new RTCPeerConnection({ iceServers: [] });
        const ips = new Set();
        
        pc.createDataChannel('');
        pc.createOffer()
          .then(offer => pc.setLocalDescription(offer))
          .catch(() => resolve(null));
        
        pc.onicecandidate = (ice) => {
          if (!ice || !ice.candidate) {
            pc.close();
            resolve(ips.size > 0 ? Array.from(ips)[0] : null);
            return;
          }
          
          const ipRegex = /([0-9]{1,3}(\.[0-9]{1,3}){3})/;
          const match = ipRegex.exec(ice.candidate.candidate);
          if (match && match[1].startsWith('192.168') || match[1].startsWith('10.')) {
            ips.add(match[1]);
          }
        };
        
        setTimeout(() => {
          pc.close();
          resolve(ips.size > 0 ? Array.from(ips)[0] : null);
        }, 3000);
      });
    }
    
    // Auto-detect IP on load
    detectLocalIP().then(ip => {
      detectedIP = ip;
      if (ip) {
        document.getElementById('device-ip').textContent = ip;
        document.getElementById('ip-input').value = ip;
      } else {
        document.getElementById('device-ip').textContent = 'Not detected - enter below';
      }
    });
    
    async function pair() {
      const pending = document.getElementById('pending');
      const success = document.getElementById('success');
      const errorDiv = document.getElementById('error');
      
      // Use manual input if available, otherwise use detected IP
      const ipInput = document.getElementById('ip-input').value.trim();
      const finalIP = ipInput || detectedIP;
      
      try {
        if (!finalIP) {
          throw new Error('Please enter your device IP address above.');
        }

        const response = await fetch('/pair', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            token: token,
            platform: navigator.userAgent.includes('Android') ? 'android' : 'ios',
            device_name: null,
            device_ip: finalIP
          })
        });

        if (!response.ok) {
          const data = await response.json();
          throw new Error(data.status || 'Pairing failed');
        }

        pending.style.display = 'none';
        success.style.display = 'block';
        
        // Close browser after 5 seconds
        setTimeout(() => window.close(), 5000);
      } catch (err) {
        errorDiv.textContent = err.message;
        errorDiv.style.display = 'block';
      }
    }
  </script>
</body>
</html>"#;

    (StatusCode::OK, axum::response::Html(html)).into_response()
}

/// POST /pair - Phone sends pairing request
pub async fn post_pair(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<PairRequest>,
) -> impl IntoResponse {
    // Validate token
    let qr_data = state.qr_data.lock().await;
    if !qr_data.as_ref().map(|q| q.token == req.token).unwrap_or(false) {
        return (StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"status": "invalid_token"}))
        ).into_response();
    }
    drop(qr_data);

    // Parse device IP from request body if provided
    let device_ip: Ipv4Addr = match req.device_ip.as_ref().and_then(|ip| ip.parse().ok()) {
        Some(ip) => ip,
        None => {
            // If no IP provided, use a default LAN IP
            // In production, middleware could extract the peer address
            // For now, default to 192.168.x.x range or generate a placeholder
            match "192.168.1.100".parse::<Ipv4Addr>() {
                Ok(ip) => ip,
                Err(_) => {
                    return (StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"status": "could_not_determine_ip"}))
                    ).into_response();
                }
            }
        }
    };

    let device_name = req.device_name.unwrap_or_else(|| "Wireless Device".to_string());

    // Add as pending device
    let mut registry = state.registry.lock().await;
    let device = registry.add_pending_device(device_ip, req.platform.clone(), device_name.clone());
    drop(registry);

    // Broadcast SSE event to dashboard
    let event = DashboardEvent::DevicePending { device: device.clone() };
    let _ = state.event_tx.send(event);

    info!("Device pairing request: {} ({}) from IP {}", device_name, req.platform, device_ip);

    (StatusCode::OK,
     Json(serde_json::json!({
        "status": "pending_approval",
        "message": "Check laptop for approval"
     }))
    ).into_response()
}

/// GET /devices - List all devices
pub async fn get_devices(
    State(state): State<std::sync::Arc<AppState>>,
) -> impl IntoResponse {
    let registry = state.registry.lock().await;
    let devices = registry.get_all();
    
    Json(serde_json::json!({
        "devices": devices,
        "total": devices.len(),
        "approved": devices.iter().filter(|d| d.approved).count(),
        "pending": devices.iter().filter(|d| !d.approved).count(),
    }))
}

/// POST /devices/:id/approve - Approve device
pub async fn approve_device(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ApproveRequest>,
) -> impl IntoResponse {
    let bandwidth_bytes = req.bandwidth_limit_mb * 1_000_000;
    
    let mut registry = state.registry.lock().await;
    if let Some(device) = registry.approve_device(&id, bandwidth_bytes) {
        drop(registry);
        
        // Update token bucket for this device (async, non-blocking)
        let device_ip = device.ip;
        let device_name = device.device_name.clone();
        let buckets = state.token_buckets.clone();
        
        tokio::spawn(async move {
            crate::wfp_bridge::set_bandwidth(buckets, device_ip, bandwidth_bytes).await;
            info!("Approved device {} with limit {} bytes/sec", device_name, bandwidth_bytes);
        });
        
        // Broadcast SSE event
        let event = DashboardEvent::DeviceApproved {
            device_id: id.clone(),
            device: device.clone(),
        };
        let _ = state.event_tx.send(event);
        
        info!("Device approved: {}", device.device_name);
        return Json(serde_json::json!({"status": "approved", "device": device})).into_response();
    }
    
    (StatusCode::NOT_FOUND,
     Json(serde_json::json!({"status": "device_not_found"}))
    ).into_response()
}

/// POST /devices/:id/deny - Deny device
pub async fn deny_device(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut registry = state.registry.lock().await;
    if let Some(device) = registry.get_by_id(&id) {
        let device_ip = device.ip;
        let device_name = device.device_name.clone();
        drop(registry);
        
        // Block the device in token bucket
        let buckets = state.token_buckets.clone();
        tokio::spawn(async move {
            crate::wfp_bridge::block_device(buckets, device_ip).await;
            info!("Blocked device {}", device_name);
        });
        
        // Now update registry
        let mut registry = state.registry.lock().await;
        registry.deny_device(&id);
        
        // Broadcast SSE event
        let event = DashboardEvent::DeviceDenied { device_id: id.clone() };
        let _ = state.event_tx.send(event);
        
        info!("Device denied: {}", id);
        return Json(serde_json::json!({"status": "denied"})).into_response();
    }
    
    (StatusCode::NOT_FOUND,
     Json(serde_json::json!({"status": "device_not_found"}))
    ).into_response()
}

/// PUT /devices/:id/bandwidth - Update bandwidth for device
pub async fn set_bandwidth(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SetBandwidthRequest>,
) -> impl IntoResponse {
    let mut registry = state.registry.lock().await;
    if let Some(device) = registry.update_bandwidth(&id, req.bytes_per_sec) {
        drop(registry);
        
        // Update token bucket for this device (async, non-blocking)
        let device_ip = device.ip;
        let bytes_per_sec = req.bytes_per_sec;
        let device_name = device.device_name.clone();
        let buckets = state.token_buckets.clone();
        
        tokio::spawn(async move {
            crate::wfp_bridge::set_bandwidth(buckets, device_ip, bytes_per_sec).await;
            info!("Bandwidth updated for {} to {} bytes/sec", device_name, bytes_per_sec);
        });
        
        info!("Bandwidth updated for {}: {} bytes/sec", device.device_name, req.bytes_per_sec);
        return Json(serde_json::json!({"status": "updated"})).into_response();
    }
    
    (StatusCode::NOT_FOUND,
     Json(serde_json::json!({"status": "device_not_found"}))
    ).into_response()
}

/// POST /api/devices/add-by-ip - Register a device by manual IP input
pub async fn add_device_by_ip(
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Parse IP from payload
    let ip_str = match payload.get("ip").and_then(|v| v.as_str()) {
        Some(ip) => ip,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"status": "invalid_ip"}))
            ).into_response();
        }
    };

    // Parse IP address
    let ip: Ipv4Addr = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"status": "invalid_ip", "message": "Invalid IPv4 address format"}))
            ).into_response();
        }
    };

    // Check if device already exists
    let registry = state.registry.lock().await;
    if let Some(existing) = registry.get_by_ip(ip) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "status": "device_exists",
                "message": format!("Device {} already registered", existing.device_name)
            }))
        ).into_response();
    }
    drop(registry);

    // Add pending device to registry
    let mut registry = state.registry.lock().await;
    let device = registry.add_pending_device(
        ip,
        "Unknown".to_string(),
        ip.to_string(),
    );
    drop(registry);

    // Broadcast SSE event for new pending device
    let event = DashboardEvent::DevicePending { device: device.clone() };
    let _ = state.event_tx.send(event);

    info!("Device registered by IP: {} ({})", ip, device.id);
    
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "registered",
            "device_id": device.id,
            "ip": device.ip,
            "message": "Device registered as pending. Approve from dashboard to enable."
        }))
    ).into_response()
}

/// GET /stats - Stats (placeholder for now)
pub async fn get_stats(
    State(state): State<std::sync::Arc<AppState>>,
) -> impl IntoResponse {
    let registry = state.registry.lock().await;
    let devices = registry.get_all();
    
    Json(serde_json::json!({
        "total_devices": devices.len(),
        "approved_count": devices.iter().filter(|d| d.approved).count(),
        "devices": devices
    }))
}

/// GET /events - SSE stream for real-time updates
pub async fn event_stream(
    State(state): State<std::sync::Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>> {
    let rx = state.event_tx.subscribe();
    
    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                let sse_event = Event::default()
                    .event("update")
                    .data(json);
                Some((Ok(sse_event), rx))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream)
}

// ============ Helper Functions ============

/// Generate QR code as base64 SVG data URL
fn generate_qr_svg(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let qr = qrcode::QrCode::new(text)?;
    let rendered = qr.render::<char>()
        .min_dimensions(200, 200)
        .build();
    
    let lines: Vec<&str> = rendered.lines().collect();
    let width = lines.get(0).map(|l| l.len()).unwrap_or(200);
    let height = lines.len();
    
    let module_size = 10;
    let total_width = width * module_size;
    let total_height = height * module_size;
    
    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        total_width, total_height, total_width, total_height
    ));
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    
    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '█' {
                let x_pos = x * module_size;
                let y_pos = y * module_size;
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="black"/>"#,
                    x_pos, y_pos, module_size, module_size
                ));
            }
        }
    }
    svg.push_str("</svg>");
    
    let encoded = general_purpose::STANDARD.encode(svg.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
}
