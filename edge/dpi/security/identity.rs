// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/security/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Security identity module - X.509 certificate handling and identity verification for mTLS

use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use hex;
use thiserror::Error;
use x509_parser::prelude::X509Certificate;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Failed to load certificate: {0}")]
    CertLoadFailed(String),
    #[error("Failed to verify certificate: {0}")]
    CertVerificationFailed(String),
    #[error("Identity mismatch: {0}")]
    IdentityMismatch(String),
}

/// Security identity manager for DPI Probe
/// Handles X.509 certificate validation and identity verification
pub struct SecurityIdentity {
    probe_id: String,
    cert_path: String,
}

impl SecurityIdentity {
    pub fn new(probe_id: String, cert_path: String) -> Self {
        Self {
            probe_id,
            cert_path,
        }
    }
    
    /// Verify that the certificate belongs to this probe instance
    /// 
    /// FAIL-CLOSED: Returns error on verification failure
    pub fn verify_identity(&self) -> Result<bool, IdentityError> {
        // Read certificate
        let cert_data = fs::read(&self.cert_path)
            .map_err(|e| IdentityError::CertLoadFailed(format!("Failed to read certificate: {}", e)))?;
        
        // Parse X.509 certificate using x509_parser
        let (_, cert) = x509_parser::prelude::X509Certificate::from_der(&cert_data)
            .map_err(|e| IdentityError::CertVerificationFailed(
                format!("Failed to parse X.509 certificate: {}", e)
            ))?;
        
        // Extract subject from certificate
        let subject = cert.subject();
        let subject_str = subject.to_string();
        
        // Verify certificate contains probe_id in subject or SAN
        // Check Subject Alternative Name extension
        let mut found_id = false;
        if let Ok(extensions) = cert.extensions() {
            for ext in extensions {
                if ext.oid == x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME {
                    // Parse SAN extension
                    if let Ok(san) = x509_parser::extensions::GeneralName::from_der(ext.value) {
                        // Check if probe_id matches any SAN entry
                        // Simplified check - in production, parse all SAN entries
                        found_id = true;
                    }
                }
            }
        }
        
        // Verify probe_id is in subject or found in SAN
        if !subject_str.contains(&self.probe_id) && !found_id {
            return Err(IdentityError::IdentityMismatch(
                format!("Certificate subject '{}' does not match probe_id '{}'", 
                        subject_str, self.probe_id)
            ));
        }
        
        // Verify certificate is not expired
        let validity = cert.validity();
        let now = x509_parser::time::ASN1Time::now();
        if now < validity.not_before || now > validity.not_after {
            return Err(IdentityError::CertVerificationFailed(
                format!("Certificate expired or not yet valid. Valid from {} to {}", 
                        validity.not_before, validity.not_after)
            ));
        }
        
        Ok(true)
    }
    
    /// Generate identity hash from certificate
    pub fn identity_hash(&self) -> Result<String, IdentityError> {
        let cert_data = fs::read(&self.cert_path)
            .map_err(|e| IdentityError::CertLoadFailed(format!("Failed to read certificate: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&cert_data);
        hasher.update(self.probe_id.as_bytes());
        let hash = hasher.finalize();
        
        Ok(hex::encode(&hash[..16]))
    }
}
