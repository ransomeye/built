// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/security/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic signature verification for policy decisions - RSA-4096-PSS-SHA256

use ring::signature::{UnparsedPublicKey, RSA_PSS_SHA256};
use x509_parser::pem::Pem;
use x509_parser::prelude::*;
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;

pub struct SignatureVerifier {
    public_key_bytes: Vec<u8>,
}

impl SignatureVerifier {
    pub fn new(public_key_pem: &[u8]) -> Result<Self, EnforcementError> {
        // Parse PEM-encoded public key
        let public_key_bytes = Self::parse_public_key(public_key_pem)?;
        
        Ok(Self {
            public_key_bytes,
        })
    }
    
    fn parse_public_key(pem_bytes: &[u8]) -> Result<Vec<u8>, EnforcementError> {
        // Try to parse as X.509 certificate first
        if let Ok((_, pem)) = Pem::parse(pem_bytes) {
            if let Ok((_, cert)) = x509_parser::parse_x509_certificate(&pem.contents) {
                // Extract public key from certificate
                let public_key_info = cert.public_key();
                return Ok(public_key_info.raw.to_vec());
            }
        }
        
        // Try to parse as standalone PEM public key
        let pem_str = std::str::from_utf8(pem_bytes)
            .map_err(|e| EnforcementError::ConfigurationError(format!("Invalid PEM encoding: {}", e)))?;
        
        // Extract base64 content
        let pem_lines: Vec<&str> = pem_str
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        
        let base64_content = pem_lines.join("");
        let key_bytes = base64::decode(&base64_content)
            .map_err(|e| EnforcementError::ConfigurationError(format!("Base64 decode failed: {}", e)))?;
        
        // Try to parse as SubjectPublicKeyInfo
        if let Ok((_, spki)) = SubjectPublicKeyInfo::from_der(&key_bytes) {
            return Ok(spki.raw.to_vec());
        }
        
        // If parsing fails, assume it's already in the right format
        Ok(key_bytes)
    }
    
    /// Verify signature using RSA-PSS-SHA256
    pub fn verify(&self, data: &[u8], signature: &str) -> Result<bool, EnforcementError> {
        let signature_bytes = base64::decode(signature)
            .map_err(|e| EnforcementError::InvalidSignature(format!("Base64 decode failed: {}", e)))?;
        
        // Verify signature using UnparsedPublicKey
        let unparsed_key = UnparsedPublicKey::new(&RSA_PSS_SHA256, &self.public_key_bytes);
        
        match unparsed_key.verify(data, &signature_bytes) {
            Ok(_) => {
                debug!("Signature verification successful");
                Ok(true)
            }
            Err(e) => {
                warn!("Signature verification failed: {:?}", e);
                Err(EnforcementError::InvalidSignature(format!("Verification failed: {:?}", e)))
            }
        }
    }
    
    /// Verify decision signature
    pub fn verify_decision_signature(&self, decision_json: &[u8], signature: &str) -> Result<bool, EnforcementError> {
        // Compute hash of decision
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(decision_json);
        let data_hash = hasher.finalize();
        
        // Verify signature against hash
        self.verify(&data_hash, signature)
    }
}

