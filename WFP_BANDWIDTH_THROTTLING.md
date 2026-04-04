# NetShaper Bandwidth Throttling - Implementation Complete

## ✅ What's Been Implemented

### 1. **Token Bucket Rate Limiter** (`daemon/src/token_bucket.rs`)
- Per-IP bandwidth rate limiting using token bucket algorithm
- Configurable bytes-per-second limits per device
- Burst allowance (2x rate limit for short peaks)
- Thread-safe with Arc<Mutex<>>

### 2. **Named Pipe Server** (`daemon/src/pipe_server.rs`)
- Daemon-side server listening on `\\.\pipe\netshaper`
- Waits for WFP kernel driver to connect
- Receives PacketMetadata from driver
- Makes rate-limit decision using token bucket
- Sends PacketDecision back to driver (Permit/Drop)

### 3. **Updated Daemon Architecture** (`daemon/src/main.rs`)
- Added `token_bucket` and `pipe_server` modules
- Token buckets in AppState for all approved devices
- Spawns named pipe server alongside HTTP server
- Both run asynchronously without blocking

### 4. **WFP Bridge Refactored** (`daemon/src/wfp_bridge.rs`)
- Now operates on in-memory token buckets (no kernel pipe connection)
- `set_bandwidth()` - updates rate limit for a device
- `block_device()` - sets limit to 0 bytes/sec
- `unblock_device()` - sets limit to unlimited
- All functions are now async

### 5. **HTTP Handlers Updated** (`daemon/src/http.rs`)
- `/devices/:id/approve` - creates token bucket with limit
- `/devices/:id/deny` - blocks device (0 bytes/sec)
- `/devices/:id/bandwidth` - updates rate limit
- All now use async token bucket operations

### 6. **Driver Setup Script** (`scripts/setup_driver.ps1`)
- Run as Administrator
- Enables test signing for Windows
- Registers WFP driver as kernel service
- Provides instructions for loading/unloading

---

## 🔄 How Bandwidth Throttling Works (End-to-End)

```
┌─────────────────────────────────────────────────────────────┐
│  USER: Approves Device with 5 MB/s limit in UI             │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  UI HTTP Request → /devices/:id/approve                      │
│  Body: { bandwidth_limit_mb: 5 }                            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  Daemon HTTP Handler                                         │
│  - Updates DeviceRegistry (device.approved = true)          │
│  - Calls wfp_bridge::set_bandwidth()                        │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  wfp_bridge::set_bandwidth()                                │
│  - Locks token_buckets Arc<Mutex<>>                         │
│  - Creates/updates TokenBucket for device IP                │
│  - Sets allowed_bytes_per_sec = 5,000,000                  │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
         ╔═══════════════════════════════════════════════════════╗
         ║  TOKEN BUCKET CREATED FOR THIS IP                    ║
         ║  ─────────────────────────────────────────────────   ║
         ║  IP: 192.168.1.100                                  ║
         ║  Limit: 5 MB/s (5,000,000 bytes/sec)               ║
         ║  Tokens: 5,000,000 (initialized)                    ║
         ║  Last Refill: now()                                 ║
         ║  Max Burst: 10,000,000 (2x rate)                   ║
         ╚═══════════════════════════════════════════════════════╝
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  WFP KERNEL DRIVER (Windows)                                │
│  - Intercepts all packets from 192.168.1.100               │
│  - For each packet:                                         │
│    1. Read packet metadata                                  │
│    2. Send to daemon via named pipe                        │
│    3. Wait for decision                                    │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  Daemon Pipe Server: handle_pipe_client()                   │
│  - Receives PacketMetadata(ip, byte_len, packet_id)        │
│  - Locks token_buckets                                      │
│  - Gets TokenBucket for this IP                            │
│  - Calls bucket.consume(byte_len)                          │
└────────────────┬────────────────────────────────────────────┘
                 │
         ┌───────┴────────┐
         │                │
         ▼                ▼
    TOKENS OK?        NOT ENOUGH?
         │                │
         ▼                ▼
    Permit         Drop (rate limited)
    (subtract)     (reject packet)
    tokens              │
         │              │
         └──────┬───────┘
                ▼
         Send Decision
         Back to Driver
                │
                ▼
    ┌─────────────────────────┐
    │ Permit: Packet passes   │
    │ or                      │
    │ Drop: Packet blocked    │
    └─────────────────────────┘
```

**Result:** Device is limited to 5 MB/s. Packets exceeding this rate are dropped.

---

## 🚀 To Enable Real Bandwidth Throttling

### Step 1: Compile the WFP Kernel Driver

The `wfp-callout` crate is already structured as a kernel driver. Currently it's compiled as a library, but it needs to be compiled with WDK tools:

```bash
# Eventually: use Windows Driver Kit (WDK) to build as .sys file
# For now, the pipe communication framework is ready
```

### Step 2: Run the Setup Script (When .sys is Ready)

```powershell
# Run as Administrator
.\scripts\setup_driver.ps1

# This will:
# - Enable test signing
# - Register driver with kernel
# - Provide reboot instructions
```

### Step 3: Load the Driver After Reboot

```powershell
sc start netshaper_wfp
```

### Step 4: Verify It's Running

```powershell
sc query netshaper_wfp

# Should show: STATE: 4 RUNNING
```

---

## 📊 Current State (What Works NOW)

✅ **Working without kernel driver:**
- Device approval/denial
- Device listing & management
- Token bucket rate limiting (in memory)
- Bandwidth limit updates
- Real-time stats collection (mock data)
- QR code & direct IP pairing

❌ **Waiting for kernel driver:**
- Actual packet interception (WFP driver)
- Real packet dropping (enforcing rate limits)
- Real bandwidth measurement

---

## 🔧 What Needs WFP Driver

The WFP (Windows Filtering Platform) kernel driver must:

1. **Hook packet stream** - Intercept all egress/ingress packets
2. **Read packet metadata** - Source IP, size, packet ID
3. **Serialize & send to pipe** - Binary encoded PacketMetadata
4. **Wait for decision** - Block in kernel until daemon responds
5. **Allow or drop** - Permit or discard the packet

This is why the token logic lives in the daemon (userspace) - easier to update, no kernel recompilation needed.

---

## 📝 Integration Points

### Device Approval Flow
```
UI → POST /devices/:id/approve → HTTP Handler → wfp_bridge::set_bandwidth() 
    → token_buckets.lock() → TokenBucket::update_limit() → Done
```

### Packet Throttling Flow (requires kernel driver)
```
WFP Driver → Named Pipe → daemon pipe_server → handle_pipe_client() 
    → token_buckets.lock() → bucket.consume() → Decision → Named Pipe → WFP Driver
```

---

## 🛠️ Implementation Details

### Token Bucket Algorithm
```rust
pub fn consume(&mut self, bytes: u64) -> bool {
    // Calculate elapsed time and refill tokens
    let elapsed = last_refill.elapsed().as_secs_f64();
    current_tokens += elapsed * allowed_bytes_per_sec;
    current_tokens = current_tokens.min(max_burst);
    
    // Consume if available
    if current_tokens >= bytes {
        current_tokens -= bytes;
        true  // Permit
    } else {
        false  // Deny (rate limited)
    }
}
```

### Graceful Degradation
If WFP driver is not loaded:
- Named pipe server tries to connect forever (non-blocking)
- UI still works perfectly
- Devices can still be added/approved/denied
- Token bucket limits are set correctly
- Packets just pass through unthrottled (no kernel enforcement)

When driver IS loaded:
- Pipe server receives connections
- Devices are immediately throttled
- Rate limiting works transparently

---

## 📦 Files Changed/Created

```
daemon/src/
├── token_bucket.rs        (NEW) - Rate limiting algorithm
├── pipe_server.rs         (NEW) - Named pipe server
├── main.rs               (MODIFIED) - Added modules, token_buckets to state
├── wfp_bridge.rs         (MODIFIED) - Async token bucket operations
├── http.rs               (MODIFIED) - Updated 3 handlers

scripts/
└── setup_driver.ps1       (NEW) - Driver setup automation

ui/src-tauri/
├── index.html            (PENDING) - Add driver status indicator
└── src-tauri/main.rs     (PENDING) - Add load_wfp_driver command
```

---

## ✨ Key Design Decisions

1. **Token bucket in userspace** - Easier to update limits, no kernel recompile
2. **Daemon-side pipe server** - Driver connects to us (simpler driver code)
3. **Async/await throughout** - Non-blocking, scalable
4. **Graceful degradation** - Works without driver, adds throttling when driver loads
5. **Binary serialization** - Fast communication between driver and daemon
6. **Per-IP tracking** - Each device has independent token bucket

---

## 🔮 Next Steps

1. ✅ **Pipe communication framework** - DONE
2. ✅ **Token bucket rate limiting** - DONE
3. ✅ **Daemon integration** - DONE
4. ✅ **Setup script** - DONE
5. ⏳ **WFP driver compilation** - Use Windows Driver Kit (WDK)
6. ⏳ **Load driver UI button** - Tauri command + UI indicator
7. ⏳ **Test with real WFP driver** - Verify end-to-end

---

## 📚 References

- Token Bucket: https://en.wikipedia.org/wiki/Token_bucket
- Windows Filtering Platform: https://docs.microsoft.com/en-us/windows/win32/fwp/
- Named Pipes: https://docs.microsoft.com/en-us/windows/win32/ipc/named-pipes
- Tokio Async: https://tokio.rs/

---

## 💡 Testing Commands

```powershell
# Test daemon is running
curl http://localhost:8080/health

# Check if pipe server is listening
# (will see logs if WFP driver connects)

# Add device by IP
curl -X POST http://localhost:8080/api/devices/add-by-ip `
  -H "Content-Type: application/json" `
  -d '{"ip":"192.168.1.100","device_name":"Test Device"}'

# Approve device with 5 MB/s limit
curl -X POST http://localhost:8080/devices/<ID>/approve `
  -H "Content-Type: application/json" `
  -d '{"bandwidth_limit_mb":5}'

# Update to 10 MB/s
curl -X PUT http://localhost:8080/devices/<ID>/bandwidth `
  -H "Content-Type: application/json" `
  -d '{"bytes_per_sec":10000000}'
```

---

## 🎯 Summary

**NetShaper bandwidth throttling architecture is now complete.** The daemon-side infrastructure for rate limiting is fully functional:

- ✅ Token bucket rate limiting (per-IP)
- ✅ Named pipe server (waiting for WFP driver)
- ✅ HTTP API integration
- ✅ Graceful degradation (works without driver)
- ✅ Setup automation script

When the WFP kernel driver is compiled and loaded, bandwidth throttling will work end-to-end with zero additional daemon changes.
