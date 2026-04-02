# Architecture Overview

## Five-Layer Architecture

NetShaper is built as five distinct technical layers. Each layer must work correctly before the next layer is built.

```
┌─────────────────────────────────────────┐
│ Layer 5: Tauri Control Panel (UI)        │
│ - System tray app                        │
│ - Device cards with sliders              │
│ - Sends BandwidthUpdate to daemon        │
└────────────────┬────────────────────────┘
                 │ (Named Pipe)
                 ↓
┌─────────────────────────────────────────┐
│ Layer 4: Kernel ↔ Userspace Bridge       │
│ - Named pipe (windows named pipe)        │
│ - Binary serialization (bincode)         │
│ - PacketMetadata → PacketDecision        │
└────────────────┬────────────────────────┘
                 │
         ┌───────┴───────┐
         ↓               ↓
┌────────────────┐  ┌────────────────────┐
│ Layer 3:       │  │ Layer 2:            │
│ Crypto/mTLS    │  │ Token Bucket +      │
│ - Device       │  │ Daemon Service      │
│ enrollment     │  │ - Rate limiter      │
│ - Consent UI   │  │ - Device registry   │
│ - TLS port     │  │ - Scheduler         │
│ 7979           │  │ - Packet queue      │
└────────────────┘  └────────────────────┘
         │               ↑
         └───────────────┤
                 │ (tokio async)
                 │
        ┌────────┴────────┐
        │                 │
        ↓                 ↓
 ┌──────────────────────────────────┐
 │ Layer 1: WFP Kernel Callout       │
 │ (Kernel mode, no std, no alloc)   │
 │ - Packet interception at NIC      │
 │ - Classify callback (~10 µs)      │
 │ - DEFER packets to userspace      │
 │ - Read decisions from named pipe  │
 └──────────────────────────────────┘
        ↑
        │ (PacketMetadata)
        │
 ┌──────────────────┐
 │ Network Stack    │
 │ (Windows Kernel) │
 └──────────────────┘
```

## Layer 1: WFP Kernel Callout (wfp-callout crate)

**Location:** `wfp-callout/`  
**Owner:** Saksham (Windows only)  
**Language:** Rust (no_std)  
**Purpose:** Intercept packets at the NIC level

### What It Does

1. **Registers** with Windows Filtering Platform (FwpmEngineOpen0)
2. **Installs** a filter rule targeting FWPM_LAYER_OUTBOUND_IPPACKET_V4
3. **Implements** a classify callback that runs for every outbound IPv4 packet
4. **Extracts** source/destination IP and packet size from FWPS_INCOMING_VALUES
5. **Enqueues** packet metadata to the named pipe (to daemon)
6. **Returns** FWP_ACTION_DEFER to hold the packet
7. **Later** receives FWP_ACTION_PERMIT or FWP_ACTION_BLOCK from the daemon

### Key Constraint

The classify callback runs at **IRQL DISPATCH_LEVEL** — kernel code at the highest priority. It must:
- ✅ Complete in **< 10 microseconds**
- ✅ Use **only stack memory** (no heap allocation)
- ✅ Call **only non-paged functions** (no file I/O, no locks)
- ✅ Never call any function that might **page fault**

**Solution:** Callbacks are minimal (~30 lines). All decision logic lives in the daemon.

### Key APIs

| Function | Purpose |
|----------|---------|
| `FwpmEngineOpen0` | Open a session with the filter engine |
| `FwpmCalloutAdd0` | Register your callout with a GUID |
| `FwpmFilterAdd0` | Add a filter rule (which layer, which conditions) |
| `FwpsCalloutRegister2` | Register the actual C callback function pointers with kernel |
| `FwpsCompleteOperation0` | Tell kernel: PERMIT this packet ID |

---

## Layer 2: Token Bucket Rate Limiter (daemon crate)

**Location:** `daemon/`  
**Owner:** Aksh (macOS cross-compile)  
**Language:** Rust + Tokio (async)  
**Purpose:** Apply rate limiting using the token bucket algorithm

### What It Does

1. **Maintains** a `DeviceRegistry` → HashMap<Ipv4Addr, DeviceBucket>
2. **For each device**, tracks a token bucket:
   - `allowed_bytes_per_sec` — configured bandwidth ceiling
   - `max_burst_bytes` — capacity (usually 2× one-second allowance)
   - `current_tokens` — bytes available right now
   - `last_refill` — timestamp for refill calculation
3. **On each 1ms tick**, calls `refill()` on all buckets
4. **Drains** packets from each queue that now have tokens
5. **Sends** PacketDecision messages (PERMIT/DROP) back to kernel

### Token Bucket Algorithm

```
Refill:
  elapsed = now - last_refill
  tokens = min(tokens + elapsed * rate, max_burst)
  last_refill = now

Try Consume (bytes):
  if tokens >= bytes:
    tokens -= bytes
    return true (PERMIT)
  else:
    return false (queue it)
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `bucket.rs` | DeviceBucket struct + token bucket logic |
| `device_registry.rs` | Manage all devices, insert/remove, update bandwidth |
| `ipc.rs` | Named pipe server, accept messages from kernel/UI |
| `scheduler.rs` | Tokio task: refill & drain on 1ms interval |

---

## Layer 3: Crypto & Consent Handshake (crypto crate)

**Location:** `crypto/`  
**Owner:** Aksh (macOS, but runs on daemon machine)  
**Language:** Rust + Tokio  
**Purpose:** Device enrollment & authorization

### What It Does

1. **On first launch:**
   - Generate Ed25519 self-signed certificate (valid 10 years)
   - Save to `~/.netshaper/ca.pem` and `~/.netshaper/ca.key`

2. **Listen on port 7979** (TLS):
   - Smartphone connects
   - Daemon presents the cert
   - User sees: "Device 192.168.1.100 wants to join NetShaper — Accept/Decline"

3. **On Accept:**
   - Write device IP to `~/.netshaper/devices.json`
   - Device is now "enrolled"

4. **Check at runtime:**
   - WFP callout consults `is_enrolled(ip)` before permitting packets
   - Unregistered devices are blocked at the kernel level

### Key Modules

| Module | Purpose |
|--------|---------|
| `cert.rs` | Generate/save/load self-signed certs |
| `handshake.rs` | TLS server, enrollment UI, device persistence |

---

## Layer 4: Kernel ↔ Userspace IPC (proto crate)

**Location:** `proto/`  
**Owner:** Both (Aksh proposes, Saksham reviews)  
**Language:** Rust (serde + bincode)  
**Purpose:** Define the message contract

### Message Types

```rust
PacketMetadata {
  src_ip: Ipv4Addr,
  dst_ip: Ipv4Addr,
  byte_len: u32,
  packet_id: u64,  // opaque handle
}

PacketDecision {
  PERMIT { packet_id },
  DROP { packet_id },
}

BandwidthUpdate {
  ip: Ipv4Addr,
  bytes_per_sec: u64,  // 0 = blocked
}

DeviceState {
  ip, hostname, bytes_per_sec, current_usage, is_blocked
}
```

### Communication Flow

```
Kernel (WFP callout):
  → read PacketMetadata from pipe
  → forward to daemon

Daemon (token bucket):
  ← receive PacketMetadata
  → apply token bucket logic
  → determine PERMIT or DROP
  → write PacketDecision to pipe

Kernel (WFP callout):
  ← read PacketDecision
  → call FwpsCompleteOperation0 with decision
```

### Named Pipe Path

```
\\.\pipe\netshaper
```

Defined in `proto/src/lib.rs` as `pub const NETSHAPER_PIPE_NAME`.

---

## Layer 5: Tauri Control Panel (ui crate)

**Location:** `ui/`  
**Owner:** Saksham (Windows only)  
**Language:** Rust (Tauri backend) + JavaScript (WebView2 frontend)  
**Purpose:** System-tray UI for device management

### What It Does

1. **System tray icon** that launches on Windows startup
2. **Device cards** showing:
   - Device IP & hostname
   - Current bandwidth slider (0 Mbps — unlimited)
   - Block toggle (quick disable)
   - Current usage graph
3. **On slider change:**
   - Send BandwidthUpdate to daemon over named pipe
   - Daemon updates the token bucket immediately
4. **1-second polling** from daemon to show live bandwidth usage

### Key Tauri Commands

```rust
#[tauri::command]
async fn list_devices() -> Vec<DeviceState>

#[tauri::command]
async fn set_bandwidth(ip: Ipv4Addr, bytes_per_sec: u64)
```

---

## Data Flow: An HTTP Download Being Throttled

```
Smartphone user downloads 10MB file over Wi-Fi
↓
[1] Kernel sees outbound TCP packets (src: 192.168.1.100, dst: cdn.example.com)
    ↓ (WFP Callout)
    Extracts: PacketMetadata { src_ip: 192.168.1.100, byte_len: 1500, packet_id: 999 }
    Writes to named pipe
    Returns FWP_ACTION_DEFER (hold packet in kernel)

↓ [2] Daemon reads PacketMetadata from pipe
    Looks up 192.168.1.100 in DeviceRegistry
    Finds DeviceBucket { allowed_bytes_per_sec: 1_000_000 (1 MB/s), current_tokens: ... }
    Calls try_consume(1500)
    If tokens available: tokens -= 1500; send PERMIT
    If not: enqueue to bucket.queue

↓ [3] Scheduler runs every 1ms:
    For each bucket: refill() (add elapsed_time * rate)
    drain_ready() to find packets that now have tokens
    For each drained packet: send PacketDecision::Permit { packet_id: 999 }

↓ [4] Kernel callout receives PacketDecision
    Calls FwpsCompleteOperation0(packet_id: 999, FWP_ACTION_PERMIT)
    Packet is released from kernel queue

↓ [5] Smartphone receives packet, stream continues at ~1 MB/s
    User's iOS downloads 10MB in ~10 seconds instead of 1 second
    Phone OS auto-detects slow link, reduces stream quality to 480p
    User doesn't see buffering — smooth throttled experience!
```

---

## Testing Strategy by Layer

| Layer | Test Type | Command |
|-------|-----------|---------|
| Layer 2 (Token Bucket) | Unit | `cargo test -p daemon` |
| Layer 3 (Crypto) | Unit | `cargo test -p crypto` |
| Layer 4 (proto IPC) | Unit | `cargo test -p proto` |
| Layers 1+2+3 | Integration | Manual on Windows with iperf3 |
| All Layers | End-to-end | Real phone + fast.com download |

---

## Why This Architecture?

### Why kernel-mode WFP?

- ✅ Intercepts packets **before** they leave the NIC (most efficient)
- ✅ No userspace TCP proxy overhead
- ✅ Supported officially by Windows (via WDK)
- ✅ Same API used by Windows Defender, Wireshark

### Why token bucket in userspace?

- ✅ Kernel memory is precious (limited to ~2 GB pool)
- ✅ Complex rate-limit math in userspace is safer
- ✅ Easier to test and debug (no BSOD risk)
- ✅ Can use Rust std lib, tokio, normal logging

### Why named pipes for IPC?

- ✅ Simple binary protocol (no schemas needed)
- ✅ Persistent connections (no syscall overhead)
- ✅ Works across privilege boundaries (kernel ↔ userspace)
- ✅ Can be captured for debugging (wireshark, etc.)

### Why Tauri?

- ✅ Rust backend (consistent with daemon/wfp-callout)
- ✅ WebView2 on Windows (no Electron bloat)
- ✅ System tray support out-of-box
- ✅ Single binary distribution

---

## Development Notes for Each Owner

### Aksh (Daemon + Crypto)

Focus on:
- Token bucket correctness
- Concurrent device management
- Device persistence
- Named pipe client implementation

Work independently:
- Develop & test entirely on macOS
- Cross-compile to .exe for testing on Windows
- Unit tests run on any platform
- No kernel code = no BSOD risk

### Saksham (WFP Callout + UI)

Focus on:
- WFP filter registration
- Classify callback performance
- Kernel stability
- UI/UX for device management

Challenges:
- WFP development requires Windows PC
- BSOD can happen in kernel code
- Minimal callback code is critical
- Must coordinate proto changes with Aksh

---

**Next:** Read [TESTING.md](./TESTING.md) for the testing strategy.
