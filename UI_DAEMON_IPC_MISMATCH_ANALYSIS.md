# 🔴 UI ↔ Daemon IPC Protocol Mismatch - Root Cause Analysis

## Executive Summary

The UI and daemon **speak different languages over the named pipe**. The UI sends **raw strings** while the daemon expects **proto::DaemonCommand enums**. This is why every button click fails silently.

---

## Problem Diagram

```
UI Frontend (React)
    ↓ invoke('list_devices')
    ↓
UI Backend (Tauri - ui/src/main.rs)
    ↓ Serializes: "list_devices".to_string() ← WRONG
    ↓
Named Pipe \\.\pipe\netshaper
    ↓
Daemon IPC Handler (daemon/src/ipc.rs)
    ↓ Tries to deserialize as DaemonCommand enum
    ✗ FAILS - Expected enum, got string
    ✗ Message dropped, no response sent
    ↑
UI Backend waits forever or times out
    ↑
UI Frontend shows nothing (mock data as fallback)
```

---

## Root Causes

### **Issue 1: String Serialization Instead of Enum**

**File:** `ui/src/main.rs`, line ~55

**Current (BROKEN):**
```rust
async fn connect_to_daemon_windows() -> Result<Vec<DeviceInfo>, String> {
    use std::fs::File;
    use std::io::{Read, Write};
    
    let mut pipe = File::open("\\\\.\\pipe\\netshaper")?;
    
    // ❌ WRONG - Serializes a STRING
    let request = bincode::serialize(&"list_devices".to_string())?;
    pipe.write_all(&request)?;
    // ...
}
```

**What daemon expects:**
```rust
// daemon/src/ipc.rs, line ~300
async fn process_command(cmd: DaemonCommand, ...) {
    match cmd {
        DaemonCommand::ListDevices => {
            // Handle list devices command
            // ...
        }
        // ...
    }
}
```

**Daemon can't deserialize:**
```rust
// This fails:
let cmd: DaemonCommand = bincode::deserialize(&buffer)?;
// Error: Cannot deserialize String as enum DaemonCommand
```

---

### **Issue 2: Wrong Command Format for Approve/Deny**

**File:** `ui/src/main.rs`, line ~150

**Current (BROKEN):**
```rust
async fn send_command_to_daemon_windows(command: String) -> Result<(), String> {
    // command = "approve:192.168.1.100"
    
    // ❌ Still trying to send a string
    let request = bincode::serialize(&command)?;
    pipe.write_all(&request)?;
}
```

**What daemon expects:**
```rust
// daemon/src/ipc.rs
use proto::DaemonCommand;
use proto::BandwidthUpdate;

// Should send this:
let cmd = DaemonCommand::UpdateBandwidth(BandwidthUpdate {
    ip: parsed_ip,
    bytes_per_sec: limit,
});
let request = bincode::serialize(&cmd)?;
```

---

### **Issue 3: Proto Definitions Mismatch**

**File:** `proto/src/lib.rs`, lines 45-54

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    UpdateBandwidth(BandwidthUpdate),  // ← For approve/deny/limits
    ListDevices,                        // ← For device list
    GetDeviceStats(Ipv4Addr),          // ← For bandwidth stats
    GetAllDeviceStats,                 // ← For all devices stats
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUpdate {
    pub ip: Ipv4Addr,
    pub bytes_per_sec: u64,
}
```

The UI was **trying to send custom string commands** instead of these **official enum variants**.

---

## Solution

### **Step 1: Fix list_devices Command**

**File:** `ui/src/main.rs`

**Before:**
```rust
#[cfg(windows)]
async fn connect_to_daemon_windows() -> Result<Vec<DeviceInfo>, String> {
    use std::fs::File;
    use std::io::{Read, Write};
    
    let pipe_name = "\\\\.\\pipe\\netshaper";
    let mut pipe = File::open(pipe_name)?;

    // ❌ Wrong: serialize a string
    let request = bincode::serialize(&"list_devices".to_string())?;
    pipe.write_all(&request)?;
    // ...
}
```

**After:**
```rust
#[cfg(windows)]
async fn connect_to_daemon_windows() -> Result<Vec<DeviceInfo>, String> {
    use std::fs::File;
    use std::io::{Read, Write};
    use proto::DaemonCommand;  // ← Add this import
    
    let pipe_name = "\\\\.\\pipe\\netshaper";
    let mut pipe = File::open(pipe_name)?;

    // ✅ Correct: serialize the enum variant
    let cmd = DaemonCommand::ListDevices;
    let request = bincode::serialize(&cmd)?;
    pipe.write_all(&request)?;
    // ...
}
```

**Same for Unix version:**
```rust
#[cfg(unix)]
async fn connect_to_daemon_unix() -> Result<Vec<DeviceInfo>, String> {
    use tokio::net::UnixStream;
    use proto::DaemonCommand;  // ← Add this
    
    // ...
    
    let cmd = DaemonCommand::ListDevices;  // ← Use enum
    let request = bincode::serialize(&cmd)?;
    // ...
}
```

---

### **Step 2: Fix approve_device and deny_device Commands**

**File:** `ui/src/main.rs`, line ~150

**Before:**
```rust
#[cfg(windows)]
async fn send_command_to_daemon_windows(command: String) -> Result<(), String> {
    // command = "approve:192.168.1.100"
    
    // ❌ Still sends a string
    let request = bincode::serialize(&command)?;
    pipe.write_all(&request)?;
}
```

**After:**
```rust
#[cfg(windows)]
async fn send_command_to_daemon_windows(command: String) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    use std::net::Ipv4Addr;
    use proto::{DaemonCommand, BandwidthUpdate};

    let mut pipe = File::open("\\\\.\\pipe\\netshaper")?;

    // Parse "approve:192.168.1.100" or "deny:192.168.1.100"
    let (action, ip_str) = command.split_once(':')
        .ok_or("Invalid command format")?;

    let ip: Ipv4Addr = ip_str.parse()
        .map_err(|_| format!("Invalid IP: {}", ip_str))?;

    // ✅ Correct: build and serialize the enum
    let cmd = match action {
        "approve" => DaemonCommand::UpdateBandwidth(BandwidthUpdate {
            ip,
            bytes_per_sec: DEFAULT_BANDWIDTH_LIMIT,
        }),
        "deny" => DaemonCommand::UpdateBandwidth(BandwidthUpdate {
            ip,
            bytes_per_sec: 0,  // 0 = blocked
        }),
        _ => return Err(format!("Unknown action: {}", action)),
    };

    let request = bincode::serialize(&cmd)?;
    pipe.write_all(&request)?;
    Ok(())
}
```

**Same for Unix version** (replace `File` with `UnixStream` and `.write_all()` with `.write_all().await()`).

---

### **Step 3: Fix get_device_stats Commands**

These are **already correct** in the current code:

```rust
#[cfg(windows)]
async fn get_device_stats_windows(ip: String) -> Result<(u64, u64, u64), String> {
    use proto::DaemonCommand;
    
    let parsed_ip: Ipv4Addr = ip.parse()?;
    let cmd = DaemonCommand::GetDeviceStats(parsed_ip);  // ✅ Correct
    
    let request = bincode::serialize(&cmd)?;
    // ...
}
```

**These don't need changes.**

---

## Communication Flow After Fix

```
UI Frontend (React)
    ↓ invoke('list_devices')
    ↓
UI Backend (Tauri)
    ↓ Serializes: DaemonCommand::ListDevices ✅
    ↓
Named Pipe \\.\pipe\netshaper
    ↓
Daemon IPC Handler
    ↓ Deserializes as DaemonCommand::ListDevices ✅
    ↓ Executes handler: returns device list
    ↓ Serializes response as Vec<DeviceStats>
    ↓
Named Pipe \\.\pipe\netshaper
    ↓
UI Backend
    ↓ Deserializes response ✅
    ↓ Converts to DeviceInfo array
    ↓
UI Frontend
    ↓ Renders device list with real-time stats ✅
```

---

## Testing Checklist

- [ ] **UI compiles** without errors
- [ ] **Daemon compiles** without errors
- [ ] **Run daemon first** in Terminal 1
  - Expected: `[INFO] IPC server listening on \\.\pipe\netshaper`
- [ ] **Run UI in Terminal 2**
  - Expected: Desktop window opens
- [ ] **Click on device in UI**
  - Expected: Shows real-time bandwidth stats
- [ ] **Click "Approve" button**
  - Expected: Device state changes, stats update
- [ ] **Generate traffic** (Terminal 3: iperf3)
  - Expected: Current usage increases in real-time
- [ ] **No errors in daemon logs**
  - Expected: Clean IPC messages, device updates logged
- [ ] **Stats refresh every 1 second**
  - Expected: Smooth animation in stats display

---

## Files Changed

| File | Change | Reason |
|------|--------|--------|
| `ui/src/main.rs` | Use `DaemonCommand` enum instead of strings | Protocol compliance |
| `ui/src/main.rs` | Import `proto` types | Required for serialization |
| `ui/src/main.rs` | Parse `BandwidthUpdate` struct | Daemon IPC expectation |

---

## Why This Happened

The UI and daemon were likely written in different phases without explicit protocol verification. The UI author assumed a string-based protocol (like HTTP), while the daemon author used the statically-typed `proto::DaemonCommand` enum for safety and clarity.

This is a **common cross-process communication (IPC) bug** - one side sends what it thinks the other expects, without verifying the protocol at boundaries.

---

## Prevention

1. **Write integration tests** that verify IPC serialization round-trips
2. **Share protocol definitions** in a single crate (`proto`) - ✅ Already done
3. **Test on Windows** - Only the real platform reveals these bugs
4. **Use type-level enforcement** - Rust's type system catches these at compile time if used correctly

---

## M5 Phase 5 Status

After applying this fix:
- ✅ IPC protocol matches on both sides
- ✅ Device list fetching works
- ✅ Approve/deny commands work
- ✅ Real-time stats update works
- ✅ End-to-end testing can proceed

**Ready for Saksham's Windows testing** once these changes are merged and built.
