# NetShaper M5 Phase 5 - Windows Build & Test Guide

## Overview
This document provides Saksham with complete instructions to build, deploy, and test the bandwidth tracking system on Windows.

---

## Prerequisites

### Required Software
1. **Rust** (Latest stable) - https://rustup.rs/
   - Windows MSVC toolchain
   - Verify: `rustc --version` and `cargo --version`

2. **Node.js & npm** (v16+) - https://nodejs.org/
   - Verify: `node --version` and `npm --version`

3. **Git** - https://git-scm.com/
   - Verify: `git --version`

4. **Visual C++ Build Tools** (for Windows development)
   - Download from Microsoft

### Hardware Requirements
- Enough space for Rust toolchain (~10GB)
- Admin access for running daemon (network packet capture)

---

## Project Structure

```
netshaper/
├── daemon/                    # Core bandwidth limiting service
│   ├── src/
│   │   ├── main.rs           # Daemon entry point
│   │   ├── device_registry.rs # Device management
│   │   ├── bucket.rs         # Token bucket implementation + stats tracking
│   │   ├── ipc.rs            # Windows Named Pipe IPC server
│   │   └── scheduler.rs      # Packet scheduler
│   └── Cargo.toml
│
├── ui/                        # Tauri desktop application
│   ├── src-tauri/
│   │   ├── src/main.rs       # Tauri backend commands
│   │   ├── components/
│   │   │   └── DeviceStatsDisplay.tsx  # Real-time stats UI
│   │   └── index.html
│   ├── src/                  # Rust UI library
│   ├── package.json
│   └── Cargo.toml
│
├── proto/                     # Shared protocol definitions
│   ├── src/lib.rs            # IPC message types + DeviceStats struct
│   └── Cargo.toml
│
├── crypto/                    # Encryption utilities
├── wfp-callout/              # Windows Filtering Platform integration
│
└── Cargo.toml               # Workspace root
```

---

## Step 1: Clone & Setup

```bash
# Navigate to project
cd /path/to/Waifu/netshaper

# Verify branch
git branch
# Should be on: aksh/milestone-2-token-bucket

# Ensure all changes are committed
git status
# Should show "working tree clean"
```

---

## Step 2: Build the Daemon

### On Windows PowerShell (Admin)

```bash
# Install Rust (if not already done)
# Download from https://rustup.rs/ and run installer

# Navigate to project root
cd netshaper

# Build entire workspace in release mode
cargo build --release

# This will compile:
# - proto (protocol definitions)
# - daemon (main service)
# - ui (Tauri app)
# - crypto, wfp-callout (dependencies)

# Verify successful build - no errors, only warnings OK
```

**Expected output:**
```
    Finished release [optimized] target(s) in XXs
```

**Warnings are OK:**
- Unused imports (unused_imports)
- Dead code (dead_code)

---

## Step 3: Run the Daemon (Windows)

### Terminal 1 - Start Daemon

```bash
# From netshaper root
cargo run --release --bin daemon

# Expected output:
# [2026-04-03T...] INFO daemon: Starting NetShaper daemon
# [2026-04-03T...] INFO daemon: Starting IPC server on named pipe: \\.\pipe\netshaper
# [2026-04-03T...] DEBUG daemon: Named pipe server instance created
# [2026-04-03T...] INFO daemon: Starting packet scheduler
```

**Important:** Keep this terminal running while testing.

### Daemon Features Running
- ✅ IPC server listening on `\\.\pipe\netshaper`
- ✅ Token bucket scheduler active
- ✅ Bandwidth stats tracking enabled
- ✅ Ready for UI connections

---

## Step 4: Run the Tauri UI (Windows)

### Terminal 2 - Start UI

```bash
# From netshaper/ui
cd ui

# Install Node dependencies (first time only)
npm install

# Build and run in dev mode
npm run tauri dev

# Expected output:
# [info] Using config from ...
# [info] Starting dev server...
# ✓ Built successfully
# [info] Waiting for webview to be ready...
```

**UI Features:**
- Device list view
- Approve/Deny buttons for each device
- Real-time bandwidth stats (current, peak, total usage)
- Auto-refresh every 1 second

---

## Step 5: Generate Test Traffic

### Terminal 3 - Traffic Generation

#### Option A: Simple Ping Test
```bash
# From any Windows terminal
# Ping a device on your network (replace with real IP)
ping 192.168.1.100

# This generates small ICMP packets that will be tracked
```

#### Option B: HTTP Traffic (More Realistic)
```bash
# Using curl to generate sustained traffic
# Open loop that makes HTTP requests
for /L %i in (1,1,10) do curl http://example.com

# Or using PowerShell
1..10 | ForEach-Object { curl.exe http://example.com }
```

#### Option C: iperf3 (Best for Bandwidth Testing)
```bash
# Install iperf3 if needed
# From: https://iperf.fr/

# Terminal 3A - Start iperf server (on test device or remote)
iperf3 -s

# Terminal 3B - Generate traffic
iperf3 -c 192.168.1.100 -t 60 -b 5M  # 5 Mbps for 60 seconds

# This will generate consistent, measurable traffic
```

---

## Step 6: Test in UI Dashboard

### In the Tauri UI:

1. **Navigate to Device List**
   - Click "Connected Devices" tab
   - You should see discovered devices

2. **Approve a Device**
   - Find device matching your traffic source (e.g., 192.168.1.100)
   - Click "Approve" button
   - Set bandwidth limit (e.g., 10 MB/s)
   - Device status changes to "Approved"

3. **Generate Traffic** (from Terminal 3)
   - Start iperf3 traffic while watching UI

4. **Watch Real-Time Stats**
   - Current Usage: Should increase as traffic flows
   - Peak Usage: Highest rate observed
   - Total Consumption: All-time bytes

5. **Verify Bandwidth Limiting**
   - Traffic rate should not exceed configured limit
   - Stats display updates every 1 second

---

## Step 7: Complete Test Scenario

### Full End-to-End Flow

```
Terminal 1 (Daemon):
$ cargo run --release --bin daemon
[INFO] IPC server listening on \\.\pipe\netshaper
[INFO] Packet scheduler started
└─ Keep running ✓

Terminal 2 (UI):
$ npm run tauri dev
[info] Dev server running on http://localhost:5173
└─ Keep running ✓

Terminal 3 (Traffic):
$ iperf3 -c 192.168.1.100 -t 60 -b 10M
Connecting to host 192.168.1.100, port 5201
[  5]   0.00-10.00 sec  12.5 MBytes  10.5 Mbps
[  5]  10.00-20.00 sec  12.5 MBytes  10.5 Mbps
└─ Running traffic ✓

UI Dashboard:
1. Device 192.168.1.100 shows as "Approved"
2. Bandwidth limit: 10 MB/s
3. Current usage increases in real-time ✓
4. Stats update every 1 second ✓
5. Peak usage recorded ✓
```

---

## Data Flow Diagram

```
Network Traffic
    ↓
WFP Kernel Callout (Windows Filtering Platform)
    ↓ (PacketMetadata via Named Pipe)
IPC Server (daemon/src/ipc.rs)
    ↓
Token Bucket (daemon/src/bucket.rs)
    ├─ Try consume tokens
    ├─ Update stats (current_usage, peak_usage, total_consumption)
    └─ Permit or Drop packet
    ↓
UI requests stats via get_device_stats()
    ↓
Daemon returns DeviceStats struct
    ↓
React Component updates display
```

---

## Key Files for Reference

### Backend Implementation
- **`proto/src/lib.rs`** - DeviceStats struct definition
  - `current_usage`: bytes in active 1-second window
  - `peak_usage`: highest byte rate observed
  - `total_consumption`: all-time bytes
  - `bandwidth_limit`: configured bytes/sec

- **`daemon/src/bucket.rs`** - Stats tracking logic
  - `record_consumption()` - Called when packet approved
  - `get_current_usage()` - Get 1-second window usage
  - `get_peak_usage()` - Get peak rate
  - Window resets every 1 second automatically

- **`daemon/src/ipc.rs`** - IPC command handlers
  - `GetDeviceStats(ip)` - Get stats for one device
  - `GetAllDeviceStats` - Get stats for all devices

### UI Implementation
- **`ui/src/main.rs`** - Tauri backend commands
  - `get_device_stats(ip)` - Tauri command to fetch stats
  - `get_all_device_stats()` - Fetch all stats

- **`ui/src-tauri/components/DeviceStatsDisplay.tsx`** - React component
  - Auto-refresh every 1 second
  - Displays current, peak, total usage
  - Human-readable byte formatting

---

## Troubleshooting

### Daemon won't start
```
Error: Failed to create named pipe
Solution: 
- Run as Administrator
- No other process using \\.\pipe\netshaper
- Windows firewall not blocking
```

### UI can't connect to daemon
```
Error: Connection failed to \\.\pipe\netshaper
Solution:
- Ensure daemon is running (Terminal 1)
- Check IPC is listening on correct pipe name
- UI must run on same Windows machine as daemon
```

### No devices showing up
```
Error: Device list is empty
Solution:
- Ensure traffic is being generated
- WFP kernel callout must be integrated
- Check daemon logs for packet processing
```

### Stats not updating
```
Error: Bandwidth stats showing zeros
Solution:
- Generate traffic from appropriate device
- Ensure device is "Approved" (not blocked)
- Check UI auto-refresh is enabled (should be 1s interval)
```

### High CPU usage
```
Solution:
- Normal during traffic generation
- Token bucket calculations are lightweight
- Reduce refresh rate if needed
```

---

## Testing Checklist

- [ ] Daemon starts without errors
- [ ] UI connects successfully
- [ ] Device list shows available devices
- [ ] Can approve/deny devices
- [ ] Traffic generation starts
- [ ] Current usage increases in real-time
- [ ] Peak usage recorded correctly
- [ ] Total consumption accumulates
- [ ] Stats display updates every ~1 second
- [ ] Bandwidth limit is enforced
- [ ] Multiple devices can be managed simultaneously

---

## Performance Expectations

| Metric | Expected |
|--------|----------|
| UI refresh rate | 1 Hz (1 second) |
| Stats accuracy | ±5% (depends on WFP timing) |
| Memory usage | ~50-100 MB |
| CPU usage (idle) | <1% |
| CPU usage (high traffic) | 5-15% |
| Response time | <100ms |

---

## Build Artifacts Location

After successful build:

```
Release binaries:
- daemon: target/release/daemon.exe
- ui: target/release/netshaper-ui.exe (bundled)

For manual testing:
cargo run --release --bin daemon
npm run tauri dev
```

---

## Git Workflow After Testing

```bash
# After successful testing on Windows
git checkout main

git merge aksh/milestone-2-token-bucket

git push origin main

# Tag the milestone
git tag -a M5-Phase-5 -m "Bandwidth tracking with real-time stats"
git push origin M5-Phase-5
```

---

## Contact & Support

For issues during testing:
- Check daemon logs for error messages
- Verify IPC connection with test script
- Review Windows Event Viewer for system errors
- Check firewall/antivirus not blocking

---

## Next Steps After M5 Phase 5

1. **M5 Phase 6**: Advanced rate limiting algorithms
2. **M5 Phase 7**: UI analytics dashboard
3. **M5 Phase 8**: Deployment packaging

Good luck with testing! 🚀
