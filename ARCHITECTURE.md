# NetShaper - Proxy Router Architecture & Specification

**Version:** 1.0 (M6 - Proxy Router Implementation)
**Date:** April 3, 2026

---

## Executive Summary

NetShaper is a **Windows desktop proxy router application** that allows users to:
- Turn their laptop into an HTTP/HTTPS proxy server
- Connect mobile devices and other computers via QR code pairing
- Monitor and control bandwidth for each connected device
- Apply traffic shaping rules in real-time

**Not a web application.** Standalone Windows desktop application (like RStudio, Wireshark).

---

## Core Architecture

### 1. Proxy Server Component
**Role:** HTTP/HTTPS proxy that intercepts and manages network traffic

**Startup Flow:**
```
User opens NetShaper
    ↓
App loads system WiFi settings (speed, bandwidth)
    ↓
User clicks "Start Proxy" button
    ↓
Proxy server starts on localhost:8080
    ↓
Generate QR code containing:
    - Device IP address (192.168.x.x)
    - Proxy port (8080)
    - Unique pairing token
    - Device name (e.g., "Saksham-Laptop")
    ↓
Waiting for devices to scan and connect
```

---

### 2. Device Pairing System
**Role:** Connect remote devices (phones, laptops) to proxy with authentication

**QR Code Contains:**
```json
{
  "server": "192.168.x.x:8080",
  "token": "ABC123XYZ789",
  "device_name": "Saksham-Laptop",
  "version": "1.0"
}
```

**Pairing Flow:**
```
Phone scans QR code
    ↓
Phone connects to proxy on 192.168.x.x:8080
    ↓
Daemon receives connection, shows pairing request on laptop
    ↓
Pairing notification appears in left panel:
    "NewDevice (192.168.1.101) requesting access"
    [Accept]  [Reject]
    ↓
User clicks Accept
    ↓
Device added to Connected Devices list
    ↓
All phone traffic now routed through laptop proxy
```

---

### 3. Bandwidth Management (Main Playground)
**Default Settings (from WiFi Provider):**
- Auto-read current WiFi speed from Windows
- Display as starting point
- User can modify globally or per-device

**Per-Device Controls:**
```
Connected Device: iPhone-12
├─ Current Usage: 2.3 Mbps (real-time)
├─ Bandwidth Limit: [5 Mbps] ← User can edit
├─ Priority: [Medium] ← Auto/High/Medium/Low
├─ Downloaded: 456 MB (total)
├─ Uploaded: 123 MB (total)
└─ Actions: [Throttle] [Pause] [Block] [Disconnect]
```

---

## UI Design (RStudio-like: 3-Panel Layout)

### Left Panel: Device Management
```
╔═════════════════════════════════╗
║  NetShaper - Proxy Router       ║
╠═════════════════════════════════╣
║  [▶ Start Proxy] [⚙ Settings]  ║
╠═════════════════════════════════╣
║  Connected Devices (2)          ║
╟─────────────────────────────────╢
║  ✓ iPhone-12                    ║
║    192.168.1.101 • 2 mins       ║
║  ✓ MacBook-Pro                  ║
║    192.168.1.102 • 15 mins      ║
╟─────────────────────────────────╢
║  Pairing Requests (1)           ║
╟─────────────────────────────────╢
║  ⏳ NewPhone (192.168.1.103)    ║
║     [Accept]  [Reject]          ║
╟─────────────────────────────────╢
║  [📱 Scan QR to Add Device]     ║
╚═════════════════════════════════╝
```

### Center Panel: Real-time Monitoring
```
╔════════════════════════════════════════════════════════╗
║     NETWORK MONITOR                                    ║
╠════════════════════════════════════════════════════════╣
║                                                        ║
║  Total Bandwidth Usage:                              ║
║  ████████░░░░░░░░░ 8.5 / 20 Mbps                    ║
║                                                        ║
║  ┌─────────────────────────────────────┐             ║
║  │  Traffic Graph (Last 5 minutes)      │             ║
║  │  10│          ╱╲                     │             ║
║  │   8│    ╱╲╱╲╱  ╲╱╲╱                 │             ║
║  │   6│╱╲╱╱  ╲╱╲╱╱                   │             ║
║  │   4│                                │             ║
║  │   2│                                │             ║
║  │   0└─────────────────→ Time         │             ║
║  └─────────────────────────────────────┘             ║
║                                                        ║
║  Top 3 Connections:                                  ║
║  1. iPhone-12:    5.2 Mbps  (YouTube)               ║
║  2. MacBook-Pro:  2.8 Mbps  (Zoom)                  ║
║  3. iPhone-12:    1.0 Mbps  (Spotify)               ║
║                                                        ║
║  Total Down: 456 MB | Total Up: 123 MB              ║
╚════════════════════════════════════════════════════════╝
```

### Right Panel: Device Control & Settings
```
╔═════════════════════════════════╗
║  Device: iPhone-12              ║
╠═════════════════════════════════╣
║                                 ║
║  Status: ✓ Connected            ║
║  IP Address: 192.168.1.101      ║
║  MAC Address: AA:BB:CC:DD       ║
║  Connected: 2 minutes ago       ║
║  OS: iOS 17.4                   ║
║                                 ║
╠═════════════════════════════════╣
║  BANDWIDTH CONTROL              ║
╟─────────────────────────────────╢
║                                 ║
║  Bandwidth Limit:               ║
║  ┌──────────────────────┐       ║
║  │ [5 Mbps        ▼]    │       ║
║  │ ████░░░░░░░░░░░░ 5   │       ║
║  └──────────────────────┘       ║
║  [ 1 ] [ 2 ] [ 5 ] [ 10 ] [∞]  ║
║                                 ║
║  Priority Level:                ║
║  ◯ High    ◉ Medium   ◯ Low     ║
║                                 ║
║  QoS Policy:                    ║
║  [Standard ▼]                   ║
║                                 ║
╠═════════════════════════════════╣
║  QUICK ACTIONS                  ║
╟─────────────────────────────────╢
║  [Throttle] [Pause]             ║
║  [Block All] [Disconnect]       ║
║  [Schedule Rule] [Alerts]       ║
╚═════════════════════════════════╝
```

---

## Technical Implementation Details

### Proxy Server
```rust
// Listen on localhost:8080
// Implement HTTP/1.1 with CONNECT tunneling for HTTPS
// Per-device traffic accounting
// Token-based rate limiting (from M5)

Port: 8080 (configurable)
Protocol: HTTP/1.1 + HTTPS tunneling
```

### QR Code Generation
```rust
// Generate QR code with pairing data
// Display in UI
// Expire tokens after 5 minutes
// New QR on each "Start Proxy" click

use qrcode::QrCode;
```

### Device Pairing
```rust
// Device connects to proxy with token
// IPC notification to UI
// User accepts/rejects
// Approved devices added to whitelist
```

### Bandwidth Control
```rust
// Read WiFi speed from Windows API
// Display as default limit
// Per-device token bucket (use existing M5 logic)
// Apply per connection
```

---

## Implementation Phases

### Phase M6: Proxy Router Core (Current)
- [ ] HTTP proxy server on localhost:8080
- [ ] QR code generation & display
- [ ] Device pairing flow (connect/accept/reject)
- [ ] Basic device list in UI
- [ ] Manual "Start Proxy" / "Stop Proxy" buttons

### Phase M7: Traffic Control
- [ ] Bandwidth limiting per device
- [ ] Real-time traffic monitoring
- [ ] Graphs and statistics
- [ ] Save/restore proxy settings

### Phase M8: Advanced Features
- [ ] Application-level traffic detection
- [ ] Scheduling rules
- [ ] Alert system
- [ ] Traffic history logs

### Phase M9: Polish
- [ ] Windows Firewall integration
- [ ] System tray icon
- [ ] Keyboard shortcuts
- [ ] Export reports

---

## No Pre-loaded Examples

**Key Requirement:** Zero example devices shown at startup.

```
✗ DO NOT SHOW:
  - Hardcoded mock devices
  - Demo data
  - Pre-connected devices

✓ SHOW INSTEAD:
  - Empty device list
  - "No devices connected" message
  - [Scan QR to Connect] button
  - Information about how to start
```

---

## Default WiFi Settings Logic

```
At Startup:
1. Read current WiFi SSID and speed
2. Check ISP-provided bandwidth limit
3. Store as baseline

When Device Connects:
- Set initial limit to ISP baseline
- Show in UI: "20 Mbps (WiFi Default)"
- User can increase/decrease per device

When Setting Bandwidth:
- Global cap prevents exceeding WiFi speed
- Per-device cap can be lower
- Priority system for fair distribution
```

---

## Test Checklist

- [ ] Start Proxy button works
- [ ] QR code displays correctly
- [ ] Phone scans QR and connects
- [ ] Pairing request appears in UI
- [ ] Accept/Reject buttons work
- [ ] Device list updates
- [ ] Bandwidth slider works
- [ ] Real-time traffic shows
- [ ] No example devices on startup

---

## Success Criteria for M6

By end of M6:
- ✓ Proxy server functional (port 8080)
- ✓ QR code generation & pairing
- ✓ Device list (connected + pending)
- ✓ Manual start/stop proxy
- ✓ RStudio-like 3-panel UI
- ✓ Zero pre-loaded examples
- ✓ Default WiFi settings applied
