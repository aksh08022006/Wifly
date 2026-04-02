use crate::bucket::DeviceBucket;
/// Device Registry
/// ================
/// Manages all enrolled devices and their bandwidth buckets
use std::collections::HashMap;
use std::net::Ipv4Addr;

/// Registry of all managed devices
pub struct DeviceRegistry {
    devices: HashMap<Ipv4Addr, DeviceBucket>,
}

impl DeviceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Insert or update a device's bandwidth ceiling
    pub fn insert_device(&mut self, ip: Ipv4Addr, bytes_per_sec: u64) {
        self.devices.insert(ip, DeviceBucket::new(bytes_per_sec));
    }

    /// Get a mutable reference to a device's bucket
    pub fn get_bucket_mut(&mut self, ip: Ipv4Addr) -> Option<&mut DeviceBucket> {
        self.devices.get_mut(&ip)
    }

    /// Get an immutable reference to a device's bucket
    pub fn get_bucket(&self, ip: Ipv4Addr) -> Option<&DeviceBucket> {
        self.devices.get(&ip)
    }

    /// Remove a device from the registry
    pub fn remove_device(&mut self, ip: Ipv4Addr) -> Option<DeviceBucket> {
        self.devices.remove(&ip)
    }

    /// List all device IPs
    pub fn list_devices(&self) -> Vec<Ipv4Addr> {
        self.devices.keys().copied().collect()
    }

    /// Update bandwidth for a device
    pub fn update_bandwidth(&mut self, ip: Ipv4Addr, bytes_per_sec: u64) {
        if let Some(bucket) = self.devices.get_mut(&ip) {
            bucket.allowed_bytes_per_sec = bytes_per_sec;
        }
    }

    /// Get the number of managed devices
    pub fn count(&self) -> usize {
        self.devices.len()
    }

    /// Get current usage for a device (bytes in active 1-second window)
    pub fn get_current_usage(&self, ip: Ipv4Addr) -> u64 {
        self.devices
            .get(&ip)
            .map(|b| b.get_current_usage())
            .unwrap_or(0)
    }

    /// Get peak usage for a device
    pub fn get_peak_usage(&self, ip: Ipv4Addr) -> u64 {
        self.devices
            .get(&ip)
            .map(|b| b.get_peak_usage())
            .unwrap_or(0)
    }

    /// Get total bytes consumed for a device
    pub fn get_total_consumption(&self, ip: Ipv4Addr) -> u64 {
        self.devices
            .get(&ip)
            .map(|b| b.get_total_consumption())
            .unwrap_or(0)
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut registry = DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 1_000_000);
        assert!(registry.get_bucket(ip).is_some());
    }

    #[test]
    fn test_remove_device() {
        let mut registry = DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 1_000_000);
        assert!(registry.remove_device(ip).is_some());
        assert!(registry.get_bucket(ip).is_none());
    }

    #[test]
    fn test_list_devices() {
        let mut registry = DeviceRegistry::new();
        let ip1: Ipv4Addr = "192.168.1.100".parse().unwrap();
        let ip2: Ipv4Addr = "192.168.1.101".parse().unwrap();

        registry.insert_device(ip1, 1_000_000);
        registry.insert_device(ip2, 2_000_000);

        let devices = registry.list_devices();
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&ip1));
        assert!(devices.contains(&ip2));
    }

    #[test]
    fn test_update_bandwidth() {
        let mut registry = DeviceRegistry::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        registry.insert_device(ip, 1_000_000);
        registry.update_bandwidth(ip, 2_000_000);

        let bucket = registry.get_bucket(ip).unwrap();
        assert_eq!(bucket.allowed_bytes_per_sec, 2_000_000);
    }
}
