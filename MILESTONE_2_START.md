# 🚀 Milestone 2: Token Bucket Rate Limiter — START HERE

**Status:** Saksham completed Milestone 1 ✅  
**Your Turn:** Implement Milestone 2 (Token Bucket + Device Registry + IPC)  
**Timeline:** Days 5-7 (estimated)

## What Saksham Did (Milestone 1)

Saksham implemented the WFP kernel callout skeleton:
- ✅ `WfpEngine` RAII wrapper (handles lifecycle)
- ✅ `WfpError` enum for error handling
- ✅ Basic WFP engine initialization structure
- ✅ Ready for packet interception

**Note:** WFP implementation is still a stub (TODOs in place), but the structure is sound.

## What You Need to Do (Milestone 2)

Your job is to implement the **daemon** — the userspace bandwidth control service.

### 3 Key Components

| Component | File | What It Does |
|-----------|------|-------------|
| **Token Bucket** | `daemon/src/bucket.rs` | Rate limiting algorithm + refill logic |
| **Device Registry** | `daemon/src/device_registry.rs` | Track all devices + their bandwidth |
| **IPC Server** | `daemon/src/ipc.rs` | Receive PacketMetadata, send PacketDecision |

---

## ✅ Immediate Next Steps

### Step 1: Create Your Feature Branch (RIGHT NOW)

```bash
cd /Users/akshkaushik/Desktop/Waifu/netshaper
git checkout main
git pull origin main
git checkout -b aksh/milestone-2-token-bucket
```

### Step 2: Review What's Already Done

The skeleton code is already in place. Check each file:

```bash
# Review current implementations (stubs with TODOs)
cat daemon/src/bucket.rs        # Token bucket structure (incomplete)
cat daemon/src/device_registry.rs  # Device management (incomplete)
cat daemon/src/ipc.rs           # IPC server stub (incomplete)
cat daemon/src/scheduler.rs     # Scheduler stub (incomplete)
```

### Step 3: Understand the Contract

The `proto/src/lib.rs` defines what you MUST work with:

```bash
cat proto/src/lib.rs
```

Key types you'll use:
- `PacketMetadata { src_ip, dst_ip, byte_len, packet_id }`
- `PacketDecision { Permit | Drop }`
- `BandwidthUpdate { ip, bytes_per_sec }`
- `DeviceState { ip, hostname, bytes_per_sec, current_usage, is_blocked }`

---

## 🎯 Implementation Roadmap

### Phase 1: Core Token Bucket (Days 1-2)

**File:** `daemon/src/bucket.rs`

Complete these functions:

```rust
impl DeviceBucket {
    pub fn new(bytes_per_sec: u64) -> Self { ... }
    pub fn refill(&mut self) { ... }
    pub fn try_consume(&mut self, bytes: u32) -> bool { ... }
    pub fn drain_ready(&mut self) -> Vec<DeferredPacket> { ... }
}
```

**Tests to pass:**
- `test_refill_adds_tokens` — Verify tokens accumulate over time
- `test_try_consume_succeeds_when_available` — Deduct tokens when available
- `test_try_consume_fails_when_empty` — Queue packets when empty
- `test_throttle_timing` — Real-world timing (1000 packets at 10 MB/s should take ~100ms)

### Phase 2: Device Registry (Day 2)

**File:** `daemon/src/device_registry.rs`

Complete these functions:

```rust
impl DeviceRegistry {
    pub fn new() -> Self { ... }
    pub fn insert_device(&mut self, ip: Ipv4Addr, bytes_per_sec: u64) { ... }
    pub fn get_bucket_mut(&mut self, ip: Ipv4Addr) -> Option<&mut DeviceBucket> { ... }
    pub fn remove_device(&mut self, ip: Ipv4Addr) -> Option<DeviceBucket> { ... }
    pub fn update_bandwidth(&mut self, ip: Ipv4Addr, bytes_per_sec: u64) { ... }
}
```

**Tests to pass:**
- `test_insert_and_get` — Device creation and retrieval
- `test_remove_device` — Cleanup
- `test_update_bandwidth` — Dynamic bandwidth changes

### Phase 3: IPC Server Skeleton (Day 3)

**File:** `daemon/src/ipc.rs`

You don't need to fully implement this yet (Saksham needs to finish kernel side first), but:

```rust
pub async fn run_pipe_server(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), DaemonError> {
    // Listen on NETSHAPER_PIPE_NAME
    // For each connection:
    //   - Read bincode-encoded DaemonCommand
    //   - Match on UpdateBandwidth / ListDevices / Shutdown
    //   - Respond appropriately
    // TODO: Implement named pipe binding
}
```

**For now:** Just make sure it compiles. Full implementation can wait.

### Phase 4: Scheduler Task (Day 3)

**File:** `daemon/src/scheduler.rs`

```rust
pub async fn run_scheduler(
    registry: Arc<Mutex<DeviceRegistry>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Every 1ms:
        //   1. Lock registry
        //   2. For each device: call bucket.refill()
        //   3. For each device: drain_ready() packets
        //   4. Send PacketDecision back to kernel (when kernel is ready)
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
```

---

## 📋 Detailed Implementation Guide

### Token Bucket Algorithm (daemon/src/bucket.rs)

**Key Formula:**
```
Refill:
  elapsed = now - last_refill
  tokens = min(tokens + elapsed * rate, max_burst)
  last_refill = now

Try Consume:
  if tokens >= bytes:
    tokens -= bytes
    return true
  else:
    queue packet, return false
```

**Critical: Use `as_secs_f64()` for timing**
```rust
let elapsed = self.last_refill.elapsed().as_secs_f64();
```

**DO NOT** use a counter variable — float rounding errors accumulate over time.

### Device Registry (daemon/src/device_registry.rs)

Simple HashMap wrapper:
```rust
pub struct DeviceRegistry {
    devices: HashMap<Ipv4Addr, DeviceBucket>,
}
```

Thread-safe access via `Arc<Mutex<DeviceRegistry>>` in main.rs.

### Integration Test Example

```rust
#[tokio::test]
async fn test_token_bucket_timing() {
    let mut bucket = DeviceBucket::new(100_000); // 100 KB/s
    bucket.current_tokens = 0.0;
    bucket.last_refill = Instant::now();

    let start = Instant::now();

    // Try to consume 10 KB, 10 times (total 100 KB)
    for _ in 0..10 {
        while !bucket.try_consume(10_000) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let elapsed = start.elapsed();
    // Should take ~1 second to drain 100 KB at 100 KB/s
    assert!(elapsed >= Duration::from_millis(900));
    assert!(elapsed <= Duration::from_millis(1100));
}
```

---

## 🧪 Testing Strategy

### Run Tests Locally

```bash
# Test just token bucket
cargo test -p daemon --lib bucket

# Test device registry
cargo test -p daemon --lib device_registry

# Run all daemon tests
cargo test -p daemon

# Check for warnings
cargo clippy -p daemon
```

### Integration Tests

Create `daemon/tests/integration_test.rs`:

```rust
#[tokio::test]
async fn test_registry_with_multiple_devices() {
    let mut registry = DeviceRegistry::new();
    
    let ip1: Ipv4Addr = "192.168.1.100".parse().unwrap();
    let ip2: Ipv4Addr = "192.168.1.101".parse().unwrap();
    
    registry.insert_device(ip1, 1_000_000);  // 1 MB/s
    registry.insert_device(ip2, 5_000_000);  // 5 MB/s
    
    // Verify both exist
    assert!(registry.get_bucket(ip1).is_some());
    assert!(registry.get_bucket(ip2).is_some());
}
```

---

## 💾 Git Workflow

### Commit Early & Often

```bash
# After implementing token bucket
git add daemon/src/bucket.rs
git commit -m "feat(daemon): implement token bucket refill and try_consume"
git push origin aksh/milestone-2-token-bucket

# After implementing device registry
git add daemon/src/device_registry.rs
git commit -m "feat(daemon): implement device registry with insert/remove/update"
git push origin aksh/milestone-2-token-bucket

# When ready for review
# Open PR on GitHub requesting review from Saksham
```

### Commit Message Format

```
feat(daemon): implement token bucket refill logic
fix(daemon): correct bucket timing calculation
test(daemon): add integration test for multiple devices
docs(daemon): clarify token bucket algorithm
```

---

## 🧩 Milestones Within Milestone 2

### Milestone 2.1: Token Bucket ✅ FIRST
- [ ] `bucket.rs` fully implemented
- [ ] All unit tests pass
- [ ] No compiler warnings

### Milestone 2.2: Device Registry ✅ SECOND
- [ ] `device_registry.rs` fully implemented
- [ ] All unit tests pass
- [ ] Integration tests with buckets pass

### Milestone 2.3: IPC Scaffold ✅ THIRD
- [ ] `ipc.rs` compiles (TODO OK for now)
- [ ] `scheduler.rs` compiles
- [ ] Main entry point builds
- [ ] Ready for IPC when kernel is ready

---

## ⚠️ Critical Timing Notes

### Float Precision

```rust
// ❌ WRONG: Counter accumulates rounding error
static mut total_tokens: f64 = 0.0;
total_tokens += rate * dt;  // Error compounds!

// ✅ RIGHT: Use elapsed time from Instant
let elapsed = self.last_refill.elapsed().as_secs_f64();
self.current_tokens = (self.current_tokens + elapsed * rate).min(max_burst);
```

### 1ms Scheduler Precision

The scheduler wakes every 1ms to refill buckets. This is aggressive but necessary:
- Too slow (10ms): Bursty, less smooth throttling
- Too fast (0.1ms): CPU-heavy
- **1ms: Sweet spot** between smoothness and efficiency

---

## 📚 Resources

### Token Bucket Algorithm
- https://en.wikipedia.org/wiki/Token_bucket
- Core concept: Tokens = permission to send bytes

### Rust Async Timing
- https://docs.rs/tokio/latest/tokio/time/
- `tokio::time::sleep()` for delays
- `Instant::elapsed().as_secs_f64()` for timing

### Test Patterns
- https://doc.rust-lang.org/book/ch11-03-test-organization.html
- Unit tests in same file (`#[cfg(test)]` modules)
- Integration tests in `tests/` directory

---

## ✨ Success Criteria

When you're done with Milestone 2, you should have:

- ✅ Token bucket that throttles correctly (measurable via tests)
- ✅ Device registry that tracks multiple devices
- ✅ IPC server skeleton that compiles
- ✅ All unit tests passing
- ✅ Zero compiler warnings
- ✅ All commits pushed to `aksh/milestone-2-token-bucket`
- ✅ PR open for Saksham to review

---

## 🎯 You Are Ready!

Everything you need:
- ✅ Proto contract in `proto/src/lib.rs`
- ✅ Skeleton code in `daemon/src/`
- ✅ Documentation in `docs/`
- ✅ CI/CD will validate your work

**Start now:**

```bash
cd /Users/akshkaushik/Desktop/Waifu/netshaper
git checkout -b aksh/milestone-2-token-bucket
vim daemon/src/bucket.rs  # Start implementing!
```

Good luck! 🚀 You've got this.
