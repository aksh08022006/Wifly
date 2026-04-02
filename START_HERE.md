# 🎯 START HERE - Saksham's Wifly Setup

You're **70% done**. Just 2 steps to complete M0! ⚡

---

## What You Have Now ✅

- ✅ Repository cloned
- ✅ Rust installed
- ✅ All documentation ready
- ✅ Automated setup script created

---

## What You Need To Do (5 minutes of work + 30 min auto installation)

### Step 1️⃣: Run the Setup Script (Has Admin Powers Built-In)

Open **PowerShell as Administrator** and copy-paste this exact command:

```powershell
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force; C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1
```

That's it! The script will:
- Enable test signing ✅
- Install Windows Driver Kit ✅
- Install Visual Studio Build Tools ✅
- Install LLVM ✅
- Test the build ✅
- **Auto-restart** your computer ✅

⏱️ Takes ~30 minutes total (mostly automatic downloads and installation)

### Step 2️⃣: After Restart, Verify Everything Works

Open normal PowerShell (non-admin) and run:

```powershell
cd C:\Users\SAKSHAM\Wifly
cargo build -p proto
```

Should see: `Finished dev [unoptimized + debuginfo] target(s) in X.XXs` ✅

**Done!** M0 is complete. 🎉

---

## 📖 Your Complete Development Guides

After M0 setup is done, read these:

1. **`SAKSHAM_WORKFLOW.md`** ⭐ START HERE FOR DEVELOPMENT
   - M0: Complete setup guide (20 min read)
   - M1: WFP Callout implementation (3 days work)
   - M4: IPC Integration (3 days work)
   - M5: Tauri UI (4 days work)
   - M6: MSI Installer (2 days work)
   - Testing strategy
   - Git workflow
   - Troubleshooting

2. **`README_M0_SETUP.md`** - Detailed setup with troubleshooting
3. **`M0_SETUP_STATUS.md`** - Technical status & what was done
4. **`Setup-M0-Windows.ps1`** - The automated setup script

---

## 🔑 Quick Admin PowerShell Guide

### How to Open as Admin

**Windows 11:**
- Search "PowerShell" → Right-click → "Run as administrator"

**Windows 10:**
- Press `Win + R` → Type `powershell` → Press `Ctrl + Shift + Enter`

---

## ⏭️ After M0 Completes

1. Read `SAKSHAM_WORKFLOW.md` (especially M1 section)
2. Start M1: WFP Callout Skeleton (3 days)
3. Create PR when done: `git checkout -b saksham/milestone-1-wfp-skeleton`

---

## 💬 Summary

| What | Where |
|------|-------|
| 🎯 **Setup Instructions** | This file (you're reading it!) |
| 📋 **Full Development Guide** | `SAKSHAM_WORKFLOW.md` |
| 🔧 **Automated Setup Script** | `Setup-M0-Windows.ps1` (run as admin) |
| 📊 **Technical Status** | `M0_SETUP_STATUS.md` |

---

## 🚀 Your Next Action (Choose One)

### If you want to complete M0 RIGHT NOW:
1. Open PowerShell as Admin
2. Run: `Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force; C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1`
3. Wait for restart + completion
4. Verify: `cd C:\Users\SAKSHAM\Wifly && cargo build -p proto`

### If you want to read documentation first:
- Open `SAKSHAM_WORKFLOW.md` and read M0 section
- Then come back and run the setup script

---

## ✨ What Makes This Easy

The setup script does **everything for you**:
- ✅ Enables test signing (required for kernel development)
- ✅ Installs WDK (Windows Driver Kit)
- ✅ Installs VS Build Tools with C++ workload
- ✅ Installs LLVM
- ✅ Automatically restarts when needed
- ✅ Tests the build to confirm everything works

You just run one command as admin. Everything else is automatic!

---

## 🎓 What is M0?

Milestone 0 is the "setup phase" where you get:
- Toolchain working (Rust, MSVC, WDK)
- Test signing enabled (for loading unsigned drivers)
- Build system verified (can compile code)

Once M0 is done, you're ready to start M1: Building the actual WFP kernel driver!

---

**Questions?** See the detailed guides above. ☝️

**Ready?** Open PowerShell as Admin and run the setup script! 🚀

---

*For: Saksham - Windows Driver & UI Developer  
Project: NetShaper / Wifly  
Date: April 2, 2026*
