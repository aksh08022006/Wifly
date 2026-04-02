# NetShaper Milestone 2: Quick Reference Guide

## 30-Second Overview

**What**: Userspace daemon that rate-limits bandwidth per device
**How**: Token bucket algorithm, one per device, refilled every 1ms
**Why**: Windows WFP kernel can't do sophisticated QoS; daemon provides the "brain"

```
Kernel intercepts packet → Checks with daemon → Daemon says PERMIT or DROP
```

---

## The 4 Components

### 1. Token Bucket (`bucket.rs`)

**One bucket per device. Implements this logic:**

```
┌─────────────────────────────────────┐
│ Token Bucket (100 KB/s bandwidth)   │
├─────────────────────────────────────┤
│                                     │
│  Capacity: 200 KB (2× bandwidth)   │
│  Current tokens: [████████░░] 150KB│
│                                     │
│  Refill rate: 100 KB/s              │
│  Refilled every 1ms: +0.1 KB        │
│                                     │
│  Incoming packet 50KB?              │
│    ✓ Can pay 50KB → PERMIT          │
│    Remaining: 100KB                 │
│                                     │
│  Incoming packet 200KB?             │
│    ✗ Need 200KB, have 100KB         │
│    → QUEUE packet (wait)            │
│                                     │
└─────────────────────────────────────┘
```

**The math:**
- Starts FULL at 2× bandwidth
- Adds `elapsed_time × bandwidth` tokens every tick
- Capped at max burst
- Packet consumes N bytes of tokens
- If not enough: queued until more tokens available

**Key insight**: Uses `f64` precision, not integer tokens. Allows smooth, fair distribution.

---

### 2. Device Registry (`device_registry.rs`)

**Just a HashMap. Tracks all managed devices.**

```
┌──────────────────────────────────────────┐
│ DeviceRegistry                           │
├──────────────────────────────────────────┤
│                                          │
│ 192.168.1.100 → DeviceBucket (1 MB/s)   │
│ 192.168.1.101 → DeviceBucket (100 KB/s) │
│ 192.168.1.102 → DeviceBucket (BLOCKED)  │
│                                          │
└──────────────────────────────────────────┘
```

**Operations:**
- `insert_device(ip, bytes_per_sec)` - Add or replace
- `get_bucket(ip)` - Read device's bucket
- `get_bucket_mut(ip)` - Modify device's bucket (scheduler uses this)
- `update_bandwidth(ip, new_rate)` - Change rate on the fly
- `list_devices()` - Get all IPs
- `remove_device(ip)` - Delete device

---

### 3. IPC Server (`ipc.rs`)

**Listens for commands, sends responses. Platform-specific:**

**Windows**:
```
Named Pipe: \\.\pipe\netshaper
Kernel ←→ Daemon ←→ UI (all use this)
```

**macOS/Linux (dev)**:
```
Unix Socket: /tmp/netshaper.sock
Daemon only, for testing
```

**Commands the daemon accepts:**

1. **UpdateBandwidth**
   ```
   Sender: UI or kernel
   Payload: { ip: Ipv4Addr, bytes_per_sec: u64 }
   Action: Update device's bandwidth ceiling
   Response: None (async)
   ```

2. **ListDevices**
   ```
   Sender: UI
   Payload: None
   Action: Snapshot all devices
   Response: Vec<DeviceState> {
       ip, hostname, bytes_per_sec, current_usage, is_blocked
   }
   ```

3. **Shutdown**
   ```
   Sender: Admin/system
   Payload: None
   Action: Stop daemon
   Response: Process exits
   ```

---

### 4. Scheduler (`scheduler.rs`)

**The heartbeat. Wakes every 1ms to refill and drain.**

```
loop {
    lock registry
    
    for each device:
        refill bucket (add tokens)
        drain ready packets (release any that now have tokens)
        log stats
    
    unlock registry
    sleep 1ms
}
```

**Why every 1ms?**
- Smooth: 1000 wakeups/sec = fine-grained control
- Efficient: Not too chatty (100µs would be overkill)
- Reasonable: 1ms @1MB/s = 1KB token grants per tick (easy math)

**What it tracks:**
- Total ticks since start
- Packets released this tick
- Packets still queued

**Logs every 1000 ticks (~1 second):**
```
Scheduler stats: 1000 ticks, 500 packets released, 45 currently queued
```

---

## Data Flow Examples

### Example 1: UI Changes Device Bandwidth

```
1. UI → Daemon (UpdateBandwidth 192.168.1.100, 5MB/s)
   
2. IPC server receives over named pipe
   
3. process_command() matches UpdateBandwidth
   
4. Locks registry, calls:
   reg.update_bandwidth(ip, 5_000_000)
   
5. Registry finds device, updates:
   bucket.allowed_bytes_per_sec = 5_000_000
   
6. Immediately, next scheduler tick:
   refill() will add tokens at 5MB/s rate
```

### Example 2: Kernel Sends Packet, Daemon Rate-Limits

```
1. Kernel intercepts 10KB packet from 192.168.1.100
   
2. [NOT YET IMPLEMENTED - waiting for Saksham's kernel integration]
   Kernel sends PacketMetadata over named pipe
   
3. Daemon receives (in scheduler or IPC thread)
   
4. Calls bucket.try_consume(10_000)
   
5. If PERMIT (tokens available):
   - Consume tokens
   - Send PacketDecision::Permit back to kernel
   - Kernel lets packet through
   
6. If QUEUE (no tokens):
   - No response to kernel
   - Kernel blocks packet
   - Daemon queues packet internally
   - Next tick (1ms later): refill() adds tokens
   - Next tick (or later): drain_ready() releases packet
   - Send PacketDecision::Permit to kernel
   - Kernel unblocks packet
```

### Example 3: UI Polls Device List

```
1. UI → Daemon (ListDevices)
   
2. IPC server receives
   
3. process_command() matches ListDevices
   
4. Locks registry
   
5. Calls build_device_states() which:
   for each device ip:
       get bucket = registry.get_bucket(ip)
       create DeviceState {
           ip: ip,
           bytes_per_sec: bucket.allowed_bytes_per_sec,
           current_usage: 0 (TODO: rolling average),
           is_blocked: (bytes_per_sec == 0),
           hostname: None (TODO: reverse DNS)
       }
   
6. Serializes Vec<DeviceState> with bincode
   
7. Writes to named pipe
   
8. UI receives, deserializes, renders device cards
```

---

## Concurrency Model

**Arc<Mutex<DeviceRegistry>>** - Shared between IPC and Scheduler

```
┌─────────────────────────────────────────┐
│ main.rs                                 │
│                                         │
│ registry = Arc::new(Mutex::new(...))    │
│                                         │
│ tokio::spawn IPC server                 │
│   └─ clones Arc                         │
│                                         │
│ tokio::spawn scheduler                  │
│   └─ clones Arc                         │
│                                         │
└─────────────────────────────────────────┘
         ↓
    Both tasks can now:
    let mut reg = registry.lock().await
    // Access registry
```

**Lock held:**
- IPC: ~10µs (deserialize + update)
- Scheduler: ~100µs (iterate + refill + drain)

**Lock released:**
- IPC: Waiting for next command
- Scheduler: Sleeping for 1ms

**No deadlocks** because:
- Only one Mutex (registry)
- Acquired in same order
- Always released

---

## Testing Quick Start

**Run all daemon tests:**
```bash
cd netshaper
cargo test -p daemon
```

**Run specific test:**
```bash
cargo test -p daemon test_refill_adds_tokens -- --nocapture
```

**Build and check for warnings:**
```bash
cargo clippy -p daemon -- -D warnings
```

**Format code:**
```bash
cargo fmt -p daemon
```

---

## Understanding the Algorithm

### Token Bucket Intuition

Imagine a bucket with a drain at the bottom:
- Bucket fills at 100 KB/s (your bandwidth)
- Each packet is a weight (e.g., 50 KB)
- If bucket has enough water: packet goes through (consume water)
- If bucket is dry: packet waits (queue up)
- Water slowly fills again (1 ms ticks)
- When full enough: packet goes through

### Why Burst?

```
Without burst:
- 1 MB/s bandwidth
- Every 1ms: +1 KB of tokens
- Small packet (10 bytes)? Might have to wait 1ms
- Unfair/bursty traffic

With burst (2× bandwidth):
- Start with 2 MB worth of tokens
- Small packet (10 bytes)? Approved immediately
- Device uses burst capacity in first 2 seconds
- Then stabilizes at 1 MB/s
- Smooth traffic, fair allocation
```

### Floating Point Precision

```rust
// CORRECT (what we do)
bucket.current_tokens = 150.5;  // Can represent fractional bytes

// WRONG (counter-based)
counter = 0;
on_tick() { counter += 1; }  // Loses precision over time
```

With f64, we get exact token counts forever. With integer counters, rounding errors accumulate.

---

## Files Structure

```
netshaper/
├── daemon/                  # Main daemon crate
│   ├── src/
│   │   ├── main.rs         # Entry point (Tokio runtime)
│   │   ├── bucket.rs       # Token bucket algorithm
│   │   ├── device_registry.rs # Device management
│   │   ├── ipc.rs          # IPC server (Windows + Unix)
│   │   └── scheduler.rs    # 1ms refill/drain loop
│   ├── Cargo.toml          # Dependencies (platform-specific)
│   └── tests/
│       └── integration_test.rs  # Full integration tests
│
├── proto/                  # Shared types (stable, locked)
│   └── src/lib.rs         # IPC message types
│
├── wfp-callout/            # Kernel callout (Saksham's)
├── crypto/                 # mTLS enrollment (future)
├── ui/                     # Tauri UI (future)
└── docs/                   # Architecture docs
```

---

## Common Questions

**Q: Why is the bucket starting full?**
A: So devices can burst immediately (better UX). After 2 seconds at full rate, it stabilizes.

**Q: Why 1ms and not 10ms?**
A: 1ms is smooth (1000 ticks/sec) but not overkill. 10ms is too coarse. 100µs is wasteful.

**Q: What if the scheduler lags?**
A: refill() looks at elapsed time, so it always catches up. No tokens are lost.

**Q: What happens to packets in the queue if device is removed?**
A: They're lost. Current design doesn't persist. Future enhancement could flush them.

**Q: Why Arc<Mutex> instead of RwLock?**
A: RwLock is slower for this workload (mostly mutations). Mutex is simpler.

**Q: How many devices can we support?**
A: 100s easily at 1% CPU. 1000s require multi-threaded scheduler. Scales well.

**Q: What if a packet arrives while scheduler is running?**
A: IPC handler tries to acquire lock. Waits ~100µs for scheduler. No problem.

---

## Next Steps (Post-Milestone 2)

1. **Compile on Windows** and test with real WFP
2. **Saksham implements kernel IPC** (send PacketMetadata, receive PacketDecision)
3. **Merge to main** after integration
4. **Milestone 3**: Crypto/mTLS enrollment
5. **Milestone 5**: Tauri UI

---

## Getting Help

**If tests fail:**
1. Check platform (Windows vs macOS/Linux)
2. Check dependencies: `cargo tree -p daemon`
3. Check for unstable features

**If scheduler lags:**
1. Check CPU usage (should be <1%)
2. Check lock contention (add logging in main.rs)
3. Consider multi-threaded scheduler (future optimization)

**If IPC fails:**
1. Check named pipe name is correct
2. Check Windows Named Pipe permissions
3. On Unix, check socket file permissions (/tmp/netshaper.sock)

**If tokens don't refill:**
1. Check that bucket.last_refill is updated (it is, in refill())
2. Check that allowed_bytes_per_sec > 0
3. Check timing with test: `cargo test test_refill_adds_tokens -- --nocapture`
