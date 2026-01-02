// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/security/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model verification - verifies model integrity

use tracing::debug;
use crate::errors::AdvisoryError;

pub struct ModelVerifier;

impl ModelVerifier {
    pub fn new() -> Self {
        Self
    }
    
    /// Verify model integrity
    pub fn verify(&self, model_bytes: &[u8], expected_hash: &str) -> Result<bool, AdvisoryError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let mut hasher = Sha256::new();
        hasher.update(model_bytes);
        let computed_hash = hex::encode(hasher.finalize());
        
        if computed_hash != expected_hash {
            return Err(AdvisoryError::ModelIntegrityFailed(
                format!("Model hash mismatch: expected {}, got {}", expected_hash, computed_hash)
            ));
        }
        
        debug!("Model integrity verified");
        Ok(true)
    }
}

