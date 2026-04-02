# M1 Implementation Checklist for Saksham

## Phase 1: Integrate Packet Tracker (Foundation)

### 1.1 Add Global Packet Tracker to callout.rs

```rust
// Add to top of callout.rs after imports:
use lazy_static::lazy_static;
use crate::packet_tracker::PacketTracker;

lazy_static! {
    static ref PACKET_TRACKER: PacketTracker = {
        PacketTracker::new(2000, 1_000_000) // Max 2000 packets, 1s timeout
    };
}

// Also add for tracking current time:
use std::time::{SystemTime, UNIX_EPOCH};

fn get_current_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
```

**Expected Errors & Fixes:**
- `lazy_static` not in Cargo.toml → Add to `[dependencies]`
- Can't use `SystemTime` in kernel context → Use performance counter instead (TODO)

### 1.2 Update classify_callback to Use Tracker

**CURRENT CODE** (in callout.rs):
```rust
pub unsafe extern "system" fn classify_callback(...) {
    *action = FWP_ACTION_PERMIT.0;

    let metadata = match extract_packet_metadata(meta_values, context) {
        Some(m) => m,
        None => {
            return;
        }
    };

    if let Some(ref pipe) = PIPE_CLIENT {
        if let Some(decision) = pipe.query_decision(&metadata) {
            *action = match decision {
                PacketDecision::Permit { .. } => FWP_ACTION_PERMIT.0,
                PacketDecision::Drop { .. } => FWP_ACTION_BLOCK.0,
            };
        }
    }
}
```

**ENHANCED CODE** (with tracker integration):
```rust
pub unsafe extern "system" fn classify_callback(...) {
    *action = FWP_ACTION_PERMIT.0; // Safe default

    // Extract metadata
    let metadata = match extract_packet_metadata(meta_values, context) {
        Some(m) => m,
        None => {
            return; // Permit on extraction failure
        }
    };

    // Get current time for timeout tracking
    let current_time = get_current_micros();

    // Try to add to tracker
    if let Err(e) = PACKET_TRACKER.add_pending(metadata.clone(), current_time) {
        // Tracker full or locked - permit packet (safe fallback)
        tracing::warn!("Failed to track packet {}: {}", metadata.packet_id, e);
        return;
    }

    // Query daemon for decision
    if let Some(ref pipe) = PIPE_CLIENT {
        if let Some(decision) = pipe.query_decision(&metadata) {
            // Apply decision and remove from tracker
            if let Ok(Some(packet)) = PACKET_TRACKER.apply_decision(decision.clone()) {
                *action = match decision {
                    PacketDecision::Permit { .. } => {
                        FWP_ACTION_PERMIT.0
                    }
                    PacketDecision::Drop { .. } => {
                        // TODO: Inject ICMP/RST response here
                        FWP_ACTION_BLOCK.0
                    }
                };
            }
        }
    }

    // Periodically cleanup expired packets (every 100 packets approx)
    // TODO: Add cleanup trigger logic
}
```

---

## Phase 2: Add Packet Injector (For Blocking)

### 2.1 Create Global Packet Injector

```rust
// Add to callout.rs:
use crate::packet_injector::PacketInjector;

lazy_static! {
    static ref PACKET_INJECTOR: Mutex<PacketInjector> = {
        Mutex::new(PacketInjector::new())
    };
}
```

### 2.2 Apply Drop Decision with Injection

**In classify_callback, when decision is Drop:**
```rust
PacketDecision::Drop { .. } => {
    // Attempt to inject response (non-blocking)
    if let Ok(mut injector_guard) = PACKET_INJECTOR.try_lock() {
        let _ = injector_guard.apply_drop_decision(
            &metadata,
            false, // don't disable ICMP
        );
    }
    // Whether injection succeeds or not, block the packet
    FWP_ACTION_BLOCK.0
}
```

---

## Phase 3: Async Decision Handling (ADVANCED)

### 3.1 Implement Deferred Decision Processing

If daemon is slow, decisions may arrive AFTER classify_callback returns.

**Solution:** Use tracker as deferral queue
```rust
// When decision arrives (in a separate thread/task):
if let Ok(Some(packet)) = PACKET_TRACKER.apply_decision(decision) {
    // Packet was found! Apply decision now (via WFP re-injection API)
    // This is async - packet already passed first filter
    // TODO: Use FwpsStreamInjectAsync0 to apply decision
}
```

---

## Phase 4: Error Handling & Robustness

### 4.1 Handle Daemon Connection Loss

```rust
// At DLL load (DllMain):
// If PIPE_CLIENT connection fails:
// - Set PIPE_CLIENT = None
// - Continue without limiting (permit all)
// - Log warning: "Daemon not available"

// On classify_callback:
// - If PIPE_CLIENT is None, permit all packets
// - No errors, just graceful degradation
```

### 4.2 Handle Tracker Capacity

```rust
// If tracker.add_pending() fails:
// - Permit packet (don't break normal traffic)
// - Log warning: "Tracker capacity exceeded"
// - Trigger cleanup_expired() to free space
```

### 4.3 Periodic Maintenance

```rust
// Add a counter to classify_callback
static CLEANUP_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Every 100 packets, cleanup expired
if CLEANUP_COUNTER.fetch_add(1, Ordering::Relaxed) % 100 == 0 {
    let removed = PACKET_TRACKER.cleanup_expired(get_current_micros());
    if removed > 10 {
        tracing::info!("Cleaned up {} expired packets", removed);
    }
}
```

---

## Phase 5: Testing (Verify Each Phase)

### 5.1 Unit Test: Tracker Integration
```rust
#[test]
fn test_classify_callback_with_tracker() {
    // Mock WFP data structures
    // Call classify_callback
    // Verify tracker has pending packet
    // Verify decision is applied
}
```

### 5.2 Integration Test: Daemon Communication
```rust
#[test]
#[ignore] // Requires daemon running
fn test_classify_callback_with_daemon() {
    // Start daemon subprocess
    // Call classify_callback
    // Verify decision received from daemon
    // Verify action is PERMIT or BLOCK as expected
}
```

### 5.3 Load Test: Concurrent Packets
```rust
#[test]
fn test_classify_callback_under_load() {
    // Simulate 1000 concurrent packets
    // Verify no panics, no memory leaks
    // Verify tracker capacity managed
}
```

---

## Cargo.toml Dependencies Check

**Required additions to wfp-callout/Cargo.toml:**
```toml
lazy_static = "1.4"
```

**Already available:**
```toml
proto = { path = "../proto" }
bincode = "1.3"
windows = { version = "0.52", features = [...] }
thiserror = { workspace = true }
tracing = { workspace = true }
```

---

## Success Criteria (Definition of Done)

- [ ] packet_tracker.rs compiles and tests pass
- [ ] packet_injector.rs compiles and tests pass
- [ ] callout.rs updated with tracker integration
- [ ] classify_callback handles concurrent packets
- [ ] Drop decisions trigger injection
- [ ] Permit decisions allow packets through
- [ ] Timeout cleanup prevents memory leaks
- [ ] Daemon connection loss handled gracefully
- [ ] Tracker capacity managed safely
- [ ] All error cases return to safe state (PERMIT)
- [ ] Code commented re: kernel-safety constraints
- [ ] Unit tests: 100% pass
- [ ] Integration tests: pass with daemon running

---

## Time Estimates

| Phase | Task | Time |
|-------|------|------|
| 1 | Tracker integration | 2-4 hours |
| 2 | Packet injector | 3-5 hours |
| 3 | Async decision handling | 4-6 hours (advanced) |
| 4 | Error handling | 2-3 hours |
| 5 | Testing | 4-6 hours |
| **TOTAL** | | **15-24 hours** |

Recommend: 
- Phases 1-4 = must-have (M1)
- Phase 3 = optional for M1, can defer
- Phase 5 = continuous throughout

---

## Dependencies on Aksh's Work

**CRITICAL PATH:**
1. ✅ M4 Phase 3 (Daemon IPC) - COMPLETE
2. ✅ Proto structures - READY
3. ⏳ M1 (Saksham) - NOW READY TO START
4. ⏳ M5 Phase 4 (UI integration) - Waits for M1

---

## Questions During Implementation?

Check:
1. **Kernel-mode safety:**
   - Can't use `SystemTime` - use Windows `QueryPerformanceCounter` instead
   - Can't allocate in callback - use pre-allocated buffers
   - Comments mark all `unsafe` blocks

2. **WFP API details:**
   - Comments in callout.rs explain field indices
   - FWP_VALUE0 union access examples provided
   - Injection API stubs in packet_injector.rs

3. **Test failures:**
   - Unit tests self-contained (no daemon needed)
   - Integration tests marked `#[ignore]` for CI
   - Manual Windows testing documented in M1_WFP_PACKET_EXTRACTION.md

---

**Status Update: Ready for Saksham to begin Phase 1** ✅
