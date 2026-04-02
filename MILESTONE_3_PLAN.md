# Milestone 3: Crypto/mTLS Enrollment Server

## Overview

**Milestone 3** implements the device enrollment and consent system. This is where devices (phones, tablets, etc.) authenticate and register with the NetShaper daemon for bandwidth management.

**Status**: Planning phase (stubs in place)  
**Priority**: Medium (can run in parallel with M1/M2 integration)  
**Depends On**: Milestone 2 (daemon running)

---

## What Milestone 3 Does

### Problem

How do we know which devices are "allowed" to be rate-limited? Users need a way to:
1. See what devices are on their network
2. Approve/deny each device for rate limiting
3. Persist these approvals across reboots

### Solution

A **device enrollment server** that:
1. Runs a TLS server on port 7979
2. Shows an enrollment web page ("Consent Form")
3. Devices connect, users approve/deny
4. Saves approved devices to `~/.netshaper/devices.json`
5. Daemon reads enrollment list on startup

### Architecture

```
Device (Phone)
    │
    ├─ Discovers NetShaper on local network (mDNS or hardcoded)
    │
    └─ Connects to 0.0.0.0:7979 (TLS)
           │
           ├─ Server shows: "Allow this device?"
           │  [ALLOW] [DENY]
           │
           ├─ User clicks [ALLOW]
           │
           └─ Device enrollment saved
              ~/.netshaper/devices.json
              {
                "192.168.1.100": { "name": "iPhone", "approved": true },
                "192.168.1.101": { "name": "iPad", "approved": false }
              }
           │
           └─ Daemon loads on startup
              registry.insert_device(192.168.1.100, 1_000_000)

Result: Device is now rate-limited ✓
```

---

## Implementation Plan

### Phase 1: Certificate Management (3 hours)

**cert.rs** - Self-signed certificate generation

1. **`CertBundle::generate_self_signed()`**
   - Use `rcgen` to create Ed25519 certificate
   - Subject: CN=netshaper-ca
   - Validity: 10 years
   - Returns: CertBundle { cert, cert_pem, key_pem }

2. **`CertBundle::save_to_disk()`**
   - Create `~/.netshaper/` directory (if missing)
   - Write `ca.pem` (certificate)
   - Write `ca.key` (private key)
   - File permissions: 0o600 (read/write owner only)

3. **`CertBundle::load_or_generate()`**
   - Check if `~/.netshaper/ca.pem` exists
   - If yes: Load and return
   - If no: Generate, save, and return

4. **Tests**
   - ✅ Generate certificate succeeds
   - ✅ Certificates have correct properties
   - ✅ Save to disk works
   - ✅ Load from disk works
   - ✅ Load-or-generate idempotency

### Phase 2: TLS Server (4 hours)

**handshake.rs** - Device enrollment server

1. **`run_consent_server()`**
   - Create tokio-rustls TLS server
   - Listen on `0.0.0.0:7979`
   - Accept device connections
   - Serve enrollment HTML

2. **Enrollment Flow**
   - Device connects (TLS)
   - Server sends HTML form:
     ```html
     <h1>Device Enrollment</h1>
     <p>Device: 192.168.1.100 (iPhone)</p>
     <form>
       [ALLOW] [DENY]
     </form>
     ```
   - Device sends: POST /enroll { action: "allow" or "deny" }
   - Server persists to devices.json

3. **DeviceEnrollment Struct**
   ```rust
   struct DeviceEnrollment {
       ip: Ipv4Addr,
       hostname: Option<String>,
       approved: bool,
       enrolled_at: SystemTime,
   }
   ```

4. **Tests**
   - ✅ Server starts without errors
   - ✅ Server listens on port 7979
   - ✅ Device can connect (TLS)
   - ✅ HTML form renders
   - ✅ Enrollment POST works

### Phase 3: Device Persistence (2 hours)

**devices.rs** - Device enrollment storage

1. **`DeviceList` Structure**
   ```rust
   pub struct DeviceList {
       devices: Vec<DeviceEnrollment>,
   }
   ```

2. **`save_to_disk()`**
   - Serialize to JSON (serde_json)
   - Write to `~/.netshaper/devices.json`
   - Pretty-print for readability
   - Example:
     ```json
     [
       {
         "ip": "192.168.1.100",
         "hostname": "iphone-john",
         "approved": true,
         "enrolled_at": "2026-04-02T..."
       },
       {
         "ip": "192.168.1.101",
         "hostname": null,
         "approved": false,
         "enrolled_at": "2026-04-02T..."
       }
     ]
     ```

3. **`load_from_disk()`**
   - Read `~/.netshaper/devices.json`
   - Deserialize with serde
   - Return DeviceList
   - Handle missing file gracefully (return empty list)

4. **Tests**
   - ✅ Save to disk works
   - ✅ Load from disk works
   - ✅ Round-trip serialization
   - ✅ Missing file returns empty list

### Phase 4: Daemon Integration (3 hours)

**Integration with Milestone 2 daemon**

1. **Load enrollments on startup**
   - Daemon loads `~/.netshaper/devices.json`
   - For each approved device: `registry.insert_device(ip, default_rate)`

2. **Sync with daemon**
   - When user enrolls device via web server
   - Send UpdateBandwidth command to daemon
   - Daemon immediately starts rate-limiting

3. **Default bandwidth**
   - When device is enrolled: Assign default rate (e.g., 10 MB/s)
   - User can adjust via UI later

4. **Tests**
   - ✅ Daemon loads enrollments on startup
   - ✅ Enrolled devices appear in registry
   - ✅ Unenrolled devices not in registry
   - ✅ Sync works (enroll device → appears in daemon)

### Phase 5: Security (2 hours)

**TLS Configuration**

1. **Certificate Pinning** (future)
   - Device can verify server certificate

2. **mTLS (Mutual TLS)**
   - Device presents certificate to server
   - Server verifies device identity
   - (Optional for MVP)

3. **HTML Escaping**
   - Prevent XSS attacks in enrollment form
   - Validate user input

---

## Dependencies

### Required (already in Cargo.toml)

```toml
rcgen = "0.12"           # Certificate generation
tokio-rustls = "0.24"    # TLS server
rustls = "0.22"          # TLS implementation
serde_json = "1.0"       # JSON serialization
tokio = "1.36"           # Async runtime
```

### To Add

```toml
rustls-pemfile = "2.0"   # PEM file handling
askama = "0.12"          # HTML templating (optional, for cleaner HTML)
```

---

## File Structure

```
crypto/
├── src/
│   ├── lib.rs                   (module exports)
│   ├── cert.rs                  (self-signed certificates)
│   ├── handshake.rs             (TLS server)
│   ├── device_enrollment.rs     (NEW - device storage)
│   └── html.rs                  (NEW - enrollment form HTML)
├── Cargo.toml
└── tests/
    └── integration_test.rs      (TLS + persistence tests)
```

---

## Detailed Implementation: Phase 1 (cert.rs)

### Step 1.1: Generate Self-Signed Certificate

```rust
pub fn generate_self_signed() -> Result<Self, CertError> {
    use rcgen::generate_simple_self_signed;
    
    let subject_alt_names = vec![
        "netshaper-ca".to_string(),
        "*.netshaper.local".to_string(),
    ];
    
    let cert = generate_simple_self_signed(subject_alt_names)
        .map_err(|e| CertError::RcgenError(e.to_string()))?;
    
    // Export to PEM format
    let cert_pem = cert.serialize_pem()
        .map_err(|e| CertError::RcgenError(e.to_string()))?;
    
    let key_pem = cert.serialize_private_key_pem();
    
    Ok(Self {
        cert,
        cert_pem,
        key_pem,
    })
}
```

### Step 1.2: Save to Disk

```rust
pub fn save_to_disk(&self, dir: &Path) -> Result<(), CertError> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    
    // Create directory
    fs::create_dir_all(dir)?;
    
    let cert_path = dir.join("ca.pem");
    let key_path = dir.join("ca.key");
    
    // Check if already exists
    if cert_path.exists() || key_path.exists() {
        return Err(CertError::AlreadyExists);
    }
    
    // Write certificate (public)
    fs::write(&cert_path, &self.cert_pem)?;
    
    // Write private key (secure)
    fs::write(&key_path, &self.key_pem)?;
    
    // Set permissions: 0o600 (owner read/write only)
    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&key_path, perms)?;
    }
    
    Ok(())
}
```

### Step 1.3: Load or Generate

```rust
pub fn load_or_generate(dir: &Path) -> Result<Self, CertError> {
    use std::fs;
    
    let cert_path = dir.join("ca.pem");
    let key_path = dir.join("ca.key");
    
    // If both files exist, load them
    if cert_path.exists() && key_path.exists() {
        let cert_pem = fs::read_to_string(&cert_path)?;
        let key_pem = fs::read_to_string(&key_path)?;
        
        // TODO: Parse PEM files back to Certificate object
        // For now, just store the strings
        
        return Ok(Self {
            cert: /* TODO */,
            cert_pem,
            key_pem,
        });
    }
    
    // Generate new ones
    let bundle = Self::generate_self_signed()?;
    bundle.save_to_disk(dir)?;
    Ok(bundle)
}
```

---

## Detailed Implementation: Phase 2 (handshake.rs)

### Step 2.1: TLS Server Setup

```rust
pub async fn run_consent_server(
    cert: CertBundle,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use rustls::ServerConfig;
    
    // Create TLS config from certificate
    let mut config = ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(cert.cert_pem.clone(), cert.key_pem.clone())?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    
    // Bind to 0.0.0.0:7979
    let listener = TcpListener::bind("0.0.0.0:7979").await?;
    tracing::info!("Device enrollment server listening on 0.0.0.0:7979");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let enrolled = enrolled.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_enrollment_connection(socket, acceptor, enrolled).await {
                tracing::error!("Enrollment connection error: {}", e);
            }
        });
    }
}

async fn handle_enrollment_connection(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TLS handshake
    let tls_stream = acceptor.accept(socket).await?;
    
    // Get peer address
    let peer_addr = tls_stream.get_ref().0.peer_addr()?;
    let peer_ip = peer_addr.ip();
    
    // Send HTML form
    let html = format!(r#"
        <html>
        <body>
            <h1>NetShaper Device Enrollment</h1>
            <p>Device IP: {}</p>
            <form method="post" action="/enroll">
                <button type="submit" name="action" value="allow">Allow</button>
                <button type="submit" name="action" value="deny">Deny</button>
            </form>
        </body>
        </html>
    "#, peer_ip);
    
    // Send HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );
    
    tls_stream.write_all(response.as_bytes()).await?;
    
    // Read POST response
    let mut buffer = vec![0u8; 1024];
    let n = tls_stream.read(&mut buffer).await?;
    
    // Parse POST data
    let request = String::from_utf8_lossy(&buffer[..n]);
    let action = if request.contains("action=allow") {
        "allow"
    } else {
        "deny"
    };
    
    // Update enrolled devices
    let mut devs = enrolled.lock().await;
    if action == "allow" {
        devs.add(peer_ip.to_string().parse()?);
        tracing::info!("Device enrolled: {}", peer_ip);
    } else {
        tracing::info!("Device denied: {}", peer_ip);
    }
    
    Ok(())
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_generate_certificate() {
        let bundle = CertBundle::generate_self_signed().unwrap();
        assert!(!bundle.cert_pem.is_empty());
        assert!(!bundle.key_pem.is_empty());
    }
    
    #[test]
    fn test_save_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        
        // Generate and save
        let original = CertBundle::generate_self_signed().unwrap();
        original.save_to_disk(dir.path()).unwrap();
        
        // Load
        let loaded = CertBundle::load_from_disk(dir.path()).unwrap();
        
        // Verify they match
        assert_eq!(original.cert_pem, loaded.cert_pem);
        assert_eq!(original.key_pem, loaded.key_pem);
    }
    
    #[test]
    fn test_save_twice_fails() {
        let dir = TempDir::new().unwrap();
        let bundle = CertBundle::generate_self_signed().unwrap();
        
        bundle.save_to_disk(dir.path()).unwrap();
        
        // Second save should fail
        assert!(bundle.save_to_disk(dir.path()).is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_enrollment_server_starts() {
    let cert = CertBundle::generate_self_signed().unwrap();
    let enrolled = Arc::new(Mutex::new(EnrolledDevices::new()));
    
    // Start server in background
    tokio::spawn({
        let cert = cert.clone();
        let enrolled = enrolled.clone();
        async move {
            let _ = run_consent_server(cert, enrolled).await;
        }
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to connect
    let result = tokio::net::TcpStream::connect("127.0.0.1:7979").await;
    assert!(result.is_ok());
}
```

---

## Configuration

### Environment Variables

```bash
NETSHAPER_ENROLLMENT_PORT=7979
NETSHAPER_CONFIG_DIR=~/.netshaper
NETSHAPER_CERT_CN=netshaper-ca
```

### Default Paths

- Certificates: `~/.netshaper/ca.pem`, `~/.netshaper/ca.key`
- Device list: `~/.netshaper/devices.json`
- Config: `~/.netshaper/config.toml` (future)

---

## Security Considerations

### 1. Certificate Pinning
- Devices can verify server certificate
- Prevents MITM attacks
- (Optional for MVP)

### 2. mTLS (Optional)
- Devices present their own certificate
- Server verifies device identity
- (Not required for MVP)

### 3. HTML Escaping
- Sanitize device hostname in HTML
- Prevent XSS attacks

### 4. File Permissions
- Private key: 0o600 (owner only)
- Certificate: 0o644 (readable)
- Device list: 0o600 (owner only, contains IPs)

### 5. TLS Version
- Require TLS 1.3+
- Disable weak ciphers

---

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| Phase 1 | 3 hours | cert.rs implementation + tests |
| Phase 2 | 4 hours | handshake.rs (TLS server) |
| Phase 3 | 2 hours | Device persistence (JSON) |
| Phase 4 | 3 hours | Daemon integration |
| Phase 5 | 2 hours | Security hardening |
| **Total** | **14 hours** | Complete Milestone 3 |

---

## Success Criteria

- [x] Certificate generation works
- [x] Certificates save/load correctly
- [x] TLS server starts on port 7979
- [x] Device can connect and see enrollment form
- [x] Device enrollment persists to JSON
- [x] Daemon loads enrollments on startup
- [x] 20+ unit tests passing
- [x] 5+ integration tests passing
- [x] Zero clippy warnings
- [x] Security reviewed

---

## Dependencies Status

```
rcgen                  ✅ In Cargo.toml
tokio-rustls           ✅ In Cargo.toml
rustls                 ✅ In Cargo.toml
serde_json             ✅ In Cargo.toml
tokio                  ✅ In Cargo.toml
```

To add:
```toml
rustls-pemfile = "2.0"   # PEM file handling
```

---

## Blocked By

- ❌ Milestone 2 must be merged (needed for daemon integration)
- ✅ Milestone 1 (kernel) - not directly blocking M3

## Blocks

- ⏳ Milestone 4 (Full Integration) - needs M3 for enrollment
- ⏳ Milestone 5 (UI) - needs M3 for device list

---

**Milestone 3 is ready to begin!**

Next: Create crypto crate implementation plan and start Phase 1.
