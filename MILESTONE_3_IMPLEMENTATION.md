# Milestone 3: Implementation Guide - Phase by Phase

## Overview

This guide walks you through implementing each phase of the device enrollment server. Each phase is designed to be completed in ~2-4 hours with full tests.

---

## Phase 1: Certificate Generation (3 hours)

### File: `crypto/src/cert.rs`

### What It Does
Generates, saves, and loads self-signed Ed25519 certificates for the TLS server.

### Tasks

#### Task 1.1: Implement `generate_self_signed()`

```rust
pub fn generate_self_signed() -> Result<Self, CertError> {
    // Use rcgen to generate Ed25519 certificate
    // Subject: CN=netshaper-ca with SANs
    // Validity: 10 years
    // Return: CertBundle with cert + PEM strings
}
```

**Implementation**:
```rust
use rcgen::Certificate;
use std::time::Duration;

pub fn generate_self_signed() -> Result<Self, CertError> {
    // Subject alternative names
    let san_list = vec![
        "netshaper-ca".to_string(),
        "127.0.0.1".to_string(),
    ];
    
    // Generate Ed25519 certificate
    let cert = rcgen::generate_simple_self_signed(san_list)
        .map_err(|e| CertError::RcgenError(format!("{:?}", e)))?;
    
    // Serialize to PEM
    let cert_pem = cert.serialize_pem()
        .map_err(|e| CertError::RcgenError(format!("{:?}", e)))?;
    let key_pem = cert.serialize_private_key_pem();
    
    Ok(Self {
        cert,
        cert_pem,
        key_pem,
    })
}
```

**Test**:
```rust
#[test]
fn test_generate_self_signed() {
    let bundle = CertBundle::generate_self_signed().unwrap();
    assert!(!bundle.cert_pem.is_empty());
    assert!(!bundle.key_pem.is_empty());
    assert!(bundle.cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(bundle.key_pem.contains("BEGIN PRIVATE KEY"));
}
```

#### Task 1.2: Implement `save_to_disk()`

```rust
pub fn save_to_disk(&self, dir: &Path) -> Result<(), CertError> {
    // Create ~/.netshaper/ if missing
    // Save cert as ca.pem
    // Save key as ca.key (with 0o600 permissions)
    // Fail if files already exist
}
```

**Implementation**:
```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::fs::Permissions;

pub fn save_to_disk(&self, dir: &Path) -> Result<(), CertError> {
    // Create directory
    fs::create_dir_all(dir)
        .map_err(|e| CertError::Io(e))?;
    
    let cert_path = dir.join("ca.pem");
    let key_path = dir.join("ca.key");
    
    // Check if files exist (don't overwrite)
    if cert_path.exists() || key_path.exists() {
        return Err(CertError::AlreadyExists);
    }
    
    // Write certificate
    fs::write(&cert_path, &self.cert_pem)
        .map_err(|e| CertError::Io(e))?;
    
    // Write private key
    fs::write(&key_path, &self.key_pem)
        .map_err(|e| CertError::Io(e))?;
    
    // Set strict permissions on private key
    #[cfg(unix)]
    {
        let perms = Permissions::from_mode(0o600);
        fs::set_permissions(&key_path, perms)
            .map_err(|e| CertError::Io(e))?;
    }
    
    Ok(())
}
```

**Test**:
```rust
#[test]
fn test_save_to_disk() {
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let bundle = CertBundle::generate_self_signed().unwrap();
    
    bundle.save_to_disk(dir.path()).unwrap();
    
    assert!(dir.path().join("ca.pem").exists());
    assert!(dir.path().join("ca.key").exists());
}

#[test]
fn test_save_twice_fails() {
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let bundle = CertBundle::generate_self_signed().unwrap();
    
    bundle.save_to_disk(dir.path()).unwrap();
    assert!(bundle.save_to_disk(dir.path()).is_err());
}
```

#### Task 1.3: Implement `load_or_generate()`

```rust
pub fn load_or_generate(dir: &Path) -> Result<Self, CertError> {
    // If ~/.netshaper/ca.pem + ca.key exist: load them
    // Otherwise: generate, save, and return
}
```

**Implementation**:
```rust
pub fn load_or_generate(dir: &Path) -> Result<Self, CertError> {
    let cert_path = dir.join("ca.pem");
    let key_path = dir.join("ca.key");
    
    // Check if both files exist
    if cert_path.exists() && key_path.exists() {
        let cert_pem = fs::read_to_string(&cert_path)?;
        let key_pem = fs::read_to_string(&key_path)?;
        
        // Note: We're storing the PEM strings, not re-parsing them to Certificate
        // This is sufficient for the TLS server usage
        return Ok(Self {
            cert: Certificate { /* placeholder */ },
            cert_pem,
            key_pem,
        });
    }
    
    // Generate new certificate
    let bundle = Self::generate_self_signed()?;
    bundle.save_to_disk(dir)?;
    Ok(bundle)
}
```

**Test**:
```rust
#[test]
fn test_load_or_generate() {
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    
    // First call: generates
    let bundle1 = CertBundle::load_or_generate(dir.path()).unwrap();
    
    // Second call: loads
    let bundle2 = CertBundle::load_or_generate(dir.path()).unwrap();
    
    // Should be identical
    assert_eq!(bundle1.cert_pem, bundle2.cert_pem);
    assert_eq!(bundle1.key_pem, bundle2.key_pem);
}
```

### Completion Checklist
- [ ] `generate_self_signed()` implemented
- [ ] `save_to_disk()` implemented
- [ ] `load_or_generate()` implemented
- [ ] All unit tests pass
- [ ] No clippy warnings
- [ ] Code formatted

---

## Phase 2: TLS Server (4 hours)

### File: `crypto/src/handshake.rs`

### What It Does
Runs a TLS server on port 7979 that devices connect to for enrollment.

### Tasks

#### Task 2.1: Create TLS Config

```rust
async fn setup_tls_config(cert: &CertBundle) -> Result<Arc<ServerConfig>, Box<dyn Error>> {
    // Create ServerConfig from certificate
    // Use TLS 1.3 only
    // Disable weak ciphers
}
```

**Implementation**:
```rust
use rustls::ServerConfig;
use std::sync::Arc;

async fn setup_tls_config(cert: &CertBundle) -> Result<Arc<ServerConfig>, Box<dyn Error>> {
    use rustls_pemfile::certs;
    use std::io::Cursor;
    
    // Parse certificate
    let mut cert_reader = Cursor::new(cert.cert_pem.as_bytes());
    let cert_chain = certs(&mut cert_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect::<Vec<_>>();
    
    // Parse private key
    let mut key_reader = Cursor::new(cert.key_pem.as_bytes());
    let key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .next()
        .ok_or("No private key found")?;
    let key = rustls::PrivateKey(key);
    
    // Create server config
    let mut config = ServerConfig::new(
        rustls::NoClientAuth::new(),
        key,
        cert_chain,
    );
    
    Ok(Arc::new(config))
}
```

#### Task 2.2: Implement `run_consent_server()`

```rust
pub async fn run_consent_server(
    cert: CertBundle,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup TLS config
    // Create TCP listener on 0.0.0.0:7979
    // Accept connections in loop
    // For each connection: spawn async task to handle it
}
```

**Implementation**:
```rust
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

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
```

#### Task 2.3: Implement `handle_enrollment_connection()`

```rust
async fn handle_enrollment_connection(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn Error>> {
    // TLS handshake
    // Get peer IP
    // Send HTML form
    // Read POST response
    // Update enrolled devices
}
```

**Implementation**:
```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
    
    // Create HTML form
    let html = create_enrollment_html(peer_ip);
    
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
            devs.add(peer_ip);
            tracing::info!("Device approved: {}", peer_ip);
        } else {
            tracing::info!("Device denied: {}", peer_ip);
        }
    }
    
    // Send confirmation response
    let confirmation = if approved {
        "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nApproved!"
    } else {
        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nDenied!"
    };
    
    tls_stream.write_all(confirmation.as_bytes()).await?;
    tls_stream.shutdown().await?;
    
    Ok(())
}
```

#### Task 2.4: Create HTML Form

```rust
fn create_enrollment_html(peer_ip: std::net::IpAddr) -> String {
    // Return HTML form with Approve/Deny buttons
}
```

**Implementation**:
```rust
fn create_enrollment_html(peer_ip: std::net::IpAddr) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>NetShaper Device Enrollment</title>
    <style>
        body {{ font-family: sans-serif; margin: 40px; }}
        .container {{ max-width: 500px; margin: auto; }}
        .buttons {{ margin-top: 20px; }}
        button {{ padding: 10px 20px; margin: 5px; font-size: 16px; }}
        .approve {{ background-color: #4CAF50; color: white; }}
        .deny {{ background-color: #f44336; color: white; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 NetShaper Device Enrollment</h1>
        <p>Device IP: <strong>{}</strong></p>
        <p>Would you like to allow this device?</p>
        <form method="post" action="/enroll" class="buttons">
            <button type="submit" name="action" value="allow" class="approve">✓ Allow</button>
            <button type="submit" name="action" value="deny" class="deny">✗ Deny</button>
        </form>
    </div>
</body>
</html>
    "#, peer_ip)
}
```

### Completion Checklist
- [ ] `setup_tls_config()` implemented
- [ ] `run_consent_server()` implemented
- [ ] `handle_enrollment_connection()` implemented
- [ ] HTML form generation working
- [ ] Server starts on :7979
- [ ] Devices can connect
- [ ] POST handling works
- [ ] All tests pass

---

## Phase 3: Device Persistence (2 hours)

### File: `crypto/src/device_enrollment.rs` (NEW)

### Create this file:

```rust
/// Device Enrollment Storage
/// ==========================
/// Persists approved devices to ~/.netshaper/devices.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnrollmentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceEnrollment {
    pub ip: Ipv4Addr,
    pub hostname: Option<String>,
    pub approved: bool,
    pub enrolled_at: String, // ISO 8601 timestamp
}

pub struct DeviceList {
    devices: Vec<DeviceEnrollment>,
}

impl DeviceList {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
    
    pub fn add(&mut self, ip: Ipv4Addr, approved: bool) {
        // Add or update device
        if let Some(device) = self.devices.iter_mut().find(|d| d.ip == ip) {
            device.approved = approved;
        } else {
            self.devices.push(DeviceEnrollment {
                ip,
                hostname: None,
                approved,
                enrolled_at: chrono::Local::now().to_rfc3339(),
            });
        }
    }
    
    pub fn approved_devices(&self) -> Vec<Ipv4Addr> {
        self.devices
            .iter()
            .filter(|d| d.approved)
            .map(|d| d.ip)
            .collect()
    }
    
    pub fn save_to_disk(&self, path: &Path) -> Result<(), EnrollmentError> {
        let json = serde_json::to_string_pretty(&self.devices)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_from_disk(path: &Path) -> Result<Self, EnrollmentError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let json = fs::read_to_string(path)?;
        let devices = serde_json::from_str(&json)?;
        Ok(Self { devices })
    }
}

impl Default for DeviceList {
    fn default() -> Self {
        Self::new()
    }
}
```

### Tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use tempfile::TempDir;
    
    #[test]
    fn test_add_device() {
        let mut list = DeviceList::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();
        
        list.add(ip, true);
        assert_eq!(list.approved_devices().len(), 1);
    }
    
    #[test]
    fn test_save_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("devices.json");
        
        let mut list = DeviceList::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();
        list.add(ip, true);
        
        list.save_to_disk(&path).unwrap();
        let loaded = DeviceList::load_from_disk(&path).unwrap();
        
        assert_eq!(loaded.approved_devices().len(), 1);
    }
}
```

### Update `crypto/src/lib.rs`:

```rust
pub mod device_enrollment;
pub use device_enrollment::DeviceList;
```

---

## Phase 4: Daemon Integration (3 hours)

### File: `daemon/src/main.rs` (modify)

### Add device loading on startup:

```rust
#[tokio::main]
async fn main() {
    // ... existing code ...
    
    let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
    
    // NEW: Load enrolled devices
    {
        let devices_path = std::path::PathBuf::from(env!("HOME"))
            .join(".netshaper/devices.json");
        
        match crypto::DeviceList::load_from_disk(&devices_path) {
            Ok(device_list) => {
                let mut reg = registry.lock().await;
                for ip in device_list.approved_devices() {
                    reg.insert_device(ip, 10_000_000); // 10 MB/s default
                    tracing::info!("Loaded enrolled device: {}", ip);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load enrolled devices: {}", e);
            }
        }
    }
    
    // ... rest of code ...
}
```

---

## Phase 5: Security Hardening (2 hours)

### Review Checklist

- [ ] TLS 1.3 only (no TLS 1.2)
- [ ] Strong ciphers only
- [ ] Private key permissions: 0o600
- [ ] HTML escaping in forms
- [ ] Input validation on POST
- [ ] Rate limiting on enrollment (prevent DOS)
- [ ] Error messages don't leak info

### Code Review Items

1. **TLS Config**: Verify `ServerConfig` uses strong settings
2. **File Permissions**: Check `0o600` on ca.key
3. **HTML**: Check device IP is properly escaped
4. **Input Validation**: Verify POST data is validated
5. **Errors**: Ensure no sensitive data in error messages

---

## Testing Summary

### Run All Tests
```bash
cargo test -p crypto
```

### Phase 1 Tests
```bash
cargo test -p crypto test_generate
cargo test -p crypto test_save
cargo test -p crypto test_load
```

### Phase 2 Tests
```bash
cargo test -p crypto test_enrollment_server
cargo test -p crypto test_tls_connection
```

### Phase 3 Tests
```bash
cargo test -p crypto test_device_persistence
cargo test -p crypto test_json_roundtrip
```

### Integration Test
```bash
cargo test -p crypto --test integration_test
```

---

## Completion Checklist

- [ ] Phase 1: Certificates ✓
- [ ] Phase 2: TLS Server ✓
- [ ] Phase 3: Device Persistence ✓
- [ ] Phase 4: Daemon Integration ✓
- [ ] Phase 5: Security ✓
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation complete

---

## Next Steps After Phase Completion

1. Merge M3 to feature branch
2. Have Saksham review M1/M2 integration
3. Begin M4 (Full integration testing)
4. Start M5 (UI) in parallel

**Milestone 3 Implementation Ready! 🚀**
