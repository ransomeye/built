// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/acknowledgment.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Acknowledgment handling and verification

use tracing::{debug, error, info};
use super::acknowledgment_envelope::{AcknowledgmentEnvelope, ExecutionResult};
use super::trust_chain::TrustChain;
use super::errors::DispatcherError;
use serde_json;

pub struct AcknowledgmentHandler {
    trust_chain: TrustChain,
}

impl AcknowledgmentHandler {
    pub fn new(trust_chain: TrustChain) -> Self {
        Self {
            trust_chain,
        }
    }
    
    /// Verify acknowledgment signature and structure
    pub fn verify(&self, ack: &AcknowledgmentEnvelope) -> Result<(), DispatcherError> {
        debug!("Verifying acknowledgment for directive {}", ack.directive_id);
        
        // Validate structure
        ack.validate_structure()
            .map_err(|e| DispatcherError::InvalidAcknowledgment(format!("Structure validation failed: {}", e)))?;
        
        // Verify signature
        let ack_json = serde_json::to_string(ack)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let verified = self.trust_chain.verify_acknowledgment(&ack.agent_id, &ack_json, &ack.signature)
            .map_err(|e| DispatcherError::InvalidAcknowledgment(format!("Signature verification failed: {}", e)))?;
        
        if !verified {
            return Err(DispatcherError::InvalidAcknowledgment("Signature verification returned false".to_string()));
        }
        
        // Verify signature hash
        let computed_hash = self.compute_ack_hash(ack)?;
        if computed_hash != ack.signature_hash {
            return Err(DispatcherError::InvalidAcknowledgment(
                format!("Hash mismatch: expected {}, got {}", ack.signature_hash, computed_hash)
            ));
        }
        
        info!("Acknowledgment verified for directive {}", ack.directive_id);
        Ok(())
    }
    
    /// Check if acknowledgment indicates failure
    pub fn is_failure(&self, ack: &AcknowledgmentEnvelope) -> bool {
        matches!(ack.execution_result, ExecutionResult::Failed | ExecutionResult::Partial)
    }
    
    fn compute_ack_hash(&self, ack: &AcknowledgmentEnvelope) -> Result<String, DispatcherError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let mut ack_for_hash = serde_json::to_value(ack)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        if let Some(obj) = ack_for_hash.as_object_mut() {
            obj.remove("signature");
            obj.remove("signature_hash");
        }
        
        let json_bytes = serde_json::to_vec(&ack_for_hash)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}
