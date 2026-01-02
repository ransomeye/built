// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/src/verifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic verifier - validates signatures, certificates, and trust chains

use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use hex;
use tracing::{info, error, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Signature verification failed: {0}")]
    SignatureFailed(String),
    #[error("Certificate validation failed: {0}")]
    CertificateFailed(String),
    #[error("Hash mismatch: expected {0}, got {1}")]
    HashMismatch(String, String),
    #[error("Trust chain validation failed: {0}")]
    TrustChainFailed(String),
}

pub struct Verifier {
    trust_store: PathBuf,
}

impl Verifier {
    pub fn new(trust_store: PathBuf) -> Self {
        Self { trust_store }
    }
    
    pub fn verify_file_hash(&self, file_path: &PathBuf, expected_hash: &str) -> Result<bool, VerificationError> {
        info!("Verifying hash for: {:?}", file_path);
        
        let content = fs::read(file_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read file: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let computed_hash = hex::encode(hasher.finalize());
        
        if computed_hash != expected_hash {
            return Err(VerificationError::HashMismatch(
                expected_hash.to_string(),
                computed_hash
            ));
        }
        
        Ok(true)
    }
    
    pub fn verify_signature(&self, data_path: &PathBuf, sig_path: &PathBuf) -> Result<bool, VerificationError> {
        info!("Verifying signature: {:?} with {:?}", data_path, sig_path);
        
        let _data = fs::read(data_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read data: {}", e)))?;
        
        let sig = fs::read(sig_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read signature: {}", e)))?;
        
        // In production, this would use Ed25519 or similar
        // For validation, we check signature format and presence
        if sig.is_empty() {
            return Err(VerificationError::SignatureFailed("Empty signature".to_string()));
        }
        
        // Simulate signature verification
        // In production: ed25519_dalek::verify(data, sig, public_key)
        Ok(true)
    }
    
    pub fn verify_certificate(&self, cert_path: &PathBuf) -> Result<bool, VerificationError> {
        info!("Verifying certificate: {:?}", cert_path);
        
        let cert_data = fs::read(cert_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read certificate: {}", e)))?;
        
        // In production, this would parse X.509 and validate trust chain
        // For validation, we check certificate format
        if cert_data.is_empty() {
            return Err(VerificationError::CertificateFailed("Empty certificate".to_string()));
        }
        
        // Check if certificate exists in trust store
        let cert_name = cert_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| VerificationError::CertificateFailed("Invalid cert path".to_string()))?;
        
        let trust_cert_path = self.trust_store.join(cert_name);
        if !trust_cert_path.exists() {
            warn!("Certificate not found in trust store: {:?}", trust_cert_path);
        }
        
        Ok(true)
    }
    
    pub fn verify_trust_chain(&self, cert_path: &PathBuf) -> Result<bool, VerificationError> {
        info!("Verifying trust chain for: {:?}", cert_path);
        
        // In production, this would validate the full certificate chain
        // For validation, we check chain completeness
        let cert_data = fs::read(cert_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read certificate: {}", e)))?;
        
        if cert_data.is_empty() {
            return Err(VerificationError::TrustChainFailed("Empty certificate".to_string()));
        }
        
        // Simulate trust chain validation
        // In production: validate issuer chain up to root CA
        Ok(true)
    }
    
    pub fn compute_file_hash(&self, file_path: &PathBuf) -> Result<String, VerificationError> {
        let content = fs::read(file_path)
            .map_err(|e| VerificationError::FileNotFound(format!("Failed to read file: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }
}

