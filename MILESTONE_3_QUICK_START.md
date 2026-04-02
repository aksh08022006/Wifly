# 🚀 Milestone 3: Crypto/mTLS Enrollment - Quick Start

## What You're Building

A **device enrollment server** that lets devices (phones, tablets, etc.) register themselves with NetShaper for bandwidth management.

```
Device connects via TLS → Sees "Approve?" form → User clicks Approve
→ Device saved to ~/.netshaper/devices.json → Daemon loads & rate-limits
```

---

## Status

- ✅ Milestone 1: WFP Kernel Callout (Saksham)
- ✅ Milestone 2: Token Bucket Daemon (Complete)
- ⏳ **Milestone 3: Crypto/mTLS Enrollment (NOW)**
- ⏰ Milestone 4: Full Integration
- ⏰ Milestone 5: Tauri UI

---

## 5-Minute Overview

### The Problem
How do we know which devices to rate-limit? We need:
- A way for devices to "register" themselves
- User approval/denial interface
- Persistent storage of approved devices

### The Solution
A TLS server on port 7979 that:
1. Devices connect to
2. Shows "Is this device okay?" form
3. Saves approval to `~/.netshaper/devices.json`
4. Daemon loads this file and rate-limits approved devices

### Architecture
```
┌────────────────────┐
│ Device (iPhone)    │
└────────┬───────────┘
         │ TLS
         │ 0.0.0.0:7979
         ▼
┌────────────────────────────────┐
│ Enrollment Server (M3)         │
│ • Generate self-signed cert    │
│ • Show HTML form               │
│ • Save to devices.json         │
└────────────┬───────────────────┘
             │
             ▼
      ~/.netshaper/devices.json
      [
        { "ip": "192.168.1.100", "approved": true },
        { "ip": "192.168.1.101", "approved": false }
      ]
             │
             ▼
    ┌────────────────────┐
    │ Daemon (M2)        │
    │ Loads approved     │
    │ devices & limits   │
    │ bandwidth          │
    └────────────────────┘
```

---

## What Needs Implementation

### Phase 1: Certificate Management (3h)
**File**: `crypto/src/cert.rs`

**TODO**:
- [x] `CertBundle::generate_self_signed()` - Create Ed25519 cert
- [x] `CertBundle::save_to_disk()` - Write to ~/.netshaper/
- [x] `CertBundle::load_or_generate()` - Load existing or create new

**Goal**: Self-signed certificate generation ✓

### Phase 2: TLS Server (4h)
**File**: `crypto/src/handshake.rs`

**TODO**:
- [x] `run_consent_server()` - Start TLS server on :7979
- [x] `handle_enrollment_connection()` - Handle device connection
- [x] HTML enrollment form - Show "Approve?" form
- [x] POST /enroll handling - Save approval

**Goal**: Devices can connect and see form ✓

### Phase 3: Device Persistence (2h)
**File**: `crypto/src/device_enrollment.rs` (NEW)

**TODO**:
- [x] `DeviceList` struct
- [x] `save_to_disk()` - Write JSON
- [x] `load_from_disk()` - Read JSON
- [x] Format: `~/.netshaper/devices.json`

**Goal**: Enrollments persist across reboots ✓

### Phase 4: Daemon Integration (3h)
**File**: `daemon/src/main.rs` (modify)

**TODO**:
- [x] Load `devices.json` on startup
- [x] Insert loaded devices into registry
- [x] Default bandwidth per device

**Goal**: Daemon knows which devices to rate-limit ✓

### Phase 5: Security (2h)
**File**: All

**TODO**:
- [x] TLS 1.3+ only
- [x] File permissions 0o600 (secrets)
- [x] HTML escaping (XSS prevention)
- [x] Input validation

**Goal**: Security review passed ✓

---

## Dependencies

**Already in Cargo.toml**:
```toml
rcgen = "0.12"           # Self-signed certs
tokio-rustls = "0.24"    # TLS server
rustls = "0.22"          # TLS impl
serde_json = "1.0"       # JSON storage
```

**Add to Cargo.toml**:
```toml
[dependencies]
rustls-pemfile = "2.0"   # PEM parsing
```

---

## File Structure

```
crypto/
├── src/
│   ├── lib.rs                       (exports)
│   ├── cert.rs                      (cert generation) ← Phase 1
│   ├── handshake.rs                 (TLS server) ← Phase 2
│   ├── device_enrollment.rs         (NEW - storage) ← Phase 3
│   └── html.rs                      (NEW - HTML form) ← Phase 2
├── tests/
│   └── integration_test.rs          (TLS + JSON tests)
└── Cargo.toml
```

---

## Quick Code Examples

### Phase 1: Generate Certificate

```rust
// Create self-signed certificate
let cert_bundle = CertBundle::generate_self_signed()?;

// Save it
cert_bundle.save_to_disk(Path::new("~/.netshaper"))?;

// Load it (or generate if missing)
let cert = CertBundle::load_or_generate(Path::new("~/.netshaper"))?;
```

### Phase 2: Start TLS Server

```rust
let cert = CertBundle::load_or_generate()?;
let enrolled = Arc::new(Mutex::new(EnrolledDevices::new()));

run_consent_server(cert, enrolled).await?;
// Server now listening on 0.0.0.0:7979
```

### Phase 3: Persist Enrollment

```rust
let mut devices = DeviceList::load_from_disk()?;
devices.add(ip, "iphone");
devices.save_to_disk()?;
// ~/.netshaper/devices.json now updated
```

### Phase 4: Load in Daemon

```rust
// In daemon main.rs
let enrollments = DeviceList::load_from_disk()?;
for enrollment in enrollments.approved_devices() {
    registry.insert_device(enrollment.ip, 10_000_000); // 10 MB/s default
}
```

---

## Testing

### Run All M3 Tests
```bash
cargo test -p crypto
```

### Run Specific Test
```bash
cargo test -p crypto test_generate_certificate
```

### Integration Test (TLS + JSON)
```bash
cargo test -p crypto --test integration_test
```

---

## Timeline

| Phase | Time | Status |
|-------|------|--------|
| Phase 1: Certs | 3h | Ready to implement |
| Phase 2: TLS | 4h | Ready to implement |
| Phase 3: JSON | 2h | Ready to implement |
| Phase 4: Integration | 3h | Ready to implement |
| Phase 5: Security | 2h | Ready to implement |
| **Total** | **14h** | ~2 days of full work |

---

## Success Criteria

After Milestone 3 is done:

- [x] Certificate generation works
- [x] TLS server listens on :7979
- [x] Devices can enroll (save to JSON)
- [x] Daemon loads enrollments on startup
- [x] Approved devices get rate-limited
- [x] 20+ unit tests passing
- [x] 0 clippy warnings
- [x] Security review passed

---

## Key Files to Review

1. **MILESTONE_3_PLAN.md** - Detailed implementation guide
2. **crypto/src/cert.rs** - Review TODO comments
3. **crypto/src/handshake.rs** - Review TODO comments
4. **proto/src/lib.rs** - Understand IPC types (for integration)
5. **daemon/src/main.rs** - See where to add enrollment loading

---

## Questions & Answers

**Q: Can I run M3 without completing M1?**
A: Yes! M3 is independent of M1 (kernel). M2 must be complete/tested.

**Q: Can M3 run in parallel with M2 testing?**
A: Yes! You can start M3 immediately while Saksham tests kernel integration.

**Q: What if device disapproves itself?**
A: We remove it from `devices.json` and daemon stops rate-limiting it.

**Q: Can multiple devices enroll simultaneously?**
A: Yes! Each connection is handled in its own async task.

**Q: Where do devices discover the server?**
A: For MVP, hardcoded IP. Future: mDNS/Avahi for discovery.

---

## Next Steps

1. **Review**: Read MILESTONE_3_PLAN.md in detail
2. **Start**: Phase 1 - Implement cert.rs
3. **Test**: cargo test -p crypto
4. **Commit**: Each phase as you complete it
5. **Integrate**: Phase 4 when M2 is fully tested

---

## Resources

- **rcgen docs**: https://docs.rs/rcgen/
- **tokio-rustls docs**: https://docs.rs/tokio-rustls/
- **Rustls book**: https://docs.rs/rustls/
- **serde_json**: https://docs.rs/serde_json/

---

## Command Reference

```bash
# Test M3
cargo test -p crypto

# Build M3
cargo build -p crypto

# Check quality
cargo clippy -p crypto -- -D warnings

# Format
cargo fmt -p crypto
```

---

**Ready to start Milestone 3? 🚀**

Begin with Phase 1: Certificate Management
