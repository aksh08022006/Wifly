use crate::CertBundle;
/// Device Consent & Enrollment Server
/// ====================================
/// Runs a TLS server that handles device enrollment
use std::io::Cursor;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_rustls::{rustls, TlsAcceptor};

/// Tracks enrolled device IPs
pub struct EnrolledDevices {
    ips: Vec<std::net::Ipv4Addr>,
}

impl EnrolledDevices {
    pub fn new() -> Self {
        Self { ips: Vec::new() }
    }

    pub fn add(&mut self, ip: std::net::Ipv4Addr) {
        if !self.ips.contains(&ip) {
            self.ips.push(ip);
        }
    }

    pub fn is_enrolled(&self, ip: std::net::Ipv4Addr) -> bool {
        self.ips.contains(&ip)
    }
}

impl Default for EnrolledDevices {
    fn default() -> Self {
        Self::new()
    }
}

/// Setup TLS configuration from certificate
async fn setup_tls_config(
    cert: &CertBundle,
) -> Result<Arc<rustls::ServerConfig>, Box<dyn std::error::Error>> {
    use rustls::pki_types::PrivateKeyDer;

    // Parse certificate from PEM - certs() returns an Iterator over Results
    let mut cert_reader = Cursor::new(cert.cert_pem.as_bytes());
    let cert_list: Result<Vec<_>, _> = rustls_pemfile::certs(&mut cert_reader).collect();

    let cert_der: Vec<_> = cert_list?;

    if cert_der.is_empty() {
        return Err("No certificates found".into());
    }

    // Parse private key from PEM
    let mut key_reader = Cursor::new(cert.key_pem.as_bytes());
    let key_list: Result<Vec<_>, _> = rustls_pemfile::pkcs8_private_keys(&mut key_reader).collect();

    let mut keys = key_list?;
    if keys.is_empty() {
        return Err("No private key found".into());
    }

    let key_bytes = keys.remove(0);
    let key = PrivateKeyDer::Pkcs8(key_bytes);

    // Create server config
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, key)?;

    Ok(Arc::new(config))
}

/// Run the device enrollment server
/// Listens on 0.0.0.0:7979 with TLS
pub async fn run_consent_server(
    cert: CertBundle,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup TLS
    let config = setup_tls_config(&cert).await?;
    let acceptor = TlsAcceptor::from(config);

    // Create TCP listener
    let listener = TcpListener::bind("0.0.0.0:7979").await?;
    tracing::info!("Device enrollment server listening on 0.0.0.0:7979");

    // Accept connections
    loop {
        let (socket, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let enrolled = enrolled.clone();

        tracing::debug!("Device connection: {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_enrollment_connection(socket, acceptor, enrolled).await {
                tracing::warn!("Enrollment error from {}: {}", addr, e);
            }
        });
    }
}

/// Handle individual enrollment connection
async fn handle_enrollment_connection(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TLS handshake
    let mut tls_stream = acceptor.accept(socket).await?;
    let peer_addr = tls_stream.get_ref().0.peer_addr()?;
    let peer_ip = peer_addr.ip();

    tracing::info!("Device connected: {}", peer_ip);

    // Extract IPv4 address
    let peer_ipv4 = match peer_ip {
        IpAddr::V4(v4) => v4,
        IpAddr::V6(_) => {
            tracing::warn!("IPv6 device not supported: {}", peer_ip);
            return Ok(());
        }
    };

    // Create HTML form
    let html = create_enrollment_html(peer_ipv4);

    // Send HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );

    tls_stream.write_all(response.as_bytes()).await?;
    tls_stream.flush().await?;

    // Read request
    let mut buffer = vec![0u8; 4096];
    let n = tls_stream.read(&mut buffer).await?;

    if n == 0 {
        tracing::warn!("Device disconnected without enrollment");
        return Ok(());
    }

    // Parse POST data
    let request_str = String::from_utf8_lossy(&buffer[..n]);
    let approved = request_str.contains("action=allow");

    // Update enrollment status
    {
        let mut devs = enrolled.lock().await;
        if approved {
            devs.add(peer_ipv4);
            tracing::info!("Device approved: {}", peer_ipv4);
        } else {
            tracing::info!("Device denied: {}", peer_ipv4);
        }
    }

    // Send confirmation response
    let confirmation = if approved {
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 9\r\n\r\nApproved!"
    } else {
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 7\r\n\r\nDenied!"
    };

    tls_stream.write_all(confirmation.as_bytes()).await?;
    tls_stream.shutdown().await?;

    Ok(())
}

/// Generate HTML enrollment form
fn create_enrollment_html(peer_ip: std::net::Ipv4Addr) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>NetShaper Device Enrollment</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 600px; margin: 50px auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; margin-top: 0; }}
        p {{ color: #666; line-height: 1.6; }}
        .device-ip {{ font-family: monospace; background: #f0f0f0; padding: 8px 12px; border-radius: 4px; display: inline-block; }}
        .buttons {{ margin-top: 30px; display: flex; gap: 10px; }}
        button {{ padding: 12px 24px; font-size: 16px; border: none; border-radius: 4px; cursor: pointer; transition: background 0.3s; }}
        .approve {{ background-color: #4CAF50; color: white; }}
        .approve:hover {{ background-color: #45a049; }}
        .deny {{ background-color: #f44336; color: white; }}
        .deny:hover {{ background-color: #da190b; }}
        .info {{ background: #e3f2fd; border-left: 4px solid #2196F3; padding: 12px; margin: 20px 0; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 NetShaper Device Enrollment</h1>
        <div class="info">
            <p>A new device is trying to connect to your network.</p>
        </div>
        <p>
            <strong>Device IP:</strong><br>
            <span class="device-ip">{}</span>
        </p>
        <p>Would you like to allow this device to use your network bandwidth manager?</p>
        <form method="post" action="/enroll" class="buttons">
            <button type="submit" name="action" value="allow" class="approve">✓ Allow</button>
            <button type="submit" name="action" value="deny" class="deny">✗ Deny</button>
        </form>
        <p style="color: #999; font-size: 12px; margin-top: 30px;">
            NetShaper Device Enrollment | This connection is encrypted with TLS
        </p>
    </div>
</body>
</html>"#,
        peer_ip
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrolled_devices_tracking() {
        let mut enrolled = EnrolledDevices::new();
        let ip: std::net::Ipv4Addr = "192.168.1.100".parse().unwrap();

        assert!(!enrolled.is_enrolled(ip));
        enrolled.add(ip);
        assert!(enrolled.is_enrolled(ip));
    }

    #[test]
    fn test_html_generation() {
        let ip: std::net::Ipv4Addr = "192.168.1.50".parse().unwrap();
        let html = create_enrollment_html(ip);

        assert!(html.contains("NetShaper Device Enrollment"));
        assert!(html.contains("192.168.1.50"));
        assert!(html.contains("name=\"action\""));
        assert!(html.contains("value=\"allow\""));
        assert!(html.contains("value=\"deny\""));
    }

    #[test]
    fn test_enrolled_devices_list() {
        let mut enrolled = EnrolledDevices::new();
        let ip1: std::net::Ipv4Addr = "192.168.1.100".parse().unwrap();
        let ip2: std::net::Ipv4Addr = "192.168.1.101".parse().unwrap();

        enrolled.add(ip1);
        enrolled.add(ip2);
        assert_eq!(enrolled.ips.len(), 2);

        // Adding duplicate should not increase count
        enrolled.add(ip1);
        assert_eq!(enrolled.ips.len(), 2);
    }
}
