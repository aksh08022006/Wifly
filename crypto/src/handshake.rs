use crate::CertBundle;
/// Device Consent & Enrollment Server
/// ====================================
/// Runs a TLS server that handles device enrollment
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tracks enrolled device IPs
pub struct EnrolledDevices {
    ips: Vec<std::net::Ipv4Addr>,
}

impl EnrolledDevices {
    pub fn new() -> Self {
        Self { ips: Vec::new() }
    }

    pub fn add(&mut self, ip: std::net::Ipv4Addr) {
        if !self.ips.contains(&ip) {
            self.ips.push(ip);
        }
    }

    pub fn is_enrolled(&self, ip: std::net::Ipv4Addr) -> bool {
        self.ips.contains(&ip)
    }
}

impl Default for EnrolledDevices {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the device enrollment server
/// Listens on 0.0.0.0:7979 with TLS
pub async fn run_consent_server(
    _cert: CertBundle,
    _enrolled: Arc<Mutex<EnrolledDevices>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement TLS server using tokio-rustls
    // Listen on 0.0.0.0:7979
    // Serve enrollment HTML page
    // Handle /accept and /decline POST endpoints
    // Persist enrolled devices to ~/.netshaper/devices.json

    // tracing::info!("Consent server would start on 0.0.0.0:7979");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrolled_devices_tracking() {
        let mut enrolled = EnrolledDevices::new();
        let ip: std::net::Ipv4Addr = "192.168.1.100".parse().unwrap();

        assert!(!enrolled.is_enrolled(ip));
        enrolled.add(ip);
        assert!(enrolled.is_enrolled(ip));
    }
}
