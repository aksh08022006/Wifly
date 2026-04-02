# GETTING STARTED — Quick Reference

This is your entry point. Read this first, then dive into the specific docs.

## 📋 What Was Just Set Up

A complete Rust Cargo workspace for **NetShaper** — a Windows kernel-level bandwidth controller.

### 5 Crates (Already Scaffolded)

```
proto/          → Shared IPC types (the contract)
daemon/         → Userspace bandwidth control (Aksh's focus)
crypto/         → Device enrollment TLS server (Aksh's focus)
wfp-callout/    → Windows kernel driver (Saksham's focus)
ui/             → Tauri system-tray app (Saksham's focus)
```

All files are in `/Users/akshkaushik/Desktop/Waifu/netshaper/`

## ✅ What's Ready Now

- ✅ Cargo workspace with all dependencies declared
- ✅ Proto crate with full IPC message contract + serialization tests
- ✅ Daemon skeleton with token bucket stub + unit tests  
- ✅ Crypto skeleton with cert/handshake modules
- ✅ WFP callout skeleton with engine.rs wrapper
- ✅ UI skeleton with Tauri setup
- ✅ GitHub Actions CI/CD workflows (cross-platform builds)
- ✅ Documentation for both developers (SETUP_MAC.md, SETUP_WINDOWS.md)
- ✅ Architecture guide (ARCHITECTURE.md)
- ✅ Git workflow guide (GIT_WORKFLOW.md)
- ✅ First commit already in place

## 🚀 Next Steps (Before Any Coding)

### For Aksh (macOS Development)

1. **Read:** [docs/SETUP_MAC.md](docs/SETUP_MAC.md)
2. **Do:** Follow the 9-step setup (Rust, xwin, cross-compile test)
3. **Verify:** `cargo build --workspace --exclude wfp-callout --target x86_64-pc-windows-msvc` succeeds
4. **Then:** You're ready for Milestone 1

### For Saksham (Windows Development)

1. **Read:** [docs/SETUP_WINDOWS.md](docs/SETUP_WINDOWS.md)
2. **Do:** Follow the 9-step setup (Rust, WDK, test signing, MSVC)
3. **Verify:** `cargo build --workspace` succeeds
4. **Then:** You're ready for Milestone 1

### Both Together

- Schedule a 30-minute sync call
- **Share GitHub repo:** https://github.com/aksh08022006/Wifly
- Verify both setups work
- Review [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to align on design
- Agree on branch naming + Git workflow

## 📚 Documentation Structure

```
docs/
├── ARCHITECTURE.md        ← Understand the 5-layer design
├── GIT_WORKFLOW.md        ← How we branch/review/merge
├── SETUP_MAC.md           ← Aksh's setup checklist
├── SETUP_WINDOWS.md       ← Saksham's setup checklist
└── TESTING.md             ← (To be created) Test strategy
```

## 🔄 Development Workflow

```
main (protected branch)
  ↓
Create feature branch
  ├── aksh/milestone-N-*       (Aksh's work)
  ├── saksham/milestone-N-*    (Saksham's work)
  └── proto-update/*           (Shared, needs both reviews)
  ↓
Make changes + test locally
  ↓
Push branch
  ↓
Create PR (GitHub Actions CI runs)
  ↓
Peer review (request teammate)
  ↓
Address feedback
  ↓
Merge when approved + CI passes
  ↓
Delete branch
```

## 🎯 Milestone Sequence

| Milestone | Owner | Timeline | Goal |
|-----------|-------|----------|------|
| 0 | Both | Today | Toolchain setup + repo shared |
| 1 | Saksham | Days 1-3 | WFP callout loads and intercepts packets |
| 2 | Aksh | Days 4-7 | Daemon applies token bucket rate limiting |
| 3 | Aksh | Days 8-9 | Device enrollment TLS server working |
| 4 | Both | Days 10-12 | End-to-end: kernel → daemon → kernel |
| 5 | Saksham | Days 13-16 | Tauri UI sliders control bandwidth |
| 6 | Saksham | Days 17-18 | MSI installer + service registration |

## 🛠️ Quick Commands Reference

### Aksh (macOS)

```bash
# Test protocol definitions
cargo test -p proto

# Test daemon code
cargo test -p daemon

# Test crypto code
cargo test -p crypto

# Cross-compile daemon for Windows
cargo build -p daemon --target x86_64-pc-windows-msvc

# Check for warnings
cargo clippy
```

### Saksham (Windows)

```bash
# Build kernel driver
cargo build -p wfp-callout --target x86_64-pc-windows-msvc

# Build all (including daemon for Windows)
cargo build --workspace

# Run all tests
cargo test --workspace

# Load driver for testing
sc.exe create netshaper-wfp type= kernel start= demand binPath= C:\path\to\netshaper_wfp.dll
```

### Both

```bash
# Check git status
git status

# Create new feature branch
git checkout -b aksh/milestone-2-bucket or saksham/milestone-1-engine

# Commit changes
git add . && git commit -m "feat(daemon): implement bucket refill"

# Push to remote
git push origin aksh/milestone-2-bucket

# Pull latest main
git checkout main && git pull origin main
```

## ⚠️ Critical Rules

### 1. Proto Crate is Sacred

**Changes to `proto/src/lib.rs` REQUIRE both Aksh and Saksham to review.**

Breaking the IPC contract causes silent data corruption. Use branch name `proto-update/*` and add both as reviewers.

### 2. Main Branch Always Compiles

Don't push directly to main. Use feature branches and PRs. GitHub Actions must pass before merge.

### 3. Unsafe Code Must Have Comments

Every `unsafe` block must have a `// SAFETY: ...` comment explaining which memory invariant it upholds.

Example:
```rust
// SAFETY: FwpmEngineOpen0 initializes engine_handle with a valid pointer.
// We own the handle for the lifetime of this WfpEngine struct, and Drop ensures cleanup.
let result = unsafe { FwpmEngineOpen0(..., &mut engine_handle) };
```

### 4. Tests Before Features

Write tests as you code, not after. Unit tests should pass on local machine before pushing.

## 📞 Communication Channels

- **Synchronous:** Discord/Slack #netshaper-dev (real-time issues)
- **Asynchronous:** GitHub Issues (tracking work)
- **Code Review:** GitHub PR comments (documented decisions)
- **Stand-ups:** 3× weekly (TBD time)

## 🆘 Troubleshooting Quick Links

### Aksh (macOS)

- [xwin download fails?](docs/SETUP_MAC.md#troubleshooting)
- [lld-link not found?](docs/SETUP_MAC.md#troubleshooting)
- [Cross-compile errors?](docs/SETUP_MAC.md#troubleshooting)

### Saksham (Windows)

- [Test signing not enabled?](docs/SETUP_WINDOWS.md#enable-test-signing)
- [WDK headers missing?](docs/SETUP_WINDOWS.md#step-3-install-windows-driver-kit)
- [MSVC not found?](docs/SETUP_WINDOWS.md#step-4-install-visual-studio-build-tools)
- [Driver won't load?](docs/SETUP_WINDOWS.md#loading--testing-the-sys-driver)

### Both

- [Git conflicts on merge?](docs/GIT_WORKFLOW.md#if-merge-conflicts)
- [How to squash commits?](docs/GIT_WORKFLOW.md#i-want-to-squash-multiple-commits-before-pr)
- [Proto merge conflicts?](docs/GIT_WORKFLOW.md#proto-changed-on-main-i-need-to-merge)

## 📖 Suggested Reading Order

1. **This file** (you are here!)
2. **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** — Understand the layers
3. **Your setup guide** — [SETUP_MAC.md](docs/SETUP_MAC.md) or [SETUP_WINDOWS.md](docs/SETUP_WINDOWS.md)
4. **[GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md)** — How to collaborate
5. **[README.md](README.md)** — Full reference

## 🎓 Learning Resources

### Windows Filtering Platform (WFP)

- [Microsoft WFP Documentation](https://docs.microsoft.com/en-us/windows/win32/fwp/windows-filtering-platform-start-page)
- [WFP Callout Drivers](https://docs.microsoft.com/en-us/windows-hardware/drivers/network/using-packet-injection)

### Rust Windows Bindings

- [windows-rs on GitHub](https://github.com/microsoft/windows-rs)
- [windows-rs Documentation](https://docs.rs/windows/)

### Token Bucket Algorithm

- [Wikipedia](https://en.wikipedia.org/wiki/Token_bucket)
- [Rate Limiting Patterns](https://en.wikipedia.org/wiki/Rate_limiting)

### Tauri

- [Official Docs](https://tauri.app/)
- [System Tray Guide](https://tauri.app/en/docs/api/js/modules/tray/)

### Rust Best Practices

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

---

## 📝 Repository Structure at a Glance

```
netshaper/                    ← Root (git repo)
├── Cargo.toml               ← Workspace manifest
├── Cargo.lock               ← (Will be auto-generated)
├── README.md                ← Project overview
├── .cargo/config.toml       ← Cross-compile settings
├── .github/workflows/       ← CI/CD
│   ├── build-wfp.yml        ← Windows build (Saksham's driver)
│   └── test-daemon.yml      ← Cross-platform tests
├── docs/                    ← All documentation
│   ├── ARCHITECTURE.md
│   ├── GIT_WORKFLOW.md
│   ├── SETUP_MAC.md
│   ├── SETUP_WINDOWS.md
│   └── TESTING.md (to do)
├── proto/                   ← SHARED IPC types
│   ├── Cargo.toml
│   └── src/lib.rs           ← PacketMetadata, BandwidthUpdate, etc.
├── daemon/                  ← AKSH: Backend service
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── bucket.rs        ← Token bucket algorithm
│       ├── device_registry.rs ← Device management
│       ├── ipc.rs           ← Named pipe server
│       └── scheduler.rs     ← Refill & drain scheduler
├── crypto/                  ← AKSH: TLS enrollment
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── cert.rs          ← Self-signed cert generation
│       └── handshake.rs     ← TLS server & device enrollment
├── wfp-callout/             ← SAKSHAM: Kernel driver
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── engine.rs        ← WFP engine RAII wrapper
│       └── pipe.rs          ← Kernel named pipe writer
└── ui/                      ← SAKSHAM: Tauri UI
    ├── Cargo.toml
    └── src/
        └── main.rs          ← Tauri app entry point
```

## ✨ What Happens Next

1. **Today:** You read this and your setup guide
2. **Tomorrow:** Run the setup checklist on your machine
3. **Next day:** Quick sync call to verify both setups work
4. **Day 4:** Start Milestone 1
   - **Saksham:** WFP engine registration
   - **Aksh:** Wait for proto feedback, prepare unit tests

---

**Ready to begin?** 👉 Start with your setup guide:
- **Aksh:** [docs/SETUP_MAC.md](docs/SETUP_MAC.md)
- **Saksham:** [docs/SETUP_WINDOWS.md](docs/SETUP_WINDOWS.md)

Good luck! 🚀
