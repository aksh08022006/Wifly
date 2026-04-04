mod registry;
mod http;
mod wfp_bridge;
mod stats_listener;
mod token_bucket;
mod pipe_server;

use axum::{
    routing::{post, put},
    Router,
};
use std::sync::Arc;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use tokio::sync::{Mutex, broadcast, mpsc};
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use registry::{DeviceRegistry, QrCodeData, BandwidthCommand, DashboardEvent};
use token_bucket::TokenBucket;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Mutex<DeviceRegistry>>,
    pub qr_data: Arc<Mutex<Option<QrCodeData>>>,
    pub event_tx: broadcast::Sender<DashboardEvent>,
    pub bandwidth_tx: mpsc::Sender<BandwidthCommand>,
    pub token_buckets: Arc<Mutex<HashMap<Ipv4Addr, TokenBucket>>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Detect server IP - check environment variable first
    let server_ip = std::env::var("NETSHAPER_SERVER_IP")
        .unwrap_or_else(|_| get_server_ip());
    
    tracing::info!("Detected server IP: {}", server_ip);
    if std::env::var("NETSHAPER_SERVER_IP").is_err() {
        tracing::info!(r#"To override IP, set: $env:NETSHAPER_SERVER_IP = '192.168.x.x'; .\daemon.exe"#);
    }

    // Create state
    let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
    
    // Generate QR code pairing token
    let qr_data = QrCodeData {
        token: Uuid::new_v4().to_string(),
        created_at: Utc::now(),
    };
    
    let qr_data_mutex = Arc::new(Mutex::new(Some(qr_data.clone())));
    let (event_tx, _) = broadcast::channel(100);
    let (bandwidth_tx, mut bandwidth_rx) = mpsc::channel(100);
    let token_buckets = Arc::new(Mutex::new(HashMap::<Ipv4Addr, TokenBucket>::new()));

    let state = Arc::new(AppState {
        registry,
        qr_data: qr_data_mutex.clone(),
        event_tx,
        bandwidth_tx,
        token_buckets: token_buckets.clone(),
    });

    // Spawn named pipe server (talks to WFP kernel driver)
    tokio::spawn({
        let buckets = token_buckets.clone();
        async move {
            pipe_server::run_pipe_server(buckets).await;
        }
    });

    // Spawn bandwidth command handler
    tokio::spawn(async move {
        while let Some(cmd) = bandwidth_rx.recv().await {
            match cmd {
                BandwidthCommand::Update { ip, bytes_per_sec } => {
                    tracing::info!("Bandwidth update for {}: {} bytes/sec", ip, bytes_per_sec);
                    // Bandwidth is managed directly through token_buckets now
                }
                BandwidthCommand::Block { ip } => {
                    tracing::info!("Blocking device: {}", ip);
                }
                BandwidthCommand::Unblock { ip } => {
                    tracing::info!("Unblocking device: {}", ip);
                }
            }
        }
    });

    // Build router
    let app = Router::new()
        .route("/", axum::routing::get(http::root_handler))
        .route("/health", axum::routing::get(http::health_check))
        .route("/qr", axum::routing::get(http::get_qr))
        .route("/pair", axum::routing::get(http::get_pair_page).post(http::post_pair))
        .route("/devices", axum::routing::get(http::get_devices))
        .route("/devices/:id/approve", post(http::approve_device))
        .route("/devices/:id/deny", post(http::deny_device))
        .route("/devices/:id/bandwidth", put(http::set_bandwidth))
        .route("/api/devices/add-by-ip", post(http::add_device_by_ip))
        .route("/stats", axum::routing::get(http::get_stats))
        .route("/events", axum::routing::get(http::event_stream))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .into_make_service();

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to 0.0.0.0:8080");

    tracing::info!("NetShaper daemon listening on http://0.0.0.0:8080");
    tracing::info!("Pairing URL: http://{}:8080/pair?token={}", server_ip, qr_data.token);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

fn get_server_ip() -> String {
    use local_ip_address::local_ip;
    use std::process::Command;
    
    // Try to get LAN IP from ipconfig first
    if let Ok(ip) = get_lan_ip_from_ipconfig() {
        tracing::info!("Detected LAN IP from ipconfig: {}", ip);
        return ip;
    }
    
    // Fallback to local_ip_address crate
    if let Ok(ip) = local_ip() {
        let ip_str = ip.to_string();
        tracing::info!("Fallback IP from local_ip_address: {}", ip_str);
        return ip_str;
    }
    
    "127.0.0.1".to_string()
}

fn get_lan_ip_from_ipconfig() -> Result<String, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let output = Command::new("ipconfig")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Debug: print all lines that mention IPv4
    let mut found_ips = Vec::new();
    
    // Look for IPv4 addresses in the format "IPv4 Address . . . : 192.168.x.x"
    for line in stdout.lines() {
        if line.contains("IPv4 Address") && !line.contains("127.0.0.1") {
            // Extract the IP address
            if let Some(ip_part) = line.split(':').last() {
                let ip = ip_part.trim();
                found_ips.push(ip.to_string());
                
                // Prefer 192.168.x.x or 10.x.x.x IPs
                if ip.starts_with("192.168.") || ip.starts_with("10.") {
                    return Ok(ip.to_string());
                }
            }
        }
    }
    
    // If we didn't find a preferred IP, use the first one we found
    if let Some(ip) = found_ips.first() {
        return Ok(ip.clone());
    }
    
    Err("No suitable LAN IP found in ipconfig output".into())
}
