# M1: Saksham's WFP Packet Extraction - Implementation Guide

**Status:** Ready for implementation  
**Date:** April 3, 2026  
**Target:** Complete WFP packet interception and daemon communication

---

## Overview

Saksham's M1 task focuses on extracting packets from Windows Filtering Platform (WFP) at the kernel level and communicating with the daemon for bandwidth decisions.

### Key Responsibilities

**Part 1: Packet Extraction**
- Intercept packets at WFP filter layer (kernel-mode)
- Extract metadata: source IP, destination IP, packet size, unique ID
- Send to daemon via named pipe (`\\.\pipe\netshaper`)
- Receive PacketDecision (Permit or Drop)

**Part 2: Decision Application**
- If **Permit** → Release packet (allow through network)
- If **Drop** → Inject RST (TCP reset) or ICMP (unreachable)
- Handle concurrent packets (different packet_ids)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Windows Kernel                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ WFP Filter Layer (FWPM_LAYER_OUTBOUND_IPPACKET_V4)  │  │
│  │  ↓ (Every packet triggers classify_callback)         │  │
│  │ ┌─────────────────────────────────────────────────┐  │  │
│  │ │ classify_callback (callout.rs)                  │  │  │
│  │ │  • Extract packet metadata (IPs, size, ID)     │  │  │
│  │ │  • Query daemon for decision                    │  │  │
│  │ │  • Apply decision (PERMIT or BLOCK)             │  │  │
│  │ └─────────────────────────────────────────────────┘  │  │
│  │  ↓ (named pipe: \\.\pipe\netshaper)                  │  │
│  └───────────────────────────────────────────────────────┘  │
│
│ [kernel-mode WFP]  ↔ [user-mode IPC pipe] ↔ [daemon]
│
└─────────────────────────────────────────────────────────────┘
         ↓ (packets allowed/blocked at network driver)
      Real Network Traffic
```

---

## Code Structure

### Modules Ready for Saksham

1. **packet_tracker.rs** (NEW)
   - Tracks packets pending daemon decision
   - Thread-safe (Mutex-protected)
   - Handles timeout cleanup
   - Concurrent packet management
   - Usage: Store context for packets awaiting responses

2. **packet_injector.rs** (NEW)
   - Builds and injects RST/ICMP responses
   - Constructs IPv4 and TCP headers
   - Integrates with WFP injection API
   - Usage: Send blocking responses to senders

3. **callout.rs** (EXISTING - TO ENHANCE)
   - `classify_callback()` - Main packet interception point
   - `extract_packet_metadata()` - Parse WFP fields
   - `extract_ipv4()` - Convert WFP uint32 to Ipv4Addr
   - Usage: Called by WFP for EVERY packet

4. **pipe.rs** (EXISTING - READY)
   - `PipeClient::connect()` - Connect to daemon
   - `query_decision()` - Send metadata, receive decision
   - Bincode serialization/deserialization
   - Usage: IPC communication with daemon

5. **engine.rs** (EXISTING - MINIMAL)
   - WFP engine lifecycle management
   - TODO: Implement FwpmEngineOpen0
   - Usage: Initialize/cleanup WFP session

---

## Implementation Tasks

### Task 1: Enhance classify_callback for Concurrent Packet Handling

**Current Implementation:**
- Extracts single packet metadata
- Queries daemon for decision
- Returns PERMIT/BLOCK action

**Enhancements Needed:**
```rust
// In callout.rs:
// 1. Add a global PacketTracker instance
static PACKET_TRACKER: Lazy<PacketTracker> = Lazy::new(|| {
    PacketTracker::new(2000, 1_000_000) // 2000 packets, 1s timeout
});

// 2. In classify_callback, after extracting metadata:
// - Add to tracker
// - Get immediate decision OR defer processing
// - If Drop → inject response
// - If Permit → allow packet

// 3. Handle async decision delivery:
// - Daemon can respond asynchronously
// - Use packet_id to match request/response
// - Track pending packets with timeout cleanup
```

### Task 2: Implement RST/ICMP Injection

**When to Inject:**
- PacketDecision::Drop received
- Instead of silently blocking, send notification back to sender
- Helps applications detect blocked connections faster

**Implementation Options:**

**Option A: TCP RST (for TCP connections)**
```rust
// Send TCP RST from (dst_ip:dst_port) → (src_ip:src_port)
// Requires: extracting TCP ports from original packet
// Effect: Closes TCP connection cleanly
injector.inject_tcp_reset(&metadata, src_port, dst_port)?;
```

**Option B: ICMP Destination Unreachable (any protocol)**
```rust
// Send ICMP Type 3, Code 13 (Admin Prohibited)
// From: daemon's IP → attacker's IP
// Effect: Indicates destination refuses connection
injector.inject_icmp_unreachable(&metadata)?;
```

**Recommended for M1:**
- Start with ICMP (works for all protocols)
- Later add TCP RST extraction when port info is available

### Task 3: Handle Edge Cases

**Malformed Packets:**
```rust
// If extract_packet_metadata() fails:
// - Permit (safe default, don't break normal traffic)
// - Log error for debugging
```

**Daemon Not Running:**
```rust
// If pipe connection fails:
// - Set PIPE_CLIENT = None
// - Permit all packets (no limiting available)
// - Log warning
```

**Timeout Packets:**
```rust
// If daemon doesn't respond within 1 second:
// - Call tracker.cleanup_expired()
// - Remove old pending packets
// - Permit them (no response = allow through)
```

**Capacity Exceeded:**
```rust
// If >2000 packets awaiting decision:
// - Reject adding new packets
// - Permit them instead (safe fallback)
// - Log warning about overload
```

---

## Data Flow Example

```
Scenario: Drop malicious traffic to 10.0.0.50:22 (SSH scan)

1. Packet arrives (src=192.168.1.100, dst=10.0.0.50, len=60)
   └→ WFP triggers classify_callback()

2. Extract metadata:
   PacketMetadata {
     src_ip: 192.168.1.100,
     dst_ip: 10.0.0.50,
     byte_len: 60,
     packet_id: 0x7fff0123,
   }

3. Query daemon via named pipe:
   pipe.query_decision(&metadata)?
   
4. Daemon responds:
   PacketDecision::Drop {
     packet_id: 0x7fff0123,
   }

5. Apply decision:
   a) Remove from tracker
   b) Inject ICMP "Destination Unreachable"
   c) Return action = FWP_ACTION_BLOCK (kernel won't forward)

6. Result: Attacker gets ICMP response, knows port is blocked
           Network driver blocks packet at L3
```

---

## Testing Checklist

**Unit Tests (existing - should pass):**
- ✅ packet_tracker tests
- ✅ packet_injector tests  
- ✅ callout IPv4 extraction
- ✅ pipe serialization

**Integration Tests (Saksham to create):**
- [ ] Multiple concurrent packets (1000+)
- [ ] Permit/Drop decision application
- [ ] Timeout cleanup
- [ ] Malformed packet handling
- [ ] Daemon connection loss recovery
- [ ] RST injection (if implemented)
- [ ] ICMP injection (if implemented)

**Manual Testing on Windows:**
- [ ] Connect WFP engine to listen
- [ ] Run daemon on localhost
- [ ] Generate test traffic
- [ ] Verify permits work
- [ ] Verify drops work
- [ ] Check ICMP/RST responses

---

## API Reference

### PacketTracker

```rust
// Create tracker (max 1000 packets, 1s timeout)
let tracker = PacketTracker::new(1000, 1_000_000);

// Record new packet
tracker.add_pending(metadata, current_time_micros)?;

// Process decision
tracker.apply_decision(decision)?;

// Cleanup expired
tracker.cleanup_expired(current_time_micros)?;

// Check status
tracker.is_pending(packet_id)?;
tracker.pending_count()?;
```

### PacketInjector

```rust
// Create injector
let mut injector = PacketInjector::new();

// Inject responses for drops
injector.inject_tcp_reset(&metadata, src_port, dst_port)?;
injector.inject_icmp_unreachable(&metadata)?;

// Apply decision with fallback
injector.apply_drop_decision(&metadata, disable_icmp)?;
```

### PipeClient (existing)

```rust
// Already implemented in pipe.rs
// Available through: callout::PIPE_CLIENT (static)

if let Some(ref pipe) = PIPE_CLIENT {
    if let Some(decision) = pipe.query_decision(&metadata) {
        // Got response - apply decision
    }
}
```

---

## Expected Outcome

After completing M1, Saksham will have:

✅ **Packet Extraction:** Every packet is successfully extracted, serialized, sent to daemon
✅ **Daemon Communication:** Decisions received and applied correctly
✅ **Permit Handling:** Allowed packets flow through network
✅ **Drop Handling:** Blocked packets are ICMP/RST notified and prevented
✅ **Concurrency:** Multiple packets handled simultaneously
✅ **Robustness:** Timeouts, edge cases, errors handled gracefully
✅ **Tests:** Unit and integration tests passing
✅ **Documentation:** Code comments explain kernel-safety requirements

---

## Integration with Other Milestones

**Depends On:**
- ✅ M4 Phase 3 (Daemon IPC - Aksh) - READY
- ✅ Proto structures (PacketMetadata, PacketDecision) - READY
- ✅ Named pipe communication - READY

**Enables:**
- ⏳ M5 Phase 4 (Daemon UI integration - Aksh) - Waits for M1 decision stats
- ⏳ M2 Phase 2 (Token bucket enforcement) - Uses decision application

---

## Notes for Saksham

1. **Kernel Safety:**
   - Avoid heap allocation in `classify_callback`
   - Don't take locks that could block
   - Keep callback execution < 10 microseconds
   - Use `unsafe { }` carefully with WFP pointers

2. **Testing Constraints:**
   - Unit tests can run in user-mode
   - Integration tests need WFP driver set up
   - Manual testing requires Windows admin
   - Check existing tests in `packet_tracker.rs` and `packet_injector.rs`

3. **Performance Targets:**
   - Classify callback: < 10 µs per packet
   - Tracker lock contention: minimize with lock scope
   - Pipe I/O: < 100 µs (async if possible)
   - Injection: fire-and-forget, don't block callback

4. **Code Review Checklist:**
   - [ ] All `unsafe` blocks justified with comments
   - [ ] No panics in callback (return safe default)
   - [ ] Locks held for minimal duration
   - [ ] Timeout cleanup prevents memory leak
   - [ ] Tests document expected behavior

---

## Questions?

Refer to:
- `daemon/` for daemon-side implementation (M4)
- `proto/lib.rs` for message formats
- `wfp-callout/src/` for existing infrastructure
- Comments in newly created files: `packet_tracker.rs`, `packet_injector.rs`

**Good luck, Saksham! M1 is critical infrastructure for all bandwidth limiting.** 🚀
