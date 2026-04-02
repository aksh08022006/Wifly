/// Packet Tracker
/// ==============
/// Manages pending packets awaiting daemon decision.
/// Matches responses by packet_id and applies decisions.
/// SAFETY: Thread-safe for concurrent packet processing.

use proto::{PacketMetadata, PacketDecision};
use std::collections::HashMap;
use std::sync::Mutex;

/// Context for a pending packet
#[derive(Debug, Clone)]
pub struct PendingPacket {
    pub metadata: PacketMetadata,
    /// Optional: packet decision (filled in when response arrives)
    pub decision: Option<PacketDecision>,
    /// Time when packet was recorded (for timeout detection)
    pub timestamp_micros: u64,
}

/// Thread-safe tracker for packets awaiting decisions
pub struct PacketTracker {
    /// Map of packet_id -> PendingPacket
    /// packet_id is derived from the context pointer (unique per packet)
    pending: Mutex<HashMap<u64, PendingPacket>>,
    
    /// Maximum number of concurrent packets we'll track
    /// If exceeded, newest packets are dropped (permit as fallback)
    max_pending: usize,
    
    /// Timeout in microseconds (default: 1 second)
    timeout_micros: u64,
}

impl PacketTracker {
    /// Create a new packet tracker
    pub fn new(max_pending: usize, timeout_micros: u64) -> Self {
        PacketTracker {
            pending: Mutex::new(HashMap::new()),
            max_pending,
            timeout_micros,
        }
    }

    /// Default tracker: max 1000 packets, 1 second timeout
    pub fn default() -> Self {
        Self::new(1000, 1_000_000)
    }

    /// Record a new packet as pending
    /// Returns the decision if one is immediately available, None otherwise
    pub fn add_pending(
        &self,
        metadata: PacketMetadata,
        current_time_micros: u64,
    ) -> Result<(), &'static str> {
        let mut pending = self.pending.lock().map_err(|_| "Failed to acquire lock")?;

        // Check if we're at capacity
        if pending.len() >= self.max_pending {
            // Too many pending packets - drop this one (permit as safe fallback)
            return Err("Too many pending packets");
        }

        // Insert the pending packet
        pending.insert(
            metadata.packet_id,
            PendingPacket {
                metadata,
                decision: None,
                timestamp_micros: current_time_micros,
            },
        );

        Ok(())
    }

    /// Apply a decision to a pending packet
    /// Returns the full PendingPacket if found, None otherwise
    pub fn apply_decision(
        &self,
        decision: PacketDecision,
    ) -> Result<Option<PendingPacket>, &'static str> {
        let mut pending = self.pending.lock().map_err(|_| "Failed to acquire lock")?;

        let packet_id = match &decision {
            PacketDecision::Permit { packet_id } => *packet_id,
            PacketDecision::Drop { packet_id } => *packet_id,
        };

        if let Some(packet) = pending.remove(&packet_id) {
            Ok(Some(packet))
        } else {
            // Packet not found (may have timed out or already processed)
            Ok(None)
        }
    }

    /// Clean up expired packets (return count of removed packets)
    pub fn cleanup_expired(
        &self,
        current_time_micros: u64,
    ) -> Result<usize, &'static str> {
        let mut pending = self.pending.lock().map_err(|_| "Failed to acquire lock")?;

        let before_count = pending.len();

        // Remove packets that have exceeded the timeout
        pending.retain(|_, packet| {
            let elapsed = current_time_micros.saturating_sub(packet.timestamp_micros);
            elapsed < self.timeout_micros
        });

        Ok(before_count - pending.len())
    }

    /// Get current number of pending packets
    pub fn pending_count(&self) -> Result<usize, &'static str> {
        let pending = self.pending.lock().map_err(|_| "Failed to acquire lock")?;
        Ok(pending.len())
    }

    /// Check if a packet is still pending (not yet decided)
    pub fn is_pending(&self, packet_id: u64) -> Result<bool, &'static str> {
        let pending = self.pending.lock().map_err(|_| "Failed to acquire lock")?;
        Ok(pending.contains_key(&packet_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn create_test_metadata(packet_id: u64, byte_len: u32) -> PacketMetadata {
        PacketMetadata {
            src_ip: Ipv4Addr::new(192, 168, 1, 100),
            dst_ip: Ipv4Addr::new(8, 8, 8, 8),
            byte_len,
            packet_id,
        }
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = PacketTracker::default();
        assert_eq!(tracker.pending_count().unwrap(), 0);
    }

    #[test]
    fn test_add_pending_packet() {
        let tracker = PacketTracker::default();
        let metadata = create_test_metadata(1001, 1500);

        assert!(tracker.add_pending(metadata, 0).is_ok());
        assert_eq!(tracker.pending_count().unwrap(), 1);
        assert!(tracker.is_pending(1001).unwrap());
    }

    #[test]
    fn test_apply_permit_decision() {
        let tracker = PacketTracker::default();
        let metadata = create_test_metadata(2001, 1500);

        tracker.add_pending(metadata, 0).unwrap();
        assert!(tracker.is_pending(2001).unwrap());

        let decision = PacketDecision::Permit { packet_id: 2001 };
        let result = tracker.apply_decision(decision).unwrap();

        assert!(result.is_some());
        assert!(!tracker.is_pending(2001).unwrap());
    }

    #[test]
    fn test_apply_drop_decision() {
        let tracker = PacketTracker::default();
        let metadata = create_test_metadata(3001, 1024);

        tracker.add_pending(metadata.clone(), 0).unwrap();

        let decision = PacketDecision::Drop { packet_id: 3001 };
        let result = tracker.apply_decision(decision).unwrap();

        assert!(result.is_some());
        let packet = result.unwrap();
        assert_eq!(packet.metadata.packet_id, 3001);
        assert_eq!(packet.metadata.byte_len, 1024);
    }

    #[test]
    fn test_apply_decision_not_found() {
        let tracker = PacketTracker::default();

        let decision = PacketDecision::Permit { packet_id: 9999 };
        let result = tracker.apply_decision(decision).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_cleanup_expired() {
        let tracker = PacketTracker::new(100, 1000); // 1000 micros timeout

        // Add packet at time 0
        let metadata = create_test_metadata(4001, 1500);
        tracker.add_pending(metadata, 0).unwrap();

        // Check at time 500 (not expired)
        let removed = tracker.cleanup_expired(500).unwrap();
        assert_eq!(removed, 0);
        assert!(tracker.is_pending(4001).unwrap());

        // Check at time 1500 (expired)
        let removed = tracker.cleanup_expired(1500).unwrap();
        assert_eq!(removed, 1);
        assert!(!tracker.is_pending(4001).unwrap());
    }

    #[test]
    fn test_max_pending_capacity() {
        let tracker = PacketTracker::new(2, 1_000_000);

        // Add first packet
        assert!(tracker.add_pending(create_test_metadata(1, 1500), 0).is_ok());

        // Add second packet
        assert!(tracker.add_pending(create_test_metadata(2, 1500), 0).is_ok());

        // Third packet should fail (capacity exceeded)
        assert!(tracker.add_pending(create_test_metadata(3, 1500), 0).is_err());

        // Should still have 2 packets
        assert_eq!(tracker.pending_count().unwrap(), 2);
    }

    #[test]
    fn test_multiple_concurrent_packets() {
        let tracker = PacketTracker::default();

        // Simulate multiple packets arriving
        for i in 1..=10 {
            let metadata = create_test_metadata(i as u64, 1500);
            assert!(tracker.add_pending(metadata, 0).is_ok());
        }

        assert_eq!(tracker.pending_count().unwrap(), 10);

        // Apply decisions to half of them
        for i in 1..=5 {
            let decision = if i % 2 == 0 {
                PacketDecision::Permit { packet_id: i as u64 }
            } else {
                PacketDecision::Drop { packet_id: i as u64 }
            };
            tracker.apply_decision(decision).unwrap();
        }

        assert_eq!(tracker.pending_count().unwrap(), 5);
    }
}
