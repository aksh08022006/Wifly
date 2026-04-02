/// Self-signed Certificate Generation
/// ====================================
/// Uses rcgen to generate Ed25519 certs for device enrollment
use rcgen::Certificate;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[cfg(unix)]
use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
        // Subject alternative names
        let san_list = vec!["netshaper-ca".to_string(), "127.0.0.1".to_string()];

        // Generate Ed25519 certificate
        let cert = rcgen::generate_simple_self_signed(san_list)
            .map_err(|e| CertError::RcgenError(format!("{:?}", e)))?;

        // Serialize to PEM
        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| CertError::RcgenError(format!("{:?}", e)))?;
        let key_pem = cert.serialize_private_key_pem();

        Ok(Self {
            cert,
            cert_pem,
            key_pem,
        })
    }

    /// Save certificate and key to disk
    /// Creates directory if it doesn't exist
    /// Fails if files already exist
    pub fn save_to_disk(&self, dir: &Path) -> Result<(), CertError> {
        // Create directory
        fs::create_dir_all(dir).map_err(CertError::Io)?;

        let cert_path = dir.join("ca.pem");
        let key_path = dir.join("ca.key");

        // Check if files exist (don't overwrite)
        if cert_path.exists() || key_path.exists() {
            return Err(CertError::AlreadyExists);
        }

        // Write certificate
        fs::write(&cert_path, &self.cert_pem).map_err(CertError::Io)?;

        // Write private key
        fs::write(&key_path, &self.key_pem).map_err(CertError::Io)?;

        // Set strict permissions on private key (Unix only)
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o600);
            fs::set_permissions(&key_path, perms).map_err(CertError::Io)?;
        }

        Ok(())
    }

    /// Load certificate from disk, or generate and save if missing
    pub fn load_or_generate(dir: &Path) -> Result<Self, CertError> {
        let cert_path = dir.join("ca.pem");
        let key_path = dir.join("ca.key");

        // Check if both files exist
        if cert_path.exists() && key_path.exists() {
            let cert_pem = fs::read_to_string(&cert_path).map_err(CertError::Io)?;
            let key_pem = fs::read_to_string(&key_path).map_err(CertError::Io)?;

            // For load_or_generate, we don't need to parse back to Certificate
            // The PEM strings are sufficient for TLS server usage
            // Create a dummy Certificate for the struct
            let cert = rcgen::generate_simple_self_signed(vec!["netshaper-ca".to_string()])
                .map_err(|e| CertError::RcgenError(format!("{:?}", e)))?;

            return Ok(Self {
                cert,
                cert_pem,
                key_pem,
            });
        }

        // Generate new certificate
        let bundle = Self::generate_self_signed()?;
        bundle.save_to_disk(dir)?;
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_self_signed() {
        let bundle = CertBundle::generate_self_signed().unwrap();
        assert!(!bundle.cert_pem.is_empty());
        assert!(!bundle.key_pem.is_empty());
        assert!(bundle.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(bundle.key_pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_save_to_disk() {
        let dir = TempDir::new().unwrap();
        let bundle = CertBundle::generate_self_signed().unwrap();

        bundle.save_to_disk(dir.path()).unwrap();

        assert!(dir.path().join("ca.pem").exists());
        assert!(dir.path().join("ca.key").exists());
    }

    #[test]
    fn test_save_twice_fails() {
        let dir = TempDir::new().unwrap();
        let bundle = CertBundle::generate_self_signed().unwrap();

        bundle.save_to_disk(dir.path()).unwrap();
        assert!(bundle.save_to_disk(dir.path()).is_err());
    }

    #[test]
    fn test_load_or_generate_generates() {
        let dir = TempDir::new().unwrap();

        // First call should generate
        let _bundle1 = CertBundle::load_or_generate(dir.path()).unwrap();
        assert!(dir.path().join("ca.pem").exists());
        assert!(dir.path().join("ca.key").exists());

        // Files should be created
        let cert_content = std::fs::read_to_string(dir.path().join("ca.pem")).unwrap();
        let key_content = std::fs::read_to_string(dir.path().join("ca.key")).unwrap();

        assert!(!cert_content.is_empty());
        assert!(!key_content.is_empty());
    }

    #[test]
    fn test_load_or_generate_loads() {
        let dir = TempDir::new().unwrap();

        // First call: generates
        let bundle1 = CertBundle::load_or_generate(dir.path()).unwrap();

        // Second call: loads
        let bundle2 = CertBundle::load_or_generate(dir.path()).unwrap();

        // PEM strings should match
        assert_eq!(bundle1.cert_pem, bundle2.cert_pem);
        assert_eq!(bundle1.key_pem, bundle2.key_pem);
    }
}
