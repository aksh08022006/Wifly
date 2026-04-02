# Windows Application Testing - Visual Guide

## YES! You Can See the Application While Testing

The NetShaper Windows application provides a **visual desktop interface** where you can see everything in real-time.

---

## What You'll See

### Terminal 1: Daemon (Background Service)

```
C:\netshaper> cargo run --release --bin daemon

   Compiling netshaper-daemon v0.1.0 ...
    Finished release [optimized] target(s) in 15.23s
     Running `target\release\daemon.exe`

[2026-04-03T10:15:32.123Z INFO  netshaper_daemon] Starting NetShaper daemon v0.1.0
[2026-04-03T10:15:32.456Z INFO  netshaper_daemon::ipc] IPC server listening on \\.\pipe\netshaper
[2026-04-03T10:15:32.789Z INFO  netshaper_daemon::scheduler] Packet scheduler started
[2026-04-03T10:15:32.912Z DEBUG netshaper_daemon::device_registry] Device registry initialized

← Keep running in background
```

**Status Indicators:**
- ✅ IPC listening on `\\.\pipe\netshaper` = Ready for UI
- ✅ Packet scheduler started = Ready to track bandwidth
- ✅ Device registry initialized = Ready to manage devices

---

### Terminal 2: Tauri UI Application (What You'll See)

```
C:\netshaper\ui> npm run tauri dev

[info] Using config from ...
[info] Starting dev server...
[info] Vite v4.x.x ready in 234 ms

  ➜  Local:   http://localhost:5173/
  ➜  press h to show help

[info] Waiting for webview to be ready...
✓ Built successfully

← UI window appears on screen! ↓↓↓
```

**A Desktop Window Opens:**

```
╔════════════════════════════════════════════════════════════╗
║                    NetShaper Dashboard                      ║
║ ┌──────────────────────────────────────────────────────────┤
║ │ File  Edit  View  Help                                  │
╠════════════════════════════════════════════════════════════╣
║                                                             ║
║  📡 Connected Devices                                      ║
║                                                             ║
║  Device IP          Status      Bandwidth Limit            ║
║  ─────────────────────────────────────────────────────     ║
║  192.168.1.100      Pending     [Set Limit]  [Approve]    ║
║  192.168.1.101      Approved    10 MB/s      [Deny]       ║
║  192.168.1.102      Pending     [Set Limit]  [Approve]    ║
║                                                             ║
║  Selected Device: 192.168.1.101                           ║
║  ─────────────────────────────────────────────────────────  ║
║                                                             ║
║  📊 Real-Time Statistics (Auto-refresh every 1 second)    ║
║                                                             ║
║  Current Usage:    2.45 MB/s  [▓▓▓▓░░░░░░] 24%           ║
║  Peak Usage:       5.87 MB/s  Recorded                    ║
║  Total Consumed:   156.3 MB   Since approval              ║
║                                                             ║
║  Bandwidth Limit:  10 MB/s                                ║
║  Status: Active and limiting ✓                            ║
║                                                             ║
║  [Manual Refresh]  [Export Stats]  [Clear History]        ║
║                                                             ║
╚════════════════════════════════════════════════════════════╝
```

---

### What Updates in Real-Time

When you run traffic (Terminal 3), you'll see LIVE updates:

**Second 1:**
```
Current Usage:    1.23 MB/s  [▓▓░░░░░░░░] 12%
Peak Usage:       1.23 MB/s
Total Consumed:   1.23 MB
```

**Second 2:**
```
Current Usage:    2.45 MB/s  [▓▓▓▓░░░░░░] 24%
Peak Usage:       2.45 MB/s
Total Consumed:   3.68 MB   ← Increases
```

**Second 3:**
```
Current Usage:    4.56 MB/s  [▓▓▓▓▓▓░░░░] 45%
Peak Usage:       4.56 MB/s  ← Highest rate
Total Consumed:   8.24 MB    ← Still accumulating
```

---

## How to Test - Complete Visual Walkthrough

### Phase 1: Start Everything (3 Terminals)

**Terminal 1 - Daemon:**
```bash
C:\netshaper> cargo run --release --bin daemon
[✓] IPC server listening
```

**Terminal 2 - UI:**
```bash
C:\netshaper\ui> npm run tauri dev
[✓] Desktop window opens showing device list
```

**Terminal 3 - Traffic:**
```bash
C:\> iperf3 -c 192.168.1.100 -t 60 -b 10M
Connected with 192.168.1.100
Sending 10 MB/s for 60 seconds
```

---

### Phase 2: Watch the Dashboard Update

#### Before Starting Traffic:
```
Current Usage:    0 KB/s   [░░░░░░░░░░] 0%
Peak Usage:       0 KB/s
Total Consumed:   0 B
```

#### After Starting Traffic (Real-Time):
The stats update **every 1 second**:

```
Time:  T+0s    T+5s      T+10s     T+15s     T+30s
─────────────────────────────────────────────────────
Curr:  0→      3.2→      5.1→      4.8→      4.9 MB/s
Peak:  0→      3.2→      5.1→      5.1→      5.1 MB/s  
Total: 0→      16MB→     51MB→     72MB→     147MB
```

---

### Phase 3: Verify Bandwidth Limiting

**Dashboard will show:**

✅ **If Limit = 10 MB/s:**
- Current Usage stays under 10 MB/s
- Even if you try sending more
- Excess packets are dropped/queued

✅ **Status Updates:**
- Green checkmark: Active and limiting
- Red warning: Over limit detected (adjust)
- Blue info: Stats streaming OK

---

## Key Visual Components

### 1. Device List View
```
Shows all discovered devices:
✗ Pending devices (gray) - approve to manage
✓ Approved devices (green) - actively limited
⊘ Denied devices (red) - no limit tracking
```

### 2. Real-Time Stats Panel
```
Updates automatically:
- Current Usage: Live bandwidth right now
- Peak Usage: Highest spike seen
- Total Consumed: All-time accumulation
- Visual bar chart showing percentage
```

### 3. Control Buttons
```
[Approve] - Enable bandwidth management
[Deny] - Stop tracking device
[Set Limit] - Configure max bandwidth
[Refresh] - Manual update (auto-updates anyway)
[Export] - Save stats to CSV/JSON
```

### 4. Status Indicators
```
✓ Green = Healthy, actively limiting
⚠ Yellow = Warning, approaching limit
✗ Red = Error, exceeding limit
◑ Blue = Informational message
```

---

## How to See Everything

### View 1: Daemon Output (Terminal 1)
```
See logs of:
- Device registrations
- Packet processing
- IPC messages
- Error diagnostics
```

### View 2: UI Dashboard (Desktop Window)
```
See in real-time:
- Connected devices
- Bandwidth statistics
- Bandwidth limits applied
- Device status
- Visual graphs and indicators
```

### View 3: Traffic Output (Terminal 3)
```
See in iperf3:
- Bandwidth being sent
- Transfer progress
- Connections established
- Real-time throughput
```

---

## Testing Example

### Step 1: Everything Started
```
Terminal 1: ✓ Daemon running
Terminal 2: ✓ UI window visible with empty device list
Terminal 3: Ready for traffic
```

### Step 2: Device Discovered
```
UI Dashboard shows:
┌─ 192.168.1.100 [Pending] [Approve] ─┐
```

### Step 3: Approve Device with Limit
```
User clicks [Approve]
Sets limit to: 5 MB/s
UI shows: ✓ 192.168.1.100 [Approved] 5MB/s [Deny]
```

### Step 4: Start Traffic
```
Terminal 3: iperf3 -c 192.168.1.100 -t 60 -b 10M
```

### Step 5: Watch Stats Update
```
UI Dashboard updates every 1 second:

Current: 2.3→ 4.1→ 4.8→ 4.9→ 5.0 MB/s (capped!)
Peak:    2.3→ 4.1→ 4.8→ 4.8→ 4.9 MB/s
Total:   2MB→ 25MB→ 47MB→ 52MB→ 142MB

Status: ✓ Active and limiting bandwidth
```

### Step 6: Verify Limiting Works
```
Daemon logs show:
[TRACE] Device 192.168.1.100: Consumed 2.3 MB/s
[TRACE] Device 192.168.1.100: Peak updated to 4.8 MB/s
[TRACE] Bandwidth limited to 5 MB/s (no excess)
```

---

## What You Can't See (But Happens in Background)

```
- Kernel packet filtering (WFP layer)
- Token bucket algorithm calculations
- Named pipe IPC serialization
- Device registry updates
- Scheduler queueing decisions
```

**But you CAN see the results:**
- Devices appear in list
- Stats update in real-time
- Bandwidth is actually limited

---

## Troubleshooting Visual Issues

### UI Window Doesn't Appear
```
Check Terminal 2:
- Look for "✓ Built successfully"
- Look for "Waiting for webview to be ready"
- If stuck: Kill and restart with npm run tauri dev
```

### Stats Don't Update
```
Check:
1. Daemon still running in Terminal 1?
2. Traffic actually flowing in Terminal 3?
3. Device approved in UI?
4. Look at daemon logs for errors
```

### Dashboard is Blank/Empty
```
Check:
1. Are devices on the same network?
2. Is daemon listening on \\.\pipe\netshaper?
3. Are devices sending traffic?
4. Check daemon logs for "No devices discovered"
```

---

## Summary

**YES - You Can Definitely See the Application!**

You get **3 views**:
1. 🖥️ **Desktop UI** - Visual device management and stats
2. 🖨️ **Terminal Logs** - Daemon operations and debug info
3. 📊 **Real-Time Stats** - Bandwidth metrics updating live

Everything updates in **real-time** as traffic flows!

---

**Last Updated**: 3 April 2026
**Test Ready**: ✅ YES
**Visual Output**: ✅ Desktop Window + 3 Terminal Views
