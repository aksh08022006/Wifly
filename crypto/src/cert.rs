/// Self-signed Certificate Generation
/// ====================================
/// Uses rcgen to generate Ed25519 certs for device enrollment

use rcgen::{generate_simple_self_signed, Certificate};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Certificate generation error: {0}")]
    RcgenError(String),

    #[error("Certificate already exists")]
    AlreadyExists,
}

/// Bundle containing a certificate and its PEM-encoded representations
pub struct CertBundle {
    pub cert: Certificate,
    pub cert_pem: String,
    pub key_pem: String,
}

impl CertBundle {
    /// Generate a new self-signed certificate
    /// Subject: CN=netshaper-ca, validity: 10 years
    pub fn generate_self_signed() -> Result<Self, CertError> {
        // TODO: Use rcgen to generate Ed25519 self-signed certificate
        // Valid for 10 years with CN=netshaper-ca
        Err(CertError::RcgenError(
            "Certificate generation not yet implemented".to_string(),
        ))
    }

    /// Save certificate and key to disk
    /// Creates directory if it doesn't exist
    /// Fails if files already exist
    pub fn save_to_disk(&self, _dir: &Path) -> Result<(), CertError> {
        // TODO: Implement save logic
        // Create ~/.netshaper/ directory
        // Write ca.pem and ca.key
        Ok(())
    }

    /// Load certificate from disk, or generate and save if missing
    pub fn load_or_generate(_dir: &Path) -> Result<Self, CertError> {
        // TODO: Implement load logic
        // Check for existing ca.pem and ca.key
        // If missing, generate and save
        // If present, load from disk
        Err(CertError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not yet implemented",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_generation() {
        // TODO: Test certificate generation once implemented
        // let bundle = CertBundle::generate_self_signed();
        // assert!(bundle.is_ok());
    }
}
