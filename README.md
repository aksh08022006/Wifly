# NetShaper — Windows Bandwidth Controller

A kernel-level bandwidth control system for Windows written entirely in Rust. Gives administrators fine-grained control over network traffic consumed by devices on the same Wi-Fi network.

**Status:** Early Development — Milestone 0 Setup Phase

## Quick Links

- **For Aksh (macOS backend development):** See [SETUP_MAC.md](./docs/SETUP_MAC.md)
- **For Saksham (Windows driver & UI):** See [SETUP_WINDOWS.md](./docs/SETUP_WINDOWS.md)
- **Architecture Overview:** See [ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- **Testing Guide:** See [TESTING.md](./docs/TESTING.md)

## Project Structure

```
netshaper/
├── Cargo.toml                    # Workspace manifest
├── proto/                        # Shared IPC types (CRITICAL contract)
├── daemon/                       # Userspace bandwidth service (Aksh)
├── wfp-callout/                  # Kernel WFP callout driver (Saksham)
├── crypto/                       # mTLS device enrollment (Aksh)
├── ui/                           # Tauri system-tray UI (Saksham)
├── .github/workflows/            # CI/CD pipelines
└── .cargo/config.toml            # Cross-compilation settings
```

## Key Technologies

| Component | Technology | Responsibility |
|-----------|-----------|-----------------|
| **daemon** | Tokio + Rust | Token bucket algorithm, packet scheduling, device registry |
| **wfp-callout** | Windows WFP API | Kernel-mode packet interception at FWPM_LAYER_OUTBOUND_IPPACKET_V4 |
| **crypto** | rcgen + rustls | Self-signed Ed25519 certs, device enrollment server |
| **proto** | serde + bincode | IPC message contract (shared across all crates) |
| **ui** | Tauri + WebView2 | System-tray app for bandwidth control |

## Development Workflow

### Before You Start

Both Aksh and Saksham must complete **Milestone 0** setup on their respective machines:

- **Aksh (macOS):** Install Rust, xwin, and cross-compilation linker
- **Saksham (Windows):** Install Rust, WDK, Visual Studio Build Tools, enable test signing

### Core Rule: Protect the proto Crate

The `proto/` crate defines the IPC contract. **Any change to proto/ REQUIRES both teammates to review and approve.**

Breaking changes here cause **silent data corruption** at the pipe boundary — the compiler cannot save you.

### Branch Naming Convention

```
main                           # Always compiles; protected branch (no direct pushes)
aksh/milestone-N-description   # Aksh's work
saksham/milestone-N-description # Saksham's work
proto-update/description       # Any proto/ change (requires both reviews)
```

### Testing & CI

- **Unit tests:** Run anywhere (no Windows VM needed)
  ```bash
  cargo test -p proto -p daemon -p crypto
  ```

- **Windows build CI:** GitHub Actions on `windows-latest`
  ```bash
  cargo build -p wfp-callout --target x86_64-pc-windows-msvc
  ```

- **Cross-platform CI:** GitHub Actions tests daemon/crypto on Ubuntu, macOS, Windows

- **Integration tests:** Manual on Saksham's Windows machine (end-to-end bandwidth throttling)

## Milestone Roadmap

| # | Title | Owner | Duration | Goal |
|---|-------|-------|----------|------|
| 0 | Repo & Toolchain Setup | Both | 1 day | Toolchain working, cross-compile test passes |
| 1 | WFP Callout Skeleton | Saksham | 3 days | .sys file loads, registers callout, permits all packets |
| 2 | Token Bucket + IPC | Aksh | 4 days | Daemon reads packets, applies rate limiting, writes decisions |
| 3 | Crypto Consent Server | Aksh | 2 days | TLS enrollment, device persistence, is_enrolled checks |
| 4 | IPC Bridge Integration | Both | 3 days | First end-to-end: kernel → daemon → kernel, real throttling |
| 5 | Tauri Control Panel | Saksham | 4 days | System-tray UI, device cards, bandwidth sliders |
| 6 | Installer & MSI Packaging | Saksham | 2 days | cargo-wix, service registration, uninstaller |

## Running Tests

### Aksh's Responsibility (macOS)

```bash
# Compile all crates except wfp-callout (which requires Windows)
cargo build --workspace --exclude wfp-callout --target x86_64-pc-windows-msvc

# Run unit tests
cargo test -p proto
cargo test -p daemon
cargo test -p crypto
```

### Saksham's Responsibility (Windows)

```bash
# Build WFP callout
cargo build -p wfp-callout --target x86_64-pc-windows-msvc

# Run all tests
cargo test --workspace

# Load driver and test
sc.exe create netshaper-wfp type= kernel start= demand binPath= C:\path\to\netshaper_wfp.dll
```

## Known Issues & Gotchas

### "Access Denied" When Loading .sys

**Solution:** Run `bcdedit /set testsigning on` and restart. Requires Administrator.

### WFP Callout Crashes (BSOD)

**Solution:** Don't allocate memory or take locks in the classify callback. Keep it under 10 microseconds.

### Named Pipe: "Pipe Not Found"

**Solution:** Start daemon FIRST, then load .sys. The server end must exist before client tries to connect.

### Cross-Compile Fails on macOS

**Solution:** Run `xwin --accept-license splat --output ~/.xwin` first to download Windows SDK headers.

### WebView2 Missing on Windows 10

**Solution:** tauri.conf.json includes `"webviewInstallMode": { "type": "downloadBootstrapper" }` to auto-install.

## Communication Channels

- **Synchronous:** Slack #netshaper-dev
- **Asynchronous:** GitHub Issues, Pull Requests
- **Stand-ups:** 3× weekly at [TBD]
- **Proto Changes:** Always PR, always both review before merge

## References

- [Windows Filtering Platform (WFP)](https://docs.microsoft.com/en-us/windows/win32/fwp/windows-filtering-platform-start-page)
- [Rust windows crate](https://github.com/microsoft/windows-rs)
- [Token Bucket Algorithm](https://en.wikipedia.org/wiki/Token_bucket)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [rcgen Documentation](https://github.com/rustls/rcgen)

## License

TBD

---

**Next Step:** Both teammates must complete Milestone 0 setup before any coding begins.
