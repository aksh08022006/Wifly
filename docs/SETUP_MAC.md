# Setup Guide for Aksh (macOS — Backend Development)

This guide gets your Mac ready to:
1. ✅ Develop daemon/crypto crates (main backend work)
2. ✅ Cross-compile to Windows targets (daemon.exe for testing on Saksham's machine)
3. ✅ Run unit tests for proto/daemon/crypto
4. ❌ NOT compile wfp-callout (kernel driver only builds on Windows)

## Prerequisites

- macOS 11+
- ~5 GB free disk space
- Internet connection (to download Rust, LLVM, Windows SDK headers)

## Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:
```bash
rustc --version  # Should print: rustc 1.75.x (or later)
```

## Step 2: Install Windows Target

```bash
rustup target add x86_64-pc-windows-msvc
```

## Step 3: Install LLVM (for cross-compiler linker)

```bash
brew install llvm
```

Verify:
```bash
which lld-link
```

## Step 4: Download Windows SDK Headers (xwin)

This downloads Microsoft's Windows SDK headers legally and stores them locally (no .exe installation).

```bash
cargo install xwin
xwin --accept-license splat --output ~/.xwin
```

⏱️ **This takes 5-10 minutes** and downloads ~500 MB.

After completion, verify:
```bash
ls ~/.xwin/crt/lib/x86_64
# Should list: advapi32.lib, kernel32.lib, ntdll.lib, etc.
```

## Step 5: Update .cargo/config.toml

The repository already has `.cargo/config.toml` configured. Verify it references your `$HOME/.xwin` path. If on a different machine, the path should auto-resolve via `$HOME`.

## Step 6: Test Cross-Compilation

From the netshaper root:

```bash
cd netshaper
cargo build --workspace --exclude wfp-callout --target x86_64-pc-windows-msvc
```

Expected result:
- ✅ Compiles without errors
- ✅ Produces `target/x86_64-pc-windows-msvc/debug/daemon.exe`

If this works, you're ready!

## Step 7: Run Tests Locally

```bash
# Test proto crate (IPC types)
cargo test -p proto

# Test daemon crate (token bucket, registry, IPC server stubs)
cargo test -p daemon

# Test crypto crate (cert generation, TLS server stubs)
cargo test -p crypto

# Build UI crate (Tauri can't run headless on macOS, but should compile)
cargo build -p ui
```

## Step 8: Clone the Repository

```bash
git clone <repo_url> netshaper
cd netshaper
```

## Daily Workflow

### Developing a Feature

```bash
# Create a branch
git checkout -b aksh/milestone-N-description

# Make changes in daemon/, crypto/, or proto/
vim daemon/src/bucket.rs

# Run tests
cargo test -p daemon

# Build for Windows target
cargo build -p daemon --target x86_64-pc-windows-msvc

# Commit and push
git add .
git commit -m "feat: implement token bucket drain_ready"
git push origin aksh/milestone-N-description

# Open a PR, request Saksham's review
```

### Changing the proto Crate

⚠️ **CRITICAL:** Proto changes break the IPC contract.

```bash
# 1. Create a new branch (MUST be proto-update/*)
git checkout -b proto-update/add-packet-priority

# 2. Make the change
vim proto/src/lib.rs

# 3. Update ALL affected code
# - Update daemon if it consumes this type
# - Update wfp-callout if it produces this type
# - Update tests

# 4. Run ALL tests
cargo test -p proto -p daemon -p crypto

# 5. Push and request BOTH reviews
git push origin proto-update/add-packet-priority
```

### Troubleshooting

#### "lld-link: command not found"

```bash
# Reinstall LLVM
brew reinstall llvm

# Or set rustflags manually
export RUSTFLAGS="-C linker=$(brew --prefix llvm)/bin/lld"
cargo build --target x86_64-pc-windows-msvc
```

#### "xwin download failed"

```bash
# Try with an older manifest version (e.g., Windows 10)
xwin --accept-license --manifest-version 17134 splat --output ~/.xwin
```

#### "The specified module could not be found" (when linking)

```bash
# Verify xwin output directory
ls ~/.xwin/sdk/lib/um/x86_64

# If empty, re-run xwin
xwin --accept-license splat --output ~/.xwin
```

## SSH Keys for GitHub

```bash
# Generate SSH key (if you don't have one)
ssh-keygen -t ed25519 -C "aksh@example.com"

# Add to SSH agent
ssh-add ~/.ssh/id_ed25519

# Copy public key to GitHub settings
cat ~/.ssh/id_ed25519.pub
```

## VS Code Extensions (Recommended)

- **Rust-analyzer:** `rust-lang.rust-analyzer`
- **CodeLLDB:** `vadimcn.vscode-lldb` (for debugging)
- **GitLens:** `eamodio.gitlens`

## Daemon.exe on Saksham's Windows Machine

Once you've built daemon.exe, Saksham can run it on Windows:

```bash
# On your Mac:
cargo build -p daemon --target x86_64-pc-windows-msvc --release

# Copy to shared location or git push
scp target/x86_64-pc-windows-msvc/release/daemon.exe saksham@windows-pc:/path/to/daemon.exe
```

Then on Windows:
```powershell
cd path\to\daemon.exe
.\daemon.exe --log-level debug
```

---

**You're ready!** Proceed to Milestone 0 setup with Saksham.
