# NetShaper M1-M5 Testing Guide

## What You're Testing Now (M1-M5)

### What's Working:
- ✅ **M1**: Thread-safe device registry - devices can be stored and managed
- ✅ **M2**: Token bucket rate limiting - each device has bandwidth limits
- ✅ **M3**: IPC communication - daemon can receive commands
- ✅ **M4**: Real-time stats - see live bandwidth usage
- ✅ **M5**: Bandwidth tracking - current usage, peaks, and totals

### What's NOT Yet Implemented (M6+):
- ❌ Proxy router (M6) - not built yet
- ❌ Device pairing via QR code (M6) - not built yet
- ❌ HTTP server for real communication (M6) - not built yet
- ❌ Real device detection (M6) - not built yet

---

## UI Button Guide

### 🔄 Refresh Devices
- **What it does**: Reloads device list from the system
- **Currently**: Shows test devices (since Tauri API not working)
- **Test it**: Click to see 4 pre-loaded test devices

### ➕ Add Test Device
- **What it does**: Opens a form to create a fake device for testing
- **How to use**:
  1. Click button
  2. Enter device name (e.g., "PlayStation 5")
  3. Enter IP address (e.g., 192.168.1.150)
  4. Enter bandwidth limit (MB/s)
  5. Click "Add Device"
- **Test**: Add a device and see it appear in Pending list

### 📊 Router Stats
- **What it does**: Shows proxy router statistics and network health
- **Shows**:
  - Network status (Active/Inactive)
  - Device count (Approved vs Pending)
  - Total bandwidth allocated
  - Current usage stats
  - Router runtime status
- **Test**: Click to see router information

---

## How to Test Device Connection

1. **Start the app** (daemon + UI must be running)

2. **Click "➕ Add Test Device"**
   - Add a device like:
     - Name: "Test-iPhone"
     - IP: "192.168.1.150"
     - Bandwidth: "10" (MB/s)

3. **Device appears in "Pending Devices"** section
   - Shows IP, enrolled date
   - Shows current bandwidth usage
   - Has ✓ Approve and ✕ Deny buttons

4. **Click "✓ Approve"**
   - Device moves to "Approved" section
   - Now actively managed by router
   - Bandwidth limited to what you set

5. **Click "✕ Deny"**
   - Device removed from network
   - Gets blocked by router

6. **View stats with "📊 Router Stats"**
   - See total bandwidth allocated
   - See how many devices approved
   - Monitor overall usage

---

## Why "Tauri API Not Found"?

The daemon is running fine, but the UI can't communicate with it because:
- **Tauri's JavaScript API** isn't injecting into the window
- This is a known Tauri setup issue on Windows
- **Solution**: We show mock data instead so you can test the UI

**What we're using instead**:
- Mock devices (4 devices pre-loaded)
- Mock operations (add/approve/deny are local)
- In M6, we'll implement HTTP server instead of Tauri IPC

---

## What Each Toggle/Option Does

### Mock Data Mode
- Currently always ON (because Tauri not working)
- When ON: Uses local test devices
- When OFF: Would try to connect to real daemon
- **Don't worry**: This is temporary - M6 will use HTTP

### Refresh Devices
- Reloads the device list
- In M6: Will pull from real daemon
- Now: Just shows mock devices

---

## Current Architecture

```
┌─────────────────────────────────────────────┐
│   NetShaper UI (Tauri Window)               │
│   - Device management interface             │
│   - Approve/Deny buttons                    │
│   - Bandwidth stats display                 │
└──────────────┬──────────────────────────────┘
               │
               │ (Currently: Mock data)
               │ (M6 will use: HTTP + IPC)
               │
┌──────────────▼──────────────────────────────┐
│   Daemon (Rust binary)                      │
│   - M1: Device registry (✓)                 │
│   - M2: Token bucket (✓)                    │
│   - M3: IPC receiver (✓)                    │
│   - M4: Stats tracker (✓)                   │
│   - M5: Bandwidth window (✓)                │
│   - M6: Proxy router (❌ coming)            │
└─────────────────────────────────────────────┘
```

---

## Testing Checklist

- [ ] Launch UI window
- [ ] Click "Refresh Devices" → See 4 test devices
- [ ] Click "Add Test Device" → Add iPhone
- [ ] See iPhone in "Pending Devices"
- [ ] Click "✓ Approve" on iPhone
- [ ] See iPhone move to "Approved Devices"
- [ ] Click "✕ Deny" on another device
- [ ] See it removed
- [ ] Click "📊 Router Stats"
- [ ] See router statistics show updated counts
- [ ] Check notification messages appear

---

## What's Next (M6)

To get the full experience with real device connection:
1. Implement HTTP proxy server (localhost:8080)
2. Add device pairing via QR code
3. Fix Tauri IPC OR use HTTP fallback
4. Real device detection from network
5. Actual bandwidth enforcement

But for now, you can **test the entire UI workflow** with mock devices! 🎯
