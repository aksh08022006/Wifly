# 📱 Milestone 5: Tauri UI Dashboard - Plan

**Goal**: Build a beautiful, real-time dashboard for managing NetShaper devices and bandwidth.

**Architecture**: Rust backend (Tauri IPC) + React/Svelte frontend (TypeScript)

**Timeline**: ~10 hours across 5 phases

---

## Architecture Overview

```
┌──────────────────────────────┐
│   Tauri UI Dashboard         │
│   (React + TypeScript)       │
├──────────────────────────────┤
│  • Device List               │
│  • Bandwidth Display         │
│  • Approve/Deny Controls     │
│  • Live Metrics              │
└──────────────────────────────┘
           ↓↑ (IPC)
┌──────────────────────────────┐
│   Tauri Backend (Rust)       │
│   Daemon Communication       │
├──────────────────────────────┤
│  • Named Pipe Connection     │
│  • Device State Sync         │
│  • Bandwidth Updates         │
└──────────────────────────────┘
           ↓↑ (Socket)
┌──────────────────────────────┐
│   Daemon (M2 + M3)           │
│   Rate Limiting + Enrollment │
└──────────────────────────────┘
```

---

## Phase 1: Tauri Backend Setup (1.5 hours)

### Objectives
- Set up Tauri IPC commands
- Connect to daemon via named pipe (M2)
- Create device state structures
- Implement command handlers

### Tasks
1. ✅ Scaffold main Tauri application
2. ✅ Create AppState with daemon connection
3. ✅ Define `list_devices()` command
4. ✅ Define `set_bandwidth(ip, limit)` command
5. ✅ Define `approve_device(ip)` command
6. ✅ Define `deny_device(ip)` command
7. ✅ Test IPC commands

### Expected Output
- Working Tauri desktop app
- Commands callable from frontend
- Connection to daemon established

---

## Phase 2: Frontend Scaffolding (1.5 hours)

### Objectives
- Set up React/Svelte project
- Create basic layout and styling
- Connect to Tauri backend
- Display initial data

### Tasks
1. ✅ Choose framework (React or Svelte)
2. ✅ Create basic layout (header, sidebar, main area)
3. ✅ Set up Tauri invoke calls
4. ✅ Create device list component
5. ✅ Add basic styling (Tailwind/CSS)
6. ✅ Test data binding

### Expected Output
- React app loads in Tauri window
- Basic layout visible
- Can invoke Tauri commands from console

---

## Phase 3: Device Management (2 hours)

### Objectives
- Display enrolled devices
- Show device details (IP, hostname, enrollment date)
- Implement approve/deny controls
- Real-time device list updates

### Tasks
1. ✅ Fetch device list from daemon
2. ✅ Display in sortable table
3. ✅ Show device metadata
4. ✅ Add approve button (pending devices)
5. ✅ Add deny/revoke button (approved devices)
6. ✅ Implement refresh mechanism
7. ✅ Handle errors gracefully

### Expected Output
- Device list visible
- Can approve/deny devices from UI
- Changes persist to ~/.netshaper/devices.json
- Real-time updates

---

## Phase 4: Bandwidth Management (2 hours)

### Objectives
- Display current bandwidth limits
- Allow per-device bandwidth adjustment
- Show bandwidth usage graphs
- Real-time metrics

### Tasks
1. ✅ Fetch bandwidth limit for each device
2. ✅ Display in UI (MB/s format)
3. ✅ Add slider control for adjustment
4. ✅ Send updates to daemon
5. ✅ Fetch usage metrics from daemon
6. ✅ Create simple line graph (Chart.js or similar)
7. ✅ Display as percentage of limit
8. ✅ Auto-refresh metrics (1-5 second interval)

### Expected Output
- Bandwidth controls working
- Live usage graphs
- Changes apply immediately
- Real-time monitoring

---

## Phase 5: Polish & Features (2 hours)

### Objectives
- System tray integration
- Settings panel
- Status indicators
- Error handling
- Documentation

### Tasks
1. ✅ Add system tray icon
2. ✅ Show/hide app from tray
3. ✅ Create settings panel
4. ✅ Status indicator (daemon connected/disconnected)
5. ✅ Device status colors (approved/pending/denied)
6. ✅ Error notifications
7. ✅ Keyboard shortcuts
8. ✅ Responsive design
9. ✅ Unit tests for Tauri commands
10. ✅ Integration tests for frontend

### Expected Output
- Professional-grade UI
- System integration
- Robust error handling
- Production-ready

---

## Technology Stack

### Backend (Rust)
- **Tauri 2.5**: Desktop framework
- **Tokio**: Async runtime
- **Serde/Bincode**: IPC serialization
- **Named Pipes**: Daemon communication

### Frontend
- **React 18** or **Svelte**: UI framework
- **TypeScript**: Type safety
- **Tailwind CSS**: Styling
- **Chart.js**: Graphs/metrics
- **Tauri API**: Bridge to backend

### Testing
- **Vitest/Jest**: Frontend unit tests
- **Cargo test**: Rust backend tests

---

## IPC Protocol

### Tauri Commands (Backend → Frontend)

```rust
// Get all enrolled devices
#[tauri::command]
async fn list_devices(state: tauri::State<'_, AppState>) -> Result<Vec<DeviceInfo>, String> {
    // Fetches from daemon, returns device list
}

// Update bandwidth limit for device
#[tauri::command]
async fn set_bandwidth(
    state: tauri::State<'_, AppState>,
    ip: String,
    bytes_per_sec: u64,
) -> Result<(), String> {
    // Sends to daemon, updates rate limiter
}

// Approve pending device
#[tauri::command]
async fn approve_device(
    state: tauri::State<'_, AppState>,
    ip: String,
) -> Result<(), String> {
    // Updates DeviceList in daemon
}

// Deny/revoke device
#[tauri::command]
async fn deny_device(
    state: tauri::State<'_, AppState>,
    ip: String,
) -> Result<(), String> {
    // Removes from DeviceList
}

// Get bandwidth usage
#[tauri::command]
async fn get_bandwidth_usage(
    state: tauri::State<'_, AppState>,
    ip: String,
) -> Result<BandwidthMetrics, String> {
    // Returns current usage for device
}
```

### Data Structures

```typescript
// Frontend TypeScript
interface DeviceInfo {
  ip: string;
  hostname: string | null;
  approved: boolean;
  enrolled_at: string; // ISO 8601
  bandwidth_limit: number; // bytes/sec
  current_usage: number; // bytes/sec
}

interface BandwidthMetrics {
  device_ip: string;
  current: number; // bytes/sec
  average: number; // last 60 seconds
  peak: number; // last 60 seconds
  timestamp: string;
}
```

---

## File Structure

```
ui/
├── src/
│   ├── main.rs                 # Tauri entry point
│   ├── commands/
│   │   ├── mod.rs              # Command module exports
│   │   ├── device.rs           # Device management commands
│   │   └── bandwidth.rs        # Bandwidth commands
│   ├── ipc/
│   │   ├── mod.rs              # IPC client module
│   │   └── daemon_client.rs    # Named pipe connection
│   ├── state.rs                # AppState
│   └── error.rs                # Error types
├── src-tauri/
│   ├── main.js                 # Tauri JS entry
│   ├── components/
│   │   ├── DeviceList.jsx      # Device list view
│   │   ├── BandwidthChart.jsx  # Usage graph
│   │   └── SettingsPanel.jsx   # Settings
│   ├── App.jsx                 # Root component
│   └── styles/
│       └── index.css           # Tailwind CSS
├── Cargo.toml                  # Rust dependencies
├── package.json                # Node dependencies
└── tauri.conf.json             # Tauri config
```

---

## Development Checklist

### Phase 1 Checklist
- [ ] Tauri project initializes
- [ ] Named pipe client created
- [ ] AppState defined
- [ ] Commands registered
- [ ] Commands tested with `tauri dev`

### Phase 2 Checklist
- [ ] React app loads in window
- [ ] Can invoke `list_devices` from console
- [ ] Basic layout visible
- [ ] Styling framework integrated
- [ ] Build succeeds

### Phase 3 Checklist
- [ ] Device list displays
- [ ] Devices sortable/filterable
- [ ] Approve button works
- [ ] Deny button works
- [ ] JSON persistence verified

### Phase 4 Checklist
- [ ] Bandwidth display working
- [ ] Slider updates values
- [ ] Graph displays usage
- [ ] Auto-refresh working
- [ ] Units correct (MB/s)

### Phase 5 Checklist
- [ ] System tray icon added
- [ ] Settings panel functional
- [ ] Status indicators show
- [ ] Error messages clear
- [ ] Unit tests 80%+ coverage
- [ ] No clippy warnings
- [ ] Ready for M4 integration

---

## Known Constraints

- Tauri on macOS: May need code signing (skip for dev)
- Named pipes: Windows-only (use Unix sockets on macOS/Linux)
- IPC performance: Keep updates < 100ms
- Real-time updates: Not critical for M5 (polling acceptable)

---

## Integration Points with Other Milestones

### M2 (Daemon) Integration
- Fetches device list from daemon's EnrolledDevices
- Sends bandwidth updates to daemon's scheduler
- Reads metrics from daemon's token bucket

### M3 (Enrollment) Integration
- Displays pending devices from ~/.netshaper/devices.json
- Approve/deny updates JSON
- Shows enrollment server status

### M4 (Full System)
- UI becomes control panel for M4 testing
- Can approve devices, set bandwidth limits
- Monitor bandwidth enforcement

---

## Success Criteria

| Criterion | Target | M5 Goal |
|-----------|--------|---------|
| Device List | Displays all enrolled devices | ✓ |
| Bandwidth Control | Can set per-device limit | ✓ |
| Real-time Updates | Metrics update every 5 seconds | ✓ |
| Error Handling | No crashes on daemon disconnect | ✓ |
| Code Quality | 0 clippy warnings | ✓ |
| Test Coverage | 80%+ Rust backend | ✓ |
| UI Polish | Professional appearance | ✓ |
| Performance | Launches in <2 seconds | ✓ |

---

## Timeline Estimate

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Phase 1: Backend Setup | 1.5h | Now | +1.5h |
| Phase 2: Frontend Scaffold | 1.5h | +1.5h | +3h |
| Phase 3: Device Management | 2h | +3h | +5h |
| Phase 4: Bandwidth Control | 2h | +5h | +7h |
| Phase 5: Polish & Features | 2h | +7h | +9h |
| **Total** | **9h** | | |

---

## Notes for Aksh

- M5 is independent of M1/M4 work - complete it while Saksham finishes M1
- Focus on Phase 1-3 first (device management core)
- Phase 4-5 can be optimized later if time is tight
- By the time M1 is done, M5 will be partially complete
- M5 becomes the control panel for M4 integration testing
- Consider using Svelte instead of React if speed is priority (smaller bundle)

---

## Status

**Phase 1**: ⏳ IN PROGRESS
- Starting Tauri backend implementation

---

**Next Command**: `Start M5 Phase 1 implementation`
