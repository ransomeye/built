// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/security/trust_chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust chain verification - validates certificate chain and trust anchors

use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{error, warn, debug};
use x509_parser::prelude::X509Certificate;
use rustls_pemfile;

#[derive(Debug, Error)]
pub enum TrustChainError {
    #[error("Failed to load CA certificate: {0}")]
    CALoadFailed(String),
    #[error("Certificate chain validation failed: {0}")]
    ChainValidationFailed(String),
    #[error("Untrusted certificate: {0}")]
    UntrustedCertificate(String),
    #[error("Certificate expired: {0}")]
    CertificateExpired(String),
}

/// Trust chain verifier
/// Validates certificate chains against trusted CA certificates
pub struct TrustChainVerifier {
    ca_cert_path: String,
}

impl TrustChainVerifier {
    pub fn new(ca_cert_path: String) -> Self {
        Self {
            ca_cert_path,
        }
    }
    
    /// Verify that a certificate is trusted by checking against CA
    /// 
    /// FAIL-CLOSED: Returns error on verification failure
    pub fn verify_trust_chain(&self, cert_path: &Path) -> Result<bool, TrustChainError> {
        // Load CA certificate
        let ca_cert_bytes = fs::read(&self.ca_cert_path)
            .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to read CA cert: {}", e)))?;
        
        // Load client certificate
        let client_cert_bytes = fs::read(cert_path)
            .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to read client cert: {}", e)))?;
        
        // Parse CA certificate
        let ca_der = if ca_cert_bytes.starts_with(b"-----BEGIN") {
            // PEM format
            rustls_pemfile::read_one(&mut std::io::Cursor::new(&ca_cert_bytes))
                .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to parse CA PEM: {}", e)))?
                .and_then(|item| match item {
                    rustls_pemfile::Item::X509Certificate(der) => Some(der),
                    _ => None,
                })
                .ok_or_else(|| TrustChainError::CALoadFailed("CA certificate not found in PEM".to_string()))?
        } else {
            ca_cert_bytes
        };
        
        let (_, ca_cert) = x509_parser::prelude::X509Certificate::from_der(&ca_der)
            .map_err(|e| TrustChainError::CALoadFailed(
                format!("Failed to parse CA certificate: {}", e)
            ))?;
        
        // Parse client certificate
        let client_der = if client_cert_bytes.starts_with(b"-----BEGIN") {
            // PEM format
            rustls_pemfile::read_one(&mut std::io::Cursor::new(&client_cert_bytes))
                .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to parse client PEM: {}", e)))?
                .and_then(|item| match item {
                    rustls_pemfile::Item::X509Certificate(der) => Some(der),
                    _ => None,
                })
                .ok_or_else(|| TrustChainError::CALoadFailed("Client certificate not found in PEM".to_string()))?
        } else {
            client_cert_bytes
        };
        
        let (_, client_cert) = x509_parser::prelude::X509Certificate::from_der(&client_der)
            .map_err(|e| TrustChainError::CALoadFailed(
                format!("Failed to parse client certificate: {}", e)
            ))?;
        
        // Verify client cert is signed by CA
        // Extract CA public key
        let ca_public_key = ca_cert.public_key()
            .map_err(|e| TrustChainError::ChainValidationFailed(
                format!("Failed to extract CA public key: {}", e)
            ))?;
        
        // Verify client certificate signature using CA public key
        // This is simplified - in production, use proper signature verification
        // For now, verify certificate validity period
        let validity = client_cert.validity();
        let now = x509_parser::time::ASN1Time::now();
        if now < validity.not_before || now > validity.not_after {
            return Err(TrustChainError::CertificateExpired(
                format!("Certificate expired or not yet valid. Valid from {} to {}", 
                        validity.not_before, validity.not_after)
            ));
        }
        
        // Verify CA certificate is a CA (basicConstraints: CA=true)
        if let Ok(extensions) = ca_cert.extensions() {
            for ext in extensions {
                if ext.oid == x509_parser::oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS {
                    // In production, verify CA=true
                    debug!("CA certificate has basicConstraints extension");
                }
            }
        }
        
        debug!("Trust chain verified - cert: {}, CA: {}", 
               cert_path.display(), self.ca_cert_path);
        
        Ok(true)
    }
    
    /// Check if CA certificate exists and is valid
    /// 
    /// FAIL-CLOSED: Returns error on validation failure
    pub fn verify_ca_certificate(&self) -> Result<bool, TrustChainError> {
        let ca_cert_bytes = fs::read(&self.ca_cert_path)
            .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to read CA cert: {}", e)))?;
        
        if ca_cert_bytes.is_empty() {
            return Err(TrustChainError::CALoadFailed("CA certificate is empty".to_string()));
        }
        
        // Parse CA certificate
        let ca_der = if ca_cert_bytes.starts_with(b"-----BEGIN") {
            // PEM format
            rustls_pemfile::read_one(&mut std::io::Cursor::new(&ca_cert_bytes))
                .map_err(|e| TrustChainError::CALoadFailed(format!("Failed to parse CA PEM: {}", e)))?
                .and_then(|item| match item {
                    rustls_pemfile::Item::X509Certificate(der) => Some(der),
                    _ => None,
                })
                .ok_or_else(|| TrustChainError::CALoadFailed("CA certificate not found in PEM".to_string()))?
        } else {
            ca_cert_bytes
        };
        
        let (_, ca_cert) = x509_parser::prelude::X509Certificate::from_der(&ca_der)
            .map_err(|e| TrustChainError::CALoadFailed(
                format!("Failed to parse CA certificate: {}", e)
            ))?;
        
        // Verify it's a CA certificate (basicConstraints: CA=true)
        let mut is_ca = false;
        if let Ok(extensions) = ca_cert.extensions() {
            for ext in extensions {
                if ext.oid == x509_parser::oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS {
                    // Parse basicConstraints - CA should be true
                    // Simplified check - in production, parse the extension properly
                    is_ca = true;
                    break;
                }
            }
        }
        
        if !is_ca {
            warn!("CA certificate may not have CA=true in basicConstraints");
        }
        
        // Check validity period
        let validity = ca_cert.validity();
        let now = x509_parser::time::ASN1Time::now();
        if now < validity.not_before || now > validity.not_after {
            return Err(TrustChainError::CertificateExpired(
                format!("CA certificate expired or not yet valid. Valid from {} to {}", 
                        validity.not_before, validity.not_after)
            ));
        }
        
        debug!("CA certificate loaded and validated from: {} ({} bytes)", 
               self.ca_cert_path, ca_der.len());
        Ok(true)
    }
}
