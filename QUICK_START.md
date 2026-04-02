# Quick Start Checklist for Saksham (Windows)

## Pre-Flight Check (Do Once)
- [ ] Install Rust from https://rustup.rs/
- [ ] Install Node.js v16+ from https://nodejs.org/
- [ ] Verify: `rustc --version`
- [ ] Verify: `npm --version`
- [ ] Clone repo (already done)
- [ ] `cd netshaper`

## Build (Do Once)
```bash
cargo build --release
```
- [ ] Build completes successfully (warnings OK)
- [ ] Takes 2-5 minutes first time

## Test Day Setup (Each Session)

### Terminal 1: Start Daemon
```bash
cargo run --release --bin daemon
```
**Wait for:** `INFO daemon: IPC server listening on \\.\pipe\netshaper`

### Terminal 2: Start UI
```bash
cd ui
npm install  # (first time only)
npm run tauri dev
```
**Wait for:** `✓ Built successfully` and UI window appears

### Terminal 3: Generate Traffic
```bash
# Option 1: iperf3 (Best)
iperf3 -c 192.168.1.100 -t 60 -b 10M

# Option 2: Curl
for /L %i in (1,1,100) do curl http://example.com

# Option 3: Ping
ping 192.168.1.100
```

## Testing Steps

1. **In UI Dashboard:**
   - [ ] See device list
   - [ ] Click "Approve" on target device
   - [ ] Set bandwidth limit (e.g., 10 MB/s)

2. **Start Traffic (Terminal 3)**
   - [ ] iperf3 or curl running
   - [ ] Check traffic is flowing

3. **Watch Dashboard**
   - [ ] Current Usage increases in real-time
   - [ ] Peak Usage updates
   - [ ] Total Consumption accumulates
   - [ ] Stats refresh every ~1 second

4. **Verify Rate Limiting**
   - [ ] Traffic doesn't exceed bandwidth limit
   - [ ] Device is responsive
   - [ ] No errors in daemon logs

## Success Criteria
✅ All pass = M5 Phase 5 works!

- [ ] Daemon starts without errors
- [ ] UI shows device list
- [ ] Can approve/deny devices
- [ ] Real-time stats display
- [ ] Bandwidth limit enforced
- [ ] No crashes or hangs

## Stop Everything
```bash
Terminal 1: Ctrl+C (daemon)
Terminal 2: Ctrl+C (UI)
Terminal 3: Ctrl+C (traffic)
```

## Key IPC Messages (For Debugging)

**Daemon logs to watch:**
```
[INFO] Device 192.168.1.100 approved: 10000000 bytes/sec
[INFO] Packet received: 1500 bytes
[DEBUG] Device 192.168.1.100 consumed: 1500 bytes
```

**UI commands sent:**
- `GetDeviceStats(192.168.1.100)` → Returns stats every 1s
- `UpdateBandwidth(ip, limit)` → When approving/denying
- `ListDevices` → When loading device list

## Expected File Locations After Build
```
target/release/daemon.exe        # Daemon binary
target/release/netshaper-ui.exe  # UI bundle (in dev mode: browser)
```

---

**Need Help?** Read WINDOWS_BUILD_AND_TEST.md for detailed troubleshooting.
