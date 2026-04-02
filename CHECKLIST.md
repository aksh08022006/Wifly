# ✅ Wifly M0-M6 Setup & Development Checklist

**For:** Saksham (Windows Driver & UI Development)  
**Date Started:** April 2, 2026  
**Status:** M0 In Progress

---

## 🚀 M0: Repo & Toolchain Setup (Current)

### Prerequisites ✅
- [x] Repository cloned to `C:\Users\SAKSHAM\Wifly`
- [x] Administrator account available
- [x] ~10 GB free disk space
- [x] Internet connection

### Toolchain Installation (Do These Steps)

**Step 1: Run Setup Script as Administrator ⏳**
```powershell
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope CurrentUser -Force
C:\Users\SAKSHAM\Wifly\Setup-M0-Windows.ps1
```

Checklist:
- [ ] PowerShell opened as Administrator
- [ ] Script started successfully
- [ ] Test signing enabled (shows in output)
- [ ] Windows Driver Kit downloaded
- [ ] Visual Studio Build Tools installing
- [ ] LLVM downloading
- [ ] Computer restarted (automatic)

**Step 2: Verify After Restart ⏳**
```powershell
cd C:\Users\SAKSHAM\Wifly
rustc --version          # Should show: rustc 1.94.1
cargo --version          # Should show: cargo 1.94.1
bcdedit | findstr test   # Should show: testsigning    Yes
cargo build -p proto     # Should show: Finished ...
```

Checklist:
- [ ] Rust version verified
- [ ] Cargo version verified
- [ ] Test signing confirmed as ON
- [ ] Proto crate builds successfully
- [ ] All tests pass

**Step 3: Verify Full Workspace ⏳**
```powershell
cargo build --workspace
cargo test -p proto
cargo test -p daemon
cargo test -p crypto
```

Checklist:
- [ ] Workspace builds without errors
- [ ] Proto tests pass
- [ ] Daemon tests pass
- [ ] Crypto tests pass
- [ ] UI builds (may have warnings)

### M0 Completion Checklist ✅
- [ ] All Steps 1-3 above complete
- [ ] No build errors (warnings OK)
- [ ] Test signing enabled
- [ ] WDK installed and verified
- [ ] VS Build Tools installed
- [ ] LLVM installed
- [ ] Can build all crates except wfp-callout (requires driver signing later)

### Create M0 PR
```powershell
git checkout -b saksham/milestone-0-setup
git add -A
git commit -m "Milestone 0: Complete setup, toolchain verified, all tests pass"
git push -u origin saksham/milestone-0-setup
```

Then create PR on GitHub and request review from Aksh

Checklist:
- [ ] Feature branch created
- [ ] Commits pushed
- [ ] PR created on GitHub
- [ ] Waiting for Aksh's review

**M0 Status:** ⏳ In Progress → Expected completion: All checks by April 3, 2026

---

## M1: WFP Callout Skeleton (Next - 3 Days)

### Day 1: WFP Engine Skeleton
- [ ] Read `SAKSHAM_WORKFLOW.md` M1 section thoroughly
- [ ] Create `wfp-callout/src/engine.rs` with WfpEngine struct
- [ ] Implement `WfpEngine::new()` - initializes FWP engine
- [ ] Implement `WfpEngine::register_callout()` - registers callout
- [ ] Implement `WfpEngine::close()` - cleanup
- [ ] Code compiles (errors OK, warnings OK)
- [ ] Unit tests pass

### Day 1-2: Classify Callback
- [ ] Create `wfp-callout/src/callout.rs`
- [ ] Implement `classify_callback()` function
- [ ] Returns `FWP_ACTION_PERMIT` for all packets
- [ ] No memory allocations in callback (⚠️ CRITICAL)
- [ ] No locks in callback
- [ ] Callback compiles without errors

### Day 2: DLL Entry Point
- [ ] Create `wfp-callout/src/lib.rs` with DllMain
- [ ] Initialize WFP engine on DLL_PROCESS_ATTACH
- [ ] Register callout in DllMain
- [ ] Cleanup on DLL_PROCESS_DETACH
- [ ] DllMain returns 1 (SUCCESS) on attach/detach

### M1 Verification
- [ ] `cargo build -p wfp-callout --release` succeeds
- [ ] Output: `target/x86_64-pc-windows-msvc/release/wfp_callout.dll`
- [ ] Manual driver loading test:
  - [ ] `sc.exe create netshaper-wfp type=kernel start=demand binPath=C:\path`
  - [ ] `sc.exe start netshaper-wfp` succeeds
  - [ ] No BSOD
  - [ ] `sc.exe stop netshaper-wfp` succeeds

### M1 PR
```powershell
git checkout -b saksham/milestone-1-wfp-skeleton
# ... implement code ...
git add -A
git commit -m "Milestone 1: WFP Callout Skeleton - loads, permits all packets"
git push -u origin saksham/milestone-1-wfp-skeleton
```

Checklist:
- [ ] All M1 code complete
- [ ] Compiles without errors
- [ ] Driver loads successfully
- [ ] PR created and waiting for review

**M1 Status:** ⏳ Not Started → Expected completion: 3 days after M0

---

## M2-M3: (Aksh's Work - Daemon & Crypto)

During this phase, you can:
- [ ] Read daemon and crypto code to understand architecture
- [ ] Review IPC contract in `proto/src/lib.rs`
- [ ] Prepare for M4 integration
- [ ] Set up test environment for M4

**Expected Duration:** 4 days (Aksh: daemon) + 2 days (Aksh: crypto)  
**Your Involvement:** Planning, review, questions

---

## M4: IPC Bridge Integration (3 Days - Shared with Aksh)

### Coordination
- [ ] Schedule sync call with Aksh (30 min)
- [ ] Review IPC contract together
- [ ] Agree on pipe name, serialization, error handling
- [ ] Understand token bucket response format

### Your M4 Work

**Day 1: Named Pipe Client in Driver**
- [ ] Create `wfp-callout/src/ipc.rs`
- [ ] Implement pipe client connection
- [ ] Send `PacketInfo` struct to daemon
- [ ] Receive `Decision` response back

**Day 2: Integrate with Classifier**
- [ ] Modify classify callback to:
  - [ ] Extract packet info (IP, bytes, device ID)
  - [ ] Send via named pipe to daemon
  - [ ] Wait for decision
  - [ ] Apply: FWP_ACTION_PERMIT or FWP_ACTION_BLOCK

**Day 3: Testing**
- [ ] Start daemon first: `cargo run -p daemon`
- [ ] Load driver: `sc.exe start netshaper-wfp`
- [ ] Generate test traffic (download file)
- [ ] Observe throttling at set bandwidth
- [ ] Check daemon logs for packet processing
- [ ] Verify no BSOD under load

### M4 PR
- [ ] IPC integration code complete
- [ ] End-to-end test passes
- [ ] Both Aksh and you review
- [ ] PR ready for merge

**M4 Status:** ⏳ Not Started → Expected start date: ~2 weeks after M0

---

## M5: Tauri Control Panel (4 Days)

### Day 1: Tauri Window Setup
- [ ] Configure system-tray in `tauri.conf.json`
- [ ] Create main window (show/hide on tray click)
- [ ] Test window appears/closes properly

### Day 2: Device List UI
- [ ] Create Tauri command: `get_devices()`
- [ ] Create React components: `App.tsx`, `DeviceCard.tsx`
- [ ] Display list of enrolled devices
- [ ] Show current bandwidth usage per device

### Day 3: Bandwidth Sliders
- [ ] Create `Slider.tsx` component (0-1000 Mbps)
- [ ] Implement `set_bandwidth(device_id, mbps)` command
- [ ] Real-time updates to daemon via IPC
- [ ] Visual feedback while updating

### Day 4: Polish
- [ ] Error handling (daemon crashes)
- [ ] Loading states
- [ ] Test on Windows 10 & 11
- [ ] Verify WebView2 auto-install works

### M5 PR
- [ ] Complete working UI
- [ ] All buttons/sliders functional
- [ ] PR ready for merge

**M5 Status:** ⏳ Not Started → Expected start date: ~3 weeks after M0

---

## M6: Installer & MSI Packaging (2 Days)

### Day 1: MSI Configuration
- [ ] Install cargo-wix: `cargo install cargo-wix`
- [ ] Create WiX config: `cargo wix new -n netshaper_installer`
- [ ] Add all binaries to package (daemon, UI, driver DLL)
- [ ] Configure default install path: `%ProgramFiles%\NetShaper`

### Day 2: Custom Actions & Testing
- [ ] Register WFP driver on install
- [ ] Register Windows service for daemon
- [ ] Uninstall script (cleanup)
- [ ] Build MSI: `cargo wix build`
- [ ] Test install/uninstall on clean VM (if available)

### M6 PR
- [ ] MSI builds successfully
- [ ] Install/uninstall works
- [ ] Driver auto-registers
- [ ] Service auto-starts
- [ ] Ready for production

**M6 Status:** ⏳ Not Started → Expected start date: ~4 weeks after M0

---

## 📚 Documentation Files

Check these as you progress:

- [x] `START_HERE.md` - Quick action items
- [x] `README_M0_SETUP.md` - Detailed M0 setup
- [x] `SAKSHAM_WORKFLOW.md` - Complete M0-M6 guide (READ THIS!)
- [ ] `M0_SETUP_STATUS.md` - Technical details
- [ ] `Setup-M0-Windows.ps1` - Automated setup

---

## 🔗 Key Git Workflow

### Branch Naming
```
main                          # Protected branch
└── saksham/milestone-N-*      # Your feature branches
    ├── saksham/milestone-0-setup
    ├── saksham/milestone-1-wfp-skeleton
    ├── saksham/milestone-4-ipc-integration
    ├── saksham/milestone-5-control-panel
    └── saksham/milestone-6-msi-installer

proto-update/*                # If you modify proto/ crate
```

### PR Process
1. Create branch: `git checkout -b saksham/milestone-N-*`
2. Make changes: `git add -A && git commit -m "..."`
3. Push: `git push -u origin saksham/milestone-N-*`
4. Create PR on GitHub
5. Request review from Aksh
6. Address feedback
7. Merge when approved

**⚠️ CRITICAL:** If you touch `proto/`, create `proto-update/*` branch and BOTH must review before merge!

---

## 📞 Communication Checklist

- [ ] Slack #netshaper-dev created (coordinate with Aksh)
- [ ] Stand-ups scheduled (3x weekly)
- [ ] GitHub Issues for blockers
- [ ] PRs for all code changes
- [ ] Proto change protocol established

---

## 🎯 Timeline Summary

| Milestone | Duration | Your Role | Status |
|-----------|----------|-----------|--------|
| **M0** | 1 day | Setup toolchain | ⏳ In Progress |
| **M1** | 3 days | WFP skeleton | ⏳ Waiting for M0 |
| **M2-M3** | 6 days | Wait (Aksh working) | ⏳ Not started |
| **M4** | 3 days | IPC integration | ⏳ Waiting for M2-M3 |
| **M5** | 4 days | Tauri UI | ⏳ Waiting for M4 |
| **M6** | 2 days | MSI installer | ⏳ Waiting for M5 |
| **TOTAL** | ~19 days | Active development | ⏳ In progress |

---

## 💾 Saving This Checklist

After each milestone:
1. Update this checklist
2. Commit: `git add CHECKLIST.md && git commit -m "Update checklist after M{N}"`
3. Push changes

---

## Notes

```
Add your personal notes here as you progress:

M0 Notes:
- 


M1 Notes:
- 


M4 Notes:
- 


M5 Notes:
- 


M6 Notes:
- 
```

---

**Good luck! You've got this! 🚀**

*Checklist for Saksham - NetShaper / Wifly Project*  
*Last Updated: April 2, 2026*
