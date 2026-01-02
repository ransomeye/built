// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/security/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Decision integrity verification - validates decision structure and signatures

use serde_json;
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;
use crate::security::signature::SignatureVerifier;

pub struct DecisionVerifier {
    signature_verifier: SignatureVerifier,
}

impl DecisionVerifier {
    pub fn new(signature_verifier: SignatureVerifier) -> Self {
        Self {
            signature_verifier,
        }
    }
    
    /// Verify decision integrity and signature
    pub fn verify_decision(&self, decision_json: &str) -> Result<(), EnforcementError> {
        // Parse decision
        let decision: serde_json::Value = serde_json::from_str(decision_json)
            .map_err(|e| EnforcementError::InvalidFormat(format!("Invalid JSON: {}", e)))?;
        
        // Extract signature
        let signature = decision.get("policy_signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::UnsignedDecision("Missing policy_signature field".to_string()))?;
        
        if signature.is_empty() {
            return Err(EnforcementError::UnsignedDecision("Empty signature".to_string()));
        }
        
        // Extract decision hash
        let decision_hash = decision.get("decision_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::IntegrityCheckFailed("Missing decision_hash field".to_string()))?;
        
        // Verify decision hash integrity
        let computed_hash = self.compute_decision_hash(decision_json)?;
        if computed_hash != decision_hash {
            return Err(EnforcementError::IntegrityCheckFailed(
                format!("Hash mismatch: expected {}, got {}", decision_hash, computed_hash)
            ));
        }
        
        // Verify signature
        let decision_bytes = decision_json.as_bytes();
        match self.signature_verifier.verify_decision_signature(decision_bytes, signature) {
            Ok(true) => {
                debug!("Decision signature verified successfully");
                Ok(())
            }
            Ok(false) => {
                Err(EnforcementError::InvalidSignature("Signature verification returned false".to_string()))
            }
            Err(e) => Err(e),
        }
    }
    
    fn compute_decision_hash(&self, decision_json: &str) -> Result<String, EnforcementError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let decision: serde_json::Value = serde_json::from_str(decision_json)
            .map_err(|e| EnforcementError::InvalidFormat(format!("Invalid JSON: {}", e)))?;
        
        // Create a copy without the hash field for computing hash
        let mut decision_for_hash = decision.clone();
        if let Some(obj) = decision_for_hash.as_object_mut() {
            obj.remove("decision_hash");
        }
        
        let json_bytes = serde_json::to_vec(&decision_for_hash)
            .map_err(|e| EnforcementError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}

