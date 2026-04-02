# 🚀 Wifly M0 Setup - READY FOR FINAL STEPS

## Status: ~70% Complete ✅

Your Wifly development environment is **almost ready**! Here's what's been done and what you need to do.

---

## ✅ What's Already Done

1. **Repository Cloned** ✅
   - Location: `C:\Users\SAKSHAM\Wifly`
   - All crates ready: proto/, daemon/, crypto/, wfp-callout/, ui/

2. **Rust Installed** ✅
   - Rustup: 1.29.0
   - Rust: 1.94.1
   - MSVC toolchain configured
   - Cargo working

3. **Configuration Fixed** ✅
   - Removed macOS cross-compile settings from `.cargo/config.toml`
   - Updated Tauri dependencies in `ui/Cargo.toml`

4. **Documentation Created** ✅
   - `SAKSHAM_WORKFLOW.md` - Complete M0-M6 development guide
   - `Setup-M0-Windows.ps1` - Automated setup script
   - `M0_SETUP_STATUS.md` - Technical status report

---

## ⚠️ What You Need To Do (2 Simple Steps)

### Step 1: Run Setup Script as Administrator (5 min)

Open **PowerShell as Administrator** and run:

```powershell
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force
C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1
```

This will:
- ✅ Enable test signing (REQUIRES RESTART)
- ✅ Verify/install Rust
- ✅ Install Windows Driver Kit (WDK)
- ✅ Install Visual Studio 2022 Build Tools with C++ workload
- ✅ Install LLVM
- ✅ Test the build

### Step 2: Restart & Verify (5 min)

```powershell
# After restart, open normal PowerShell (non-admin)
cd C:\Users\SAKSHAM\Wifly

# Verify everything works
rustc --version
cargo --version
cargo build -p proto
```

Expected output:
```
rustc 1.94.1 (...)
cargo 1.94.1 (...)
   Compiling proto v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)
```

---

## 📋 Detailed Instructions (Step-by-Step)

### How to Run PowerShell as Administrator

**Windows 11:**
1. Search for "PowerShell" in Start menu
2. Right-click "Windows PowerShell"
3. Click "Run as administrator"
4. Click "Yes" when prompted

**Windows 10:**
1. Press `Win + R`
2. Type: `powershell`
3. Press `Ctrl + Shift + Enter`
4. Click "Yes"

### Run the Setup Script

Once in admin PowerShell, copy this entire command at once:

```powershell
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force; C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1
```

Press Enter. The script will:
- Show status messages with colors
- Download and install Visual Studio Build Tools (~500 MB, may take 10-20 min)
- Enable test signing (marks for restart)
- Run a build test
- **Automatically restart your computer**

### After Restart

1. **Wait for restart** (automatic)
2. **Open PowerShell** (normal, non-admin)
3. **Test build:**
   ```powershell
   cd C:\Users\SAKSHAM\Wifly
   cargo build -p proto
   ```

If you see `Finished dev [unoptimized + debuginfo] target(s) in X.XXs`, M0 is complete! ✅

---

## 🔍 What Gets Installed

| Component | Size | Time | Purpose |
|-----------|------|------|---------|
| Rust (already installed) | ~2 GB | Already done | Compiler & build system |
| VS Build Tools | ~500 MB | 10-20 min | C++ compiler (link.exe) |
| Windows SDK | ~100 MB | 5 min | WFP driver headers |
| LLVM | ~400 MB | 5-10 min | Faster linker |
| **Total** | ~1 GB | 20-35 min | Everything needed |

---

## 🎯 Next: Milestone 1 (WFP Callout)

After M0 is complete:

1. **Create feature branch:**
   ```powershell
   cd C:\Users\SAKSHAM\Wifly
   git checkout -b saksham/milestone-1-wfp-skeleton
   ```

2. **Read the development guide:**
   ```powershell
   code SAKSHAM_WORKFLOW.md
   # Or just open in VS Code and go to "Milestone 1: WFP Callout Skeleton" section
   ```

3. **Start coding M1:**
   - Task 1.1: WFP Engine skeleton (Day 1)
   - Task 1.2: Classify callback (Day 1-2)
   - Task 1.3: DLL entry point (Day 2)

See `SAKSHAM_WORKFLOW.md` for complete M1 implementation details!

---

## ✨ Your Complete Development Guide

Everything you need is in **`SAKSHAM_WORKFLOW.md`**:
- M0 detailed setup (current step)
- M1-M6 complete implementation guides
- Testing strategy
- Git workflow
- Troubleshooting
- Known issues & solutions
- Quick reference commands

**Read this document for:**
- Architecture overview
- Your exact responsibilities
- Code examples for each milestone
- How to coordinate with Aksh (daemon developer)
- Driver loading instructions
- UI implementation guide
- MSI packaging guide

---

## 📞 Troubleshooting

### Problem: "Script cannot be run because it contains a #requires statement"

**Solution:** Open PowerShell as Administrator (see instructions above)

### Problem: Script gets stuck downloading VS Build Tools

**Solution:** The download is large (~500 MB). It may take 10-20 minutes. Wait patiently.

### Problem: "bcdedit: Access Denied"

**Solution:** Make sure you opened PowerShell as Administrator (Run as Administrator)

### Problem: After restart, "cargo" not found

**Solution:** 
```powershell
# Refresh PATH in new PowerShell window
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
cargo --version
```

### Problem: Build still fails after completing setup

**Solution:**
1. Check test signing is ON: `bcdedit | findstr testsigning` (should show `Yes`)
2. Check MSVC linker exists: `dir "C:\Program Files*\Microsoft Visual Studio\2022\*\VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe"` (should find it)
3. Try running setup script again (some steps may have failed)

---

## 📊 Current Status

```
M0 Setup Status:
├── ✅ Repository: Cloned
├── ✅ Rust: Installed & configured
├── ✅ Build config: Fixed
├── ✅ Documentation: Complete
├── ⏳ Test signing: Pending (needs admin script)
├── ⏳ WDK: Pending (needs admin script)
├── ⏳ VS Build Tools: Pending (needs admin script)
├── ⏳ LLVM: Pending (needs admin script)
└── ⏳ Build verification: Pending (after step 1)
```

**Current Blocker:** Administrator privileges needed to run setup script

**Time to Complete:**
- Run script: ~5 minutes
- Installations: ~20-30 minutes
- Restart: ~5 minutes
- Verify: ~2 minutes
- **Total: ~35 minutes**

---

## 💡 Quick Summary

| What | Status | What To Do |
|------|--------|-----------|
| Repo ready? | ✅ Yes | Nothing - already cloned |
| Rust ready? | ✅ Yes | Nothing - already installed |
| Documentation? | ✅ Yes | Read SAKSHAM_WORKFLOW.md |
| Toolchain complete? | ⏳ 70% | Run Setup-M0-Windows.ps1 as admin |
| Ready for M1? | ⏳ Almost | Complete setup script + restart |

---

## Next Action

👉 **Open PowerShell as Administrator and run:**

```powershell
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force; C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1
```

The script does everything else automatically! ✨

---

**Ready?** Let's go! 🚀

---

*Setup Documentation for Saksham  
NetShaper / Wifly Project  
April 2, 2026*
