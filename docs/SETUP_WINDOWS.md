# Setup Guide for Saksham (Windows — Driver & UI Development)

This guide gets your Windows PC ready to:
1. ✅ Develop wfp-callout/ui crates (kernel driver + Tauri UI)
2. ✅ Test WFP functionality and load .sys files
3. ✅ Build Windows executables
4. ⚠️ Run daemon/proto tests (optional, focus on driver work)

## Prerequisites

- Windows 10 or Windows 11
- Administrator account
- ~10 GB free disk space
- Internet connection

## Step 1: Enable Test Signing (Required for .sys Loading)

This allows unsigned kernel drivers to load. Required for development.

**⚠️ WARNING:** This reduces security. Use only on dev machines.

### Option A: GUI (Windows 11)

1. Open **Settings** → **System** → **Developer settings**
2. Toggle **"Enable Test Signing"** to ON
3. Restart when prompted

### Option B: Command Line (Windows 10/11)

Open **PowerShell as Administrator:**

```powershell
bcdedit /set testsigning on
shutdown /r /t 0
```

Wait for restart. Verify:
```powershell
bcdedit | findstr testsigning
# Should output: testsigning    Yes
```

## Step 2: Install Rust

```powershell
winget install Rustlang.Rustup
rustup toolchain install stable-x86_64-pc-windows-msvc
```

Verify:
```powershell
rustc --version  # Should print: rustc 1.75.x (or later)
```

## Step 3: Install Windows Driver Kit (WDK)

The WDK provides WFP headers and build tools.

```powershell
winget install Microsoft.WindowsDriverKit
```

Verify headers exist:
```powershell
dir "C:\Program Files (x86)\Windows Kits\10\Include\*\km\fwpsk.h"
# Should find fwpsk.h
```

## Step 4: Install Visual Studio Build Tools (C++ Workload)

WFP development requires MSVC compiler.

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

During installation, select:
- ✅ **Desktop development with C++**
- ✅ **Windows 10 SDK** (or Windows 11 SDK)

Or from command line:
```powershell
vs_buildtools.exe ^
  --add "Microsoft.VisualStudio.Workload.NativeDesktop" ^
  --add "Microsoft.VisualStudio.Component.Windows10SDK" ^
  --quiet --norestart
```

Verify MSVC is available:
```powershell
cl.exe /help
# Should display compiler options
```

## Step 5: Install LLVM (Optional but Recommended)

Makes linking faster:

```powershell
winget install LLVM.LLVM
```

## Step 6: Clone the Repository

```powershell
git clone <repo_url> netshaper
cd netshaper
```

## Step 7: Test Build

From PowerShell in the netshaper directory:

```powershell
# Build wfp-callout (kernel driver)
cargo build -p wfp-callout --target x86_64-pc-windows-msvc

# Build daemon for Windows
cargo build -p daemon --target x86_64-pc-windows-msvc

# Build UI (Tauri)
cargo build -p ui --target x86_64-pc-windows-msvc
```

Expected result:
- ✅ No compilation errors
- ✅ Produces `.dll` file in `target/x86_64-pc-windows-msvc/debug/`

## Step 8: Run Tests

```powershell
# Test proto (shared IPC types)
cargo test -p proto

# Test daemon (optional — Aksh's responsibility)
cargo test -p daemon

# Build all (includes wfp-callout)
cargo build --workspace
```

## Step 9: Set Up Windows Driver Testing

Create a directory for driver testing:

```powershell
mkdir "C:\NetShaper"
mkdir "C:\NetShaper\drivers"
```

Copy the compiled .sys file:
```powershell
copy target\x86_64-pc-windows-msvc\debug\wfp_callout.dll `
      C:\NetShaper\drivers\netshaper_wfp.sys
```

## Daily Workflow

### Developing WFP Callout

```powershell
# Create a branch
git checkout -b saksham/milestone-N-description

# Make changes
notepad wfp-callout/src/engine.rs

# Build
cargo build -p wfp-callout --target x86_64-pc-windows-msvc --release

# If loading driver for testing:
$dll = "target\x86_64-pc-windows-msvc\release\wfp_callout.dll"
copy $dll C:\NetShaper\drivers\netshaper_wfp.sys

# Test loading (see below)
sc.exe create netshaper-wfp type= kernel start= demand `
       binPath= C:\NetShaper\drivers\netshaper_wfp.sys

# View results
Get-EventLog -LogName System -Source netshaper-wfp | select -Last 5
```

### Developing Tauri UI

```powershell
# Build
cargo build -p ui --target x86_64-pc-windows-msvc

# Run in dev mode
cargo tauri dev  # (if frontend scaffold exists)
```

### Changing proto Crate

⚠️ **CRITICAL:** Coordinate with Aksh!

```powershell
git checkout -b proto-update/description
# Make changes
cargo test -p proto
git push
# → Request both reviews before merging
```

## Loading & Testing the .sys Driver

### Manual Load (for testing)

```powershell
# Elevated PowerShell required

# Create service entry
sc.exe create netshaper-wfp type= kernel start= demand `
       binPath= C:\NetShaper\drivers\netshaper_wfp.sys

# Load driver
sc.exe start netshaper-wfp

# View logs
Get-EventLog -LogName System -Source netshaper-wfp -Newest 10

# Unload driver
sc.exe stop netshaper-wfp

# Remove service
sc.exe delete netshaper-wfp
```

### Troubleshooting .sys Loading

| Error | Solution |
|-------|----------|
| "Access Denied" | Run PowerShell as Administrator |
| "Test Signing is Off" | Run `bcdedit /set testsigning on` and restart |
| "Module Not Found" | Ensure .dll was copied correctly |
| BSOD on start | Check classify callback — likely paged function at DISPATCH_LEVEL |
| Event Viewer shows errors | Review the error code and check WFP docs |

### View Event Logs

```powershell
# Open Event Viewer
eventvwr.msc

# Navigate: Windows Logs > System
# Look for sources: "netshaper-wfp", "WFP"
```

## GitHub Configuration

### SSH Key Setup

```powershell
# Generate SSH key
ssh-keygen -t ed25519 -C "saksham@example.com"

# Add to SSH agent
Get-Service ssh-agent | Start-Service
ssh-add $env:USERPROFILE\.ssh\id_ed25519

# Copy public key
type $env:USERPROFILE\.ssh\id_ed25519.pub | Set-Clipboard

# Paste into GitHub Settings → SSH Keys
```

## VS Code Extensions (Recommended)

- **Rust-analyzer:** `rust-lang.rust-analyzer`
- **CodeLLDB:** `vadimcn.vscode-lldb`
- **GitLens:** `eamodio.gitlens`
- **Windows-like:** `ms-vscode.makefile-tools` (for build automation)

## Performance Profiling Tools (Optional)

- **Windows Performance Toolkit (WPT):** Included with WDK for kernel tracing
- **Event Tracing:** Use `logman` CLI for custom ETW traces
- **Task Manager:** Use Resource Monitor for real-time network stats

## Integration with Aksh

### Receiving daemon.exe from Aksh

Aksh will send you `daemon.exe` (cross-compiled from macOS). Place it here:

```powershell
C:\NetShaper\daemon.exe
```

Run it:
```powershell
C:\NetShaper\daemon.exe --log-level debug
```

### Sending .sys to Aksh for VM Testing

If Aksh wants to test on a Windows VM:

```powershell
# Build release .sys
cargo build -p wfp-callout --target x86_64-pc-windows-msvc --release

# Send file
scp target\x86_64-pc-windows-msvc\release\wfp_callout.dll `
    aksh@dev-mac:/path/to/wfp_callout.sys
```

---

**You're ready!** Proceed to Milestone 0 setup with Aksh.
