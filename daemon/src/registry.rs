use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Device information for pairing and management
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub ip: Ipv4Addr,
    pub platform: String,
    pub device_name: String,
    pub approved: bool,
    pub bandwidth_limit: u64,  // bytes per second
    pub enrolled_at: DateTime<Utc>,
}

/// Device statistics for dashboard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceStats {
    pub id: String,
    pub ip: String,
    pub device_name: String,
    pub platform: String,
    pub approved: bool,
    pub current_usage: u64,    // bytes/sec
    pub peak_usage: u64,       // bytes/sec
    pub total_consumed: u64,   // bytes (session total)
    pub bandwidth_limit: u64,  // bytes/sec
}

/// QR Code pairing data
#[derive(Clone, Debug)]
pub struct QrCodeData {
    pub token: String,
    pub created_at: DateTime<Utc>,
}

/// Events broadcast to dashboard via SSE
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DashboardEvent {
    #[serde(rename = "device_pending")]
    DevicePending { device: Device },
    
    #[serde(rename = "device_approved")]
    DeviceApproved { device_id: String, device: Device },
    
    #[serde(rename = "device_denied")]
    DeviceDenied { device_id: String },
    
    #[serde(rename = "stats_update")]
    StatsUpdate { devices: Vec<DeviceStats> },
}

/// Commands for WFP bandwidth throttling
#[derive(Clone, Debug)]
pub enum BandwidthCommand {
    Update { ip: Ipv4Addr, bytes_per_sec: u64 },
    Block { ip: Ipv4Addr },
    Unblock { ip: Ipv4Addr },
}

/// Device registry holding all paired devices
pub struct DeviceRegistry {
    devices: Vec<Device>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    /// Add a new pending device (called after phone pair POST)
    pub fn add_pending_device(
        &mut self,
        ip: Ipv4Addr,
        platform: String,
        device_name: String,
    ) -> Device {
        let device = Device {
            id: Uuid::new_v4().to_string(),
            ip,
            platform,
            device_name,
            approved: false,
            bandwidth_limit: 10_000_000,  // 10 MB/s default
            enrolled_at: Utc::now(),
        };
        self.devices.push(device.clone());
        device
    }

    /// Approve a device for internet access
    pub fn approve_device(&mut self, device_id: &str, bandwidth_limit: u64) -> Option<Device> {
        for device in &mut self.devices {
            if device.id == device_id {
                device.approved = true;
                device.bandwidth_limit = bandwidth_limit;
                return Some(device.clone());
            }
        }
        None
    }

    /// Deny/block a device
    pub fn deny_device(&mut self, device_id: &str) -> bool {
        self.devices.retain(|d| d.id != device_id);
        true
    }

    /// Update bandwidth limit for device
    pub fn update_bandwidth(&mut self, device_id: &str, bytes_per_sec: u64) -> Option<Device> {
        for device in &mut self.devices {
            if device.id == device_id {
                device.bandwidth_limit = bytes_per_sec;
                return Some(device.clone());
            }
        }
        None
    }

    /// Get all devices
    pub fn get_all(&self) -> Vec<Device> {
        self.devices.clone()
    }

    /// Get all approved devices
    pub fn get_approved(&self) -> Vec<Device> {
        self.devices.iter().filter(|d| d.approved).cloned().collect()
    }

    /// Get device by ID
    pub fn get_by_id(&self, device_id: &str) -> Option<Device> {
        self.devices.iter().find(|d| d.id == device_id).cloned()
    }

    /// Get device by IP address
    pub fn get_by_ip(&self, ip: Ipv4Addr) -> Option<Device> {
        self.devices.iter().find(|d| d.ip == ip).cloned()
    }
}
