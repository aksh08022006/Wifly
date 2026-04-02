# 🚀 NetShaper Development Workflow for Saksham

**Project:** NetShaper — Windows Bandwidth Controller  
**Your Role:** Windows Driver (`wfp-callout/`) & UI (`ui/`) Development  
**Status:** Milestone 0 Setup Phase  
**Repository:** https://github.com/aksh08022006/Wifly

---

## 📋 Table of Contents

1. [Quick Overview](#quick-overview)
2. [Milestone 0: Setup (1 day)](#milestone-0-setup-1-day)
3. [Milestone 1: WFP Callout Skeleton (3 days)](#milestone-1-wfp-callout-skeleton-3-days)
4. [Milestone 4: IPC Bridge Integration (3 days shared)](#milestone-4-ipc-bridge-integration-3-days-shared)
5. [Milestone 5: Tauri Control Panel (4 days)](#milestone-5-tauri-control-panel-4-days)
6. [Milestone 6: Installer & MSI Packaging (2 days)](#milestone-6-installer--msi-packaging-2-days)
7. [Git Workflow & Branch Management](#git-workflow--branch-management)
8. [Testing Strategy](#testing-strategy)
9. [Known Issues & Troubleshooting](#known-issues--troubleshooting)

---

## Quick Overview

### Your Responsibilities

| Crate | Technology | Milestone | Duration |
|-------|-----------|-----------|----------|
| **wfp-callout/** | Windows WFP API + Rust | M1, M4 | 3 + 3 days |
| **ui/** | Tauri + WebView2 | M5 | 4 days |
| **Installer** | cargo-wix + MSI | M6 | 2 days |

### Architecture at a Glance

```
┌─────────────────────────────────────────────────────┐
│         Tauri System-Tray UI (Saksham)              │
│      (Bandwidth sliders, device cards)              │
└──────────────────┬──────────────────────────────────┘
                   │ Named Pipe IPC
┌──────────────────▼──────────────────────────────────┐
│        Daemon (Userspace, Aksh)                     │
│    Token bucket, packet scheduling, device registry│
└──────────────────┬──────────────────────────────────┘
                   │ Kernel IPC
┌──────────────────▼──────────────────────────────────┐
│  WFP Callout Driver (Kernel, Saksham)               │
│ Intercepts packets → asks daemon → allow/drop       │
└─────────────────────────────────────────────────────┘
```

---

## Milestone 0: Setup (1 day)

### Critical: Enable Test Signing First

Your Windows machine must allow unsigned kernel drivers to load.

#### Windows 11 (GUI):
1. Settings → System → Developer settings
2. Toggle "Enable Test Signing" to ON
3. Restart when prompted

#### Windows 10/11 (Command Line):
```powershell
# Run as Administrator
bcdedit /set testsigning on
shutdown /r /t 0
```

**Verify after restart:**
```powershell
bcdedit | findstr testsigning
# Output should be: testsigning    Yes
```

### Installation Checklist

- [ ] **Rust Stable (MSVC toolchain)**
  ```powershell
  winget install Rustlang.Rustup
  rustup toolchain install stable-x86_64-pc-windows-msvc
  ```

- [ ] **Windows Driver Kit (WDK)** — Provides WFP headers
  ```powershell
  winget install Microsoft.WindowsDriverKit
  ```
  Verify: `dir "C:\Program Files (x86)\Windows Kits\10\Include\*\km\fwpsk.h"`

- [ ] **Visual Studio Build Tools** — C++ workload for MSVC
  ```powershell
  winget install Microsoft.VisualStudio.2022.BuildTools
  ```
  During install, select:
  - Desktop development with C++
  - Windows 10 SDK (or Windows 11 SDK)

- [ ] **LLVM** (optional, speeds up linking)
  ```powershell
  winget install LLVM.LLVM
  ```

### Verification

Navigate to the repo and verify the workspace builds:

```powershell
cd C:\Users\SAKSHAM\Wifly

# Build all crates (this includes wfp-callout)
cargo build --workspace

# Run tests (unit tests, no integration tests yet)
cargo test -p proto
cargo test -p ui  # May have limited tests
```

**Success = No errors, normal compilation warnings are OK**

### Branch Setup

After toolchain verification:

```powershell
git checkout -b saksham/milestone-0-setup

# Make a small commit to record your setup completion
# E.g., update a setup checklist file
git add .
git commit -m "Milestone 0: Setup complete, toolchain verified"
git push -u origin saksham/milestone-0-setup
```

Create a PR against `main`. Aksh will review and merge when he confirms his setup is also done.

---

## Milestone 1: WFP Callout Skeleton (3 days)

### Goal

Build a minimal Windows driver (.sys) that:
- ✅ Compiles to a .sys file
- ✅ Registers a WFP callout
- ✅ Permits all packets (no filtering logic yet)
- ✅ Can be loaded/unloaded without crashing

### Architecture

**File Structure** (in `wfp-callout/src/`):
```
lib.rs              # Main driver entry (DllMain)
engine.rs           # WFP engine — create/delete filters
callout.rs          # Classify callback (packet handler)
utils.rs            # Helper functions (logging, error handling)
```

### Key WFP Concepts

| Concept | What It Does |
|---------|-------------|
| **WFP Engine** | Container for filters, callouts, providers |
| **Callout** | Kernel function called for each packet |
| **Filter** | Rule that says "pass this traffic to the callout" |
| **Classify Callback** | The function that runs in kernel space for each packet |

### Tasks (In Order)

#### Task 1.1: Create WFP Engine Skeleton (Day 1)
**File:** `wfp-callout/src/engine.rs`

```rust
pub struct WfpEngine {
    engine_handle: u64,
}

impl WfpEngine {
    pub fn new() -> Result<Self, String> {
        // Initialize WFP engine
        // - Call FwpmEngineOpen0()
        // - Store handle
        // Return error if already open
        todo!()
    }

    pub fn register_callout(&self, callout_name: &str) -> Result<(), String> {
        // Register a callout
        // - Create FWPM_CALLOUT struct
        // - Call FwpmCalloutAdd0()
        // Return error on failure
        todo!()
    }

    pub fn close(&mut self) -> Result<(), String> {
        // Safely close engine
        // - Call FwpmEngineClose0()
        // - Deallocate handle
        todo!()
    }
}
```

**Acceptance Criteria:**
- Code compiles (errors OK, warnings OK)
- Placeholder functions exist with correct signatures

#### Task 1.2: Implement Classify Callback (Day 1-2)
**File:** `wfp-callout/src/callout.rs`

```rust
// This function runs in kernel space for EACH PACKET
unsafe extern "system" fn classify_callback(
    layer: &u32,
    args: &FWPM_FILTER_CONDITION0,
    meta_values: &*const FWP_VALUE0,
    context: *mut std::ffi::c_void,
    filter_context: &*const std::ffi::c_void,
    action: &mut u32,
) {
    // For now: allow all packets
    *action = FWP_ACTION_PERMIT;
}
```

**Constraints (CRITICAL for kernel code):**
- ❌ Do NOT allocate memory
- ❌ Do NOT take locks
- ❌ Keep execution under 10 microseconds
- ✅ Only use stack (e.g., stack buffers)
- ✅ Return results immediately

**Acceptance Criteria:**
- Callback compiles
- Returns FWP_ACTION_PERMIT for all packets
- No allocations in the hot path

#### Task 1.3: Register Callout in DLL Entry (Day 2)
**File:** `wfp-callout/src/lib.rs`

```rust
#[no_mangle]
pub extern "system" fn DllMain(
    _module: isize,
    reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> i32 {
    match reason {
        1 => { // DLL_PROCESS_ATTACH
            // Initialize WFP engine
            // Register callout
            // Return TRUE (1) on success
            1
        },
        0 => { // DLL_PROCESS_DETACH
            // Clean up
            1
        },
        _ => 1,
    }
}
```

**Acceptance Criteria:**
- DllMain compiles
- WFP engine initializes on attach
- No crashes on detach

### Build & Test

```powershell
cd C:\Users\SAKSHAM\Wifly

# Build release version (required to convert to .sys)
cargo build -p wfp-callout --release --target x86_64-pc-windows-msvc

# Output: target/x86_64-pc-windows-msvc/release/wfp_callout.dll
```

**Manual Driver Loading (Windows 10/11):**

1. Copy the DLL to a safe location:
   ```powershell
   $dll_path = "C:\Users\SAKSHAM\AppData\Local\netshaper\wfp_callout.dll"
   mkdir -Force (Split-Path $dll_path)
   Copy-Item "target/x86_64-pc-windows-msvc/release/wfp_callout.dll" $dll_path
   ```

2. Register as a service:
   ```powershell
   sc.exe create netshaper-wfp type=kernel start=demand binPath=$dll_path
   ```
   (Exact command may vary; check WDK docs)

3. Load the driver:
   ```powershell
   sc.exe start netshaper-wfp
   ```

4. Check if loaded:
   ```powershell
   # Using WFP tools (if available)
   netsh wfp show state
   ```

5. Unload:
   ```powershell
   sc.exe stop netshaper-wfp
   sc.exe delete netshaper-wfp
   ```

### Branch & PR

```powershell
git checkout -b saksham/milestone-1-wfp-skeleton
# ... make commits ...
git push -u origin saksham/milestone-1-wfp-skeleton
```

Create a PR with:
- **Title:** `[M1] WFP Callout Skeleton — loads and permits packets`
- **Description:**
  - What works: DllMain, WFP engine, callout registration, classify callback
  - What's next: Token bucket logic (Aksh's M2)
  - Testing done: Manual driver loading on Windows (your machine)

---

## Milestone 4: IPC Bridge Integration (3 days shared)

### Goal

Enable end-to-end communication:

```
Kernel (wfp-callout)
    ↓ Named Pipe IPC
Daemon (Aksh's code)
    ↓ Token bucket decision
Kernel (wfp-callout)
    ↓ Allow/Drop packets
```

### Your Role in M4

The daemon (Aksh) will:
- Read packets from the kernel
- Apply token bucket logic
- Send back allow/drop decision

You will:
- Send packet info to the daemon (from classify callback)
- Receive allow/drop decision back
- Apply the decision (permit/block packet)

### IPC Contract (proto/)

**CRITICAL:** Do NOT modify proto/ without Aksh's approval.

The message format is defined in `proto/src/lib.rs`:
```rust
pub struct PacketInfo {
    pub device_id: u64,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub bytes_len: u32,
    pub timestamp_us: u64,
}

pub enum IpcMessage {
    PacketInbound(PacketInfo),
    Decision { allowed: bool },
}
```

**When M2 (Aksh) is ready**, his code will:
- Listen on a named pipe: `\\?\pipe\netshaper-throttle`
- Read PacketInfo
- Send back Decision (allow/drop)

Your M4 work:
- Connect to this pipe from classify callback
- Send PacketInfo for each packet
- Read Decision back
- Apply (FWP_ACTION_PERMIT or FWP_ACTION_BLOCK)

### Implementation Sketch

**In callout.rs:**
```rust
// Global pipe handle (set up in DllMain)
static mut PIPE_HANDLE: Option<PipeClient> = None;

unsafe extern "system" fn classify_callback(...) {
    if let Some(ref pipe) = PIPE_HANDLE {
        // Create packet info
        let info = PacketInfo { /* ... */ };
        
        // Send to daemon
        if let Ok(decision) = pipe.send_and_receive(&info) {
            *action = if decision.allowed {
                FWP_ACTION_PERMIT
            } else {
                FWP_ACTION_BLOCK
            };
        } else {
            // Daemon not running? Default to permit
            *action = FWP_ACTION_PERMIT;
        }
    }
}
```

### Coordination with Aksh

1. **Wait for M2 to finish** (Aksh: 4 days)
2. **Schedule a sync call** to align on:
   - Named pipe name & location
   - Serialization format (bincode)
   - Error handling (what if daemon crashes?)
   - Performance (how long should each query take?)
3. **Create proto-update/ branch** if any changes needed
4. **PR review = both review + approve**

---

## Milestone 5: Tauri Control Panel (4 days)

### Goal

Build a system-tray UI that:
- ✅ Shows enrolled devices
- ✅ Sliders to set bandwidth per device
- ✅ Sends settings to daemon via IPC
- ✅ Displays current bandwidth usage

### UI Components

| Component | Purpose |
|-----------|---------|
| **System Tray Icon** | Minimal footprint, always accessible |
| **Main Window** | Device list, bandwidth sliders, settings |
| **Device Card** | Show device name, current usage, throttle setting |
| **Bandwidth Slider** | Set max bandwidth for a device (Mbps) |

### File Structure (ui/)

```
src/
├── main.rs           # Tauri setup + window management
├── commands.rs       # Tauri commands (IPC to daemon)
├── ipc.rs            # Named pipe client
└── ui/
    ├── App.tsx       # Main React component
    ├── DeviceCard.tsx # Device bandwidth control
    └── Slider.tsx    # Bandwidth slider component

tauri.conf.json       # Tauri config (already scaffolded)
```

### Implementation Plan

#### Day 1: Tauri Setup & Window
- [ ] Configure system-tray in `tauri.conf.json`
- [ ] Create main window (show/hide on tray icon click)
- [ ] Test: Tray icon appears, window opens/closes

#### Day 2: Device List & Cards
- [ ] Create Tauri command: `get_devices()` → calls daemon
- [ ] Create Tauri command: `set_bandwidth(device_id, mbps)`
- [ ] Build React UI with device cards

#### Day 3: Bandwidth Sliders
- [ ] Implement slider component (0–1000 Mbps)
- [ ] Real-time updates to daemon
- [ ] Display current usage (read from daemon)

#### Day 4: Polish & Testing
- [ ] Error handling (daemon crashes)
- [ ] Visual feedback (loading states)
- [ ] Test on Windows 10 & Windows 11

### Key Files to Review

- **tauri.conf.json** — Already has WebView2 setup
- **src/main.rs** — Tauri window setup
- **Invoke commands** — See Tauri docs:
  ```typescript
  import { invoke } from "@tauri-apps/api/tauri";
  const devices = await invoke("get_devices");
  ```

### Branch & PR

```powershell
git checkout -b saksham/milestone-5-control-panel
# ... make commits ...
git push -u origin saksham/milestone-5-control-panel
```

---

## Milestone 6: Installer & MSI Packaging (2 days)

### Goal

- ✅ Create Windows installer (MSI)
- ✅ Auto-register WFP driver
- ✅ Install system-tray UI to %ProgramFiles%
- ✅ Create uninstaller
- ✅ Set up network service auto-start

### Tools

| Tool | Purpose |
|------|---------|
| **cargo-wix** | Create MSI from Cargo.toml |
| **WiX Toolset** | MSI configuration language |

### Setup cargo-wix

```powershell
# Install on your machine
cargo install cargo-wix

# Inside repo
cargo wix new -n netshaper_installer
```

This creates `wix/main.wxs` (MSI config).

### Implementation Plan

#### Day 1: MSI Structure
- [ ] Configure wix/main.wxs
- [ ] Add daemon .exe to package
- [ ] Add wfp-callout .dll to package
- [ ] Add UI executable to package
- [ ] Build: `cargo wix build`

#### Day 2: Custom Actions
- [ ] Register WFP driver on install
- [ ] Register Windows service (daemon)
- [ ] Create uninstall script (cleanup drivers)
- [ ] Test on clean Windows VM (if available)

### Build Manual

```powershell
# From repo root
cargo wix build

# Output: wix/target/wix/netshaper-1.0.0-x64.msi
```

Then you can double-click to install, or test:
```powershell
msiexec /i "netshaper-1.0.0-x64.msi"
msiexec /x "netshaper-1.0.0-x64.msi"  # Uninstall
```

---

## Git Workflow & Branch Management

### Branch Naming

```
main                              # Protected, always stable
├── saksham/milestone-N-*         # Your feature branches
│   ├── saksham/milestone-1-wfp-skeleton
│   ├── saksham/milestone-4-ipc-integration
│   └── saksham/milestone-5-control-panel
└── proto-update/*                # ONLY if you touch proto/
    ├── proto-update/add-new-message-type
    └── proto-update/fix-serialization
```

### For Proto Changes

⚠️ **If you ever need to change `proto/` crate:**

1. **Create a branch:** `proto-update/your-description`
2. **Make changes carefully** (breaking changes = disaster)
3. **Create PR** against `main`
4. **Both Aksh and you must approve** before merge
5. **Coordinate timing:** Make sure Aksh can rebuild his code

### Standard PR Workflow

```powershell
# Create feature branch
git checkout -b saksham/milestone-N-description

# Make changes
cargo build --workspace  # Verify compilation
cargo test               # Run tests

# Commit regularly
git add -A
git commit -m "M{N}: Feature description"

# Push & create PR
git push -u origin saksham/milestone-N-description
```

On GitHub, request review from Aksh. He'll review, comment, and approve to merge.

---

## Testing Strategy

### Unit Tests (Always Run Locally)

```powershell
# Proto tests (IPC serialization)
cargo test -p proto

# UI tests (if you add them)
cargo test -p ui
```

### Build Tests

```powershell
# Verify all crates compile for Windows
cargo build --workspace --target x86_64-pc-windows-msvc

# Release build (what gets packaged)
cargo build --release --workspace
```

### Manual Integration Tests (M4 onward)

**Prerequisites:**
- Aksh's daemon is running
- Your driver is loaded
- Test machine is NOT production

**Test Scenario:**

1. **Start daemon**
   ```powershell
   cargo run -p daemon
   ```

2. **Load driver**
   ```powershell
   cargo build -p wfp-callout --release
   # Then manually load via sc.exe (see M1)
   ```

3. **Generate test traffic** (e.g., download file)
   ```powershell
   # Download a large file, watch bandwidth
   # Expected: Throttling at configured rate
   ```

4. **Check logs** (both daemon & driver must log)
   ```powershell
   # Where are logs? Define in code:
   # - Daemon: stdout or file
   # - Driver: Event Viewer (kernel logs)
   ```

### CI/CD (Automated)

GitHub Actions will:
- Compile `wfp-callout` on `windows-latest`
- Run unit tests on Ubuntu (daemon/proto/crypto)
- Report failures

You don't need to do anything; it runs automatically on PR.

---

## Known Issues & Troubleshooting

### "Access Denied" When Loading .sys

**Problem:** `sc.exe start` fails with ERROR_ACCESS_DENIED

**Solution:**
```powershell
# First verify test signing is ON
bcdedit | findstr testsigning

# If not, enable it:
bcdedit /set testsigning on
shutdown /r /t 0
```

### WFP Callout Crashes (BSOD)

**Problem:** Kernel panic when classify callback runs

**Usual Cause:** Memory allocation or lock in the callback

**Solution:**
- ❌ Remove any `allocations` (Vec, Box, String, etc.)
- ❌ Remove any `locks` (Mutex, RwLock)
- ✅ Use only **stack buffers**
- ✅ Keep callback under **10 microseconds**

**Debug:**
```powershell
# Check Event Viewer for crash dumps
Event Viewer → Windows Logs → System
# Look for kernel crashes with source "netshaper"
```

### Named Pipe: "Pipe Not Found"

**Problem:** Driver tries to connect to daemon pipe, fails

**Solution:**
- **Start daemon FIRST**, then load driver
- **Named pipe path** must match (see proto contract)
- **Default:** `\\?\pipe\netshaper-throttle`

### Build Fails: Missing WFP Headers

**Problem:**
```
error: 'fwpsk.h' not found
```

**Solution:**
```powershell
# Verify WDK installed correctly
dir "C:\Program Files (x86)\Windows Kits\10\Include\*\km\fwpsk.h"

# If not found, reinstall WDK
winget install Microsoft.WindowsDriverKit
```

### WebView2 Missing on Windows 10

**Problem:** UI fails to start on Windows 10

**Solution:**
- Already configured in `tauri.conf.json`:
  ```json
  "webviewInstallMode": { "type": "downloadBootstrapper" }
  ```
- WebView2 will auto-install on first run
- If stuck, manually install: https://developer.microsoft.com/microsoft-edge/webview2/

---

## Communication & Coordination

### With Aksh

| Phase | Topic | Channel |
|-------|-------|---------|
| **M0** | Setup verification | Slack #netshaper-dev |
| **M1–M3** | Independent work | GitHub Issues, async |
| **M4** | IPC contract alignment | ~30-min sync call |
| **M5–M6** | Integration & testing | Daily standup (if available) |

### Proto Changes

**Always inform Aksh before touching `proto/`:**
1. Post in #netshaper-dev with proposed changes
2. Create `proto-update/` branch (not `saksham/*`)
3. Wait for his approval before continuing
4. Both review before merge

---

## Next Immediate Steps

1. ✅ **Read this document** (you are here)
2. ✅ **Clone repo** (done)
3. **Complete M0 Setup:**
   - Enable test signing
   - Install Rust, WDK, Visual Studio, LLVM
   - Run `cargo build --workspace`
   - Create PR for review
4. **Start M1:** WFP skeleton (once Aksh approves M0)

---

## Resources

| Resource | Link |
|----------|------|
| **Windows Filtering Platform** | https://docs.microsoft.com/en-us/windows/win32/fwp/windows-filtering-platform-start-page |
| **Rust windows crate** | https://github.com/microsoft/windows-rs |
| **WDK Docs** | https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk |
| **Tauri Docs** | https://tauri.app/ |
| **cargo-wix (MSI)** | https://github.com/volumetric/cargo-wix |

---

## Quick Reference: Common Commands

```powershell
# Build
cargo build --workspace --target x86_64-pc-windows-msvc
cargo build -p wfp-callout --release

# Test
cargo test -p proto
cargo test --workspace

# Git
git checkout -b saksham/milestone-N-description
git push -u origin saksham/milestone-N-description

# Driver loading (manual)
sc.exe create netshaper-wfp type=kernel start=demand binPath=C:\path\to\dll
sc.exe start netshaper-wfp
sc.exe stop netshaper-wfp

# Package
cargo wix build
```

---

**Last Updated:** April 2, 2026  
**For:** Saksham (Windows Driver & UI Development)  
**Status:** Ready for Milestone 0 Setup

Good luck! 🚀
