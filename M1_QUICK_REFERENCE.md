# M1 Quick Reference - Saksham's Desk Card

## What You're Building

**WFP Packet Extraction & Daemon Communication**

Extract packets from Windows kernel → Send to daemon for bandwidth decision → Apply permit/drop

---

## 3 New Files You Have

### 1. `packet_tracker.rs` - Track Pending Packets
```rust
// Create once globally
static PACKET_TRACKER: PacketTracker = PacketTracker::new(2000, 1_000_000);

// Add packet when it arrives
PACKET_TRACKER.add_pending(metadata, current_time_micros)?;

// Remove & process when decision arrives
let packet = PACKET_TRACKER.apply_decision(decision)?;

// Cleanup expired (call periodically)
PACKET_TRACKER.cleanup_expired(current_time_micros)?;
```

**Purpose:** Thread-safe packet queue for concurrent handling

---

### 2. `packet_injector.rs` - Send RST/ICMP
```rust
// Create once globally
static PACKET_INJECTOR: PacketInjector = PacketInjector::new();

// When decision is DROP:
injector.inject_icmp_unreachable(&metadata)?;  // Send ICMP Host Unreachable
injector.inject_tcp_reset(&metadata, src_port, dst_port)?;  // Send TCP RST

// Result: Attacker gets immediate response, doesn't timeout waiting
```

**Purpose:** Notify blocked traffic, don't silently drop

---

### 3. `callout.rs` (ENHANCE) - Main Packet Handler
```rust
// This runs for EVERY packet, must be FAST (<10µs)
pub unsafe extern "system" fn classify_callback(...) {
    // 1. Extract metadata from WFP
    let metadata = extract_packet_metadata(meta_values, context)?;
    
    // 2. Query daemon for decision
    if let Some(decision) = pipe.query_decision(&metadata) {
        // 3. Apply it
        match decision {
            Permit => *action = FWP_ACTION_PERMIT,
            Drop => {
                injector.inject_icmp_unreachable(&metadata)?;
                *action = FWP_ACTION_BLOCK;
            }
        }
    }
}
```

---

## Integration Checklist

- [ ] Add `lazy_static = "1.4"` to `Cargo.toml` dependencies
- [ ] Add module declarations to `lib.rs`
  ```rust
  mod packet_tracker;
  mod packet_injector;
  ```
- [ ] Update `classify_callback()` to use tracker
- [ ] Test with `cargo test --lib`
- [ ] Build kernel DLL: `cargo build --release`

---

## Key Constraints (Kernel Mode ⚠️)

| Constraint | What NOT To Do | Alternative |
|-----------|----------------|-------------|
| **Speed** | Don't allocate/lock in callback | Use pre-allocated buffers, try_lock |
| **Stability** | Don't panic | Return default (PERMIT) |
| **Memory** | Don't heap allocate | Stack-only structures |
| **Concurrency** | Don't deadlock | Use Mutex + timeout cleanup |
| **Time** | Don't call SystemTime | Use Windows performance counter |

---

## Quick Test Commands

```powershell
# Unit tests (no daemon needed)
cargo test --lib packet_tracker
cargo test --lib packet_injector

# Full build (produces wfp-callout.dll)
cargo build --release --lib

# Check for kernel-safety issues
cargo clippy --lib
```

---

## Daemon Communication (Already Works)

```rust
// In pipe.rs - just call this:
if let Some(ref pipe) = PIPE_CLIENT {
    if let Some(decision) = pipe.query_decision(&metadata) {
        // Got: PacketDecision::Permit { packet_id } or Drop { packet_id }
        // Use it!
    }
}
```

**No need to implement pipe logic - it's done in M4!**

---

## Success Criteria

✅ Packet arrives → Extracted
✅ Daemon called → Decision received  
✅ Permit → Packet passes
✅ Drop → ICMP/RST sent + Blocked
✅ 1000+ packets/sec → No crashes
✅ Daemon down → Safe permit all (no block)

---

## File Locations

```
wfp-callout/
├── Cargo.toml (update with dependencies)
├── src/
│   ├── lib.rs (add module declarations)
│   ├── callout.rs (ENHANCE: add tracker integration)
│   ├── packet_tracker.rs (NEW - ready to use)
│   ├── packet_injector.rs (NEW - ready to use)
│   ├── pipe.rs (READY - don't change)
│   └── engine.rs (READY - minimal WFP init)

proto/
└── src/lib.rs (Packet/Decision structs - READY)

daemon/ (Aksh's M4 - listening on \\.\pipe\netshaper)
```

---

## Architecture in One Diagram

```
Packet → classify_callback()
           ├─ extract_packet_metadata()
           ├─ PACKET_TRACKER.add_pending()
           ├─ PIPE_CLIENT.query_decision()
           │   └→ Daemon responds
           ├─ PACKET_TRACKER.apply_decision()
           └─ If Drop: PACKET_INJECTOR.inject_icmp_unreachable()
                       then FWP_ACTION_BLOCK
```

---

## Common Pitfalls 🚨

❌ **Calling SystemTime** in kernel mode → Use QueryPerformanceCounter
❌ **Panicking** in callback → Return safe default  
❌ **Allocating** in callback → Use fixed-size buffers
❌ **Taking locks** → Use try_lock, keep scope tiny
❌ **Forgetting cleanup** → Call periodic cleanup_expired()

---

## When You Get Stuck

**Q: Packet not being intercepted?**
- Check WFP filter is registered in engine.rs
- Verify FwpmEngineOpen0 actually called
- Add tracing::warn!() & check logs

**Q: Tracker filling up?**
- Daemon too slow? Check daemon performance
- Cleanup not running? Add periodic cleanup trigger
- Reduce timeout? Change from 1s to 100ms

**Q: Injection not working?**
- Check user-mode injector works first
- Verify injection_handle initialized
- Fall back to silent drop if injection API fails

**Q: Tests failing?**
- Run individual test: `cargo test packet_tracker::tests::test_name`
- Add println! or tracing::debug!()
- Check all dependencies built

---

## Daily Standup Template

**What I completed:**
- [ ] [ ] Phase X: [Task]

**What I'm working on:**
- [ ] [ ] Phase X: [Task]

**Blockers:**
- [ ] [Issue] - Solution: [plan]

**Confidence:**
- [ ] 🟢 On track | 🟡 Minor issues | 🔴 Blocking issue

---

## Resources

- 📄 M1_WFP_PACKET_EXTRACTION.md - Full design doc
- 📋 M1_IMPLEMENTATION_TODO.md - Detailed checklist
- 💾 proto/src/lib.rs - Message format reference
- 🔧 daemon/ - Daemon listening for your packets
- 📦 wfp-callout/src/ - Your code & examples

---

**Let's get M1 done! 🚀 You've got the modules, now integrate them.**

Questions → Ask Aksh in PR reviews!
