/// Device Enrollment Storage
/// ==========================
/// Persists approved devices to ~/.netshaper/devices.json
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnrollmentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceEnrollment {
    pub ip: Ipv4Addr,
    pub hostname: Option<String>,
    pub approved: bool,
    pub enrolled_at: String, // ISO 8601 timestamp
}

pub struct DeviceList {
    devices: Vec<DeviceEnrollment>,
}

impl DeviceList {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn add(&mut self, ip: Ipv4Addr, approved: bool) {
        // Add or update device
        if let Some(device) = self.devices.iter_mut().find(|d| d.ip == ip) {
            device.approved = approved;
        } else {
            self.devices.push(DeviceEnrollment {
                ip,
                hostname: None,
                approved,
                enrolled_at: chrono::Local::now().to_rfc3339(),
            });
        }
    }

    pub fn approved_devices(&self) -> Vec<Ipv4Addr> {
        self.devices
            .iter()
            .filter(|d| d.approved)
            .map(|d| d.ip)
            .collect()
    }

    pub fn is_approved(&self, ip: Ipv4Addr) -> bool {
        self.devices.iter().any(|d| d.ip == ip && d.approved)
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<(), EnrollmentError> {
        let json = serde_json::to_string_pretty(&self.devices)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_disk(path: &Path) -> Result<Self, EnrollmentError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let json = fs::read_to_string(path)?;
        let devices = serde_json::from_str(&json)?;
        Ok(Self { devices })
    }

    pub fn devices(&self) -> &[DeviceEnrollment] {
        &self.devices
    }
}

impl Default for DeviceList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_device_list_add_and_approve() {
        let mut list = DeviceList::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        list.add(ip, true);

        assert!(list.is_approved(ip));
        assert_eq!(list.approved_devices(), vec![ip]);
    }

    #[test]
    fn test_device_list_deny() {
        let mut list = DeviceList::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        list.add(ip, false);

        assert!(!list.is_approved(ip));
        assert!(list.approved_devices().is_empty());
    }

    #[test]
    fn test_device_list_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("devices.json");

        // Create and save
        let mut list = DeviceList::new();
        let ip1: Ipv4Addr = "192.168.1.100".parse().unwrap();
        let ip2: Ipv4Addr = "192.168.1.101".parse().unwrap();

        list.add(ip1, true);
        list.add(ip2, false);

        list.save_to_disk(&path).unwrap();

        // Load and verify
        let loaded = DeviceList::load_from_disk(&path).unwrap();

        assert!(loaded.is_approved(ip1));
        assert!(!loaded.is_approved(ip2));
        assert_eq!(loaded.approved_devices(), vec![ip1]);
    }

    #[test]
    fn test_device_list_load_nonexistent_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let list = DeviceList::load_from_disk(&path).unwrap();
        assert_eq!(list.devices().len(), 0);
    }

    #[test]
    fn test_device_list_update_existing() {
        let mut list = DeviceList::new();
        let ip: Ipv4Addr = "192.168.1.100".parse().unwrap();

        list.add(ip, false);
        assert!(!list.is_approved(ip));

        // Update to approved
        list.add(ip, true);
        assert!(list.is_approved(ip));

        // Should still only be 1 device
        assert_eq!(list.devices().len(), 1);
    }

    #[test]
    fn test_device_list_multiple_approved() {
        let mut list = DeviceList::new();
        let ip1: Ipv4Addr = "192.168.1.100".parse().unwrap();
        let ip2: Ipv4Addr = "192.168.1.101".parse().unwrap();
        let ip3: Ipv4Addr = "192.168.1.102".parse().unwrap();

        list.add(ip1, true);
        list.add(ip2, false);
        list.add(ip3, true);

        let approved = list.approved_devices();
        assert_eq!(approved.len(), 2);
        assert!(approved.contains(&ip1));
        assert!(approved.contains(&ip3));
        assert!(!approved.contains(&ip2));
    }
}
