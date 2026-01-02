// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/registry/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model verification - verifies model signatures and integrity

use std::path::Path;
use tracing::{error, warn, debug};
use crate::errors::AdvisoryError;
use crate::security::signature::ModelSignatureVerifier as SecurityModelSignatureVerifier;

pub struct ModelVerifier {
    signature_verifier: SecurityModelSignatureVerifier,
}

impl ModelVerifier {
    pub fn new() -> Result<Self, AdvisoryError> {
        let signature_verifier = SecurityModelSignatureVerifier::new()
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Failed to initialize signature verifier: {}", e)
            ))?;
        
        Ok(Self {
            signature_verifier,
        })
    }
    
    /// Verify model signature
    pub fn verify_model_signature(&self, model_path: &Path, signature: &str) -> Result<bool, AdvisoryError> {
        debug!("Verifying model signature for {}", model_path.display());
        
        // Read model file
        let model_bytes = std::fs::read(model_path)
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Failed to read model file: {}", e)
            ))?;
        
        // Verify signature
        self.signature_verifier.verify(&model_bytes, signature)
            .map_err(|e| AdvisoryError::InvalidModelSignature(
                format!("Model signature verification failed: {}", e)
            ))
    }
    
    /// Verify model integrity
    pub fn verify_model_integrity(&self, model_path: &Path, expected_hash: &str) -> Result<bool, AdvisoryError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let model_bytes = std::fs::read(model_path)
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Failed to read model file: {}", e)
            ))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&model_bytes);
        let computed_hash = hex::encode(hasher.finalize());
        
        if computed_hash != expected_hash {
            return Err(AdvisoryError::ModelIntegrityFailed(
                format!("Model hash mismatch: expected {}, got {}", expected_hash, computed_hash)
            ));
        }
        
        Ok(true)
    }
}

