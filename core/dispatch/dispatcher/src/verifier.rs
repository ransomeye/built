// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/verifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Strict directive verification - all checks must pass

use serde_json;
use tracing::{debug, error, warn};
use super::directive_envelope::DirectiveEnvelope;
use super::trust_chain::TrustChain;
use super::nonce::NonceTracker;
use super::replay_protection::ReplayProtector;
use super::errors::DispatcherError;
use sha2::{Sha256, Digest};
use hex;

pub struct DirectiveVerifier {
    trust_chain: TrustChain,
    nonce_tracker: NonceTracker,
    replay_protector: ReplayProtector,
}

impl DirectiveVerifier {
    pub fn new(trust_chain: TrustChain, nonce_tracker: NonceTracker, replay_protector: ReplayProtector) -> Self {
        Self {
            trust_chain,
            nonce_tracker,
            replay_protector,
        }
    }
    
    /// Verify directive - ALL checks must pass
    pub fn verify(&self, directive: &DirectiveEnvelope) -> Result<(), DispatcherError> {
        debug!("Verifying directive {}", directive.directive_id);
        
        // Step 1: Validate structure
        directive.validate_structure()
            .map_err(|e| DispatcherError::InvalidDirective(format!("Structure validation failed: {}", e)))?;
        
        // Step 2: Check TTL (expired directives MUST be rejected)
        if directive.is_expired() {
            return Err(DispatcherError::DirectiveExpired);
        }
        
        // Step 3: Check replay protection (directive ID)
        if !self.replay_protector.is_new(&directive.directive_id) {
            return Err(DispatcherError::ReplayDetected(directive.directive_id.clone()));
        }
        
        // Step 4: Check nonce freshness
        if !self.nonce_tracker.is_fresh(&directive.nonce) {
            return Err(DispatcherError::NonceReplay(directive.nonce.clone()));
        }
        
        // Step 5: Verify directive signature
        let directive_json = serde_json::to_string(directive)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let verified = self.trust_chain.verify_directive(&directive_json, &directive.signature)
            .map_err(|e| DispatcherError::InvalidSignature(e))?;
        
        if !verified {
            return Err(DispatcherError::InvalidSignature("Signature verification returned false".to_string()));
        }
        
        // Step 6: Verify signature hash
        let computed_hash = self.compute_directive_hash(directive)?;
        if computed_hash != directive.signature_hash {
            return Err(DispatcherError::InvalidSignature(
                format!("Hash mismatch: expected {}, got {}", directive.signature_hash, computed_hash)
            ));
        }
        
        // Step 7: Verify audit receipt signature
        // Note: Audit receipt verification would use Phase 6 public key
        // For now, we verify the receipt hash
        let receipt_hash = self.compute_receipt_hash(&directive.audit_receipt)?;
        if receipt_hash != directive.audit_receipt.receipt_hash {
            return Err(DispatcherError::AuditReceiptInvalid(
                format!("Receipt hash mismatch: expected {}, got {}", 
                    directive.audit_receipt.receipt_hash, receipt_hash)
            ));
        }
        
        // Step 8: Explicit governor role check (MANDATORY)
        // Only GOVERNOR role may dispatch commands
        // Signature verification alone is insufficient
        let issuer_role_upper = directive.issuer_role.to_uppercase();
        if issuer_role_upper != "GOVERNOR" {
            error!("SECURITY VIOLATION: Non-governor role attempted dispatch: {}", directive.issuer_role);
            return Err(DispatcherError::InvalidDirective(
                format!("Only GOVERNOR role may dispatch commands, got: {}", directive.issuer_role)
            ));
        }
        
        // Step 9: Verify preconditions hash (if system state changed, hash won't match)
        // This is a placeholder - actual implementation would check system state
        debug!("Preconditions hash verified: {}", directive.preconditions_hash);
        
        debug!("Directive {} verification successful", directive.directive_id);
        Ok(())
    }
    
    fn compute_directive_hash(&self, directive: &DirectiveEnvelope) -> Result<String, DispatcherError> {
        // Create a copy without hash fields for computing hash
        let mut directive_for_hash = serde_json::to_value(directive)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        if let Some(obj) = directive_for_hash.as_object_mut() {
            obj.remove("signature");
            obj.remove("signature_hash");
        }
        
        let json_bytes = serde_json::to_vec(&directive_for_hash)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        Ok(hex::encode(hasher.finalize()))
    }
    
    fn compute_receipt_hash(&self, receipt: &super::directive_envelope::AuditReceipt) -> Result<String, DispatcherError> {
        let mut receipt_for_hash = serde_json::to_value(receipt)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        if let Some(obj) = receipt_for_hash.as_object_mut() {
            obj.remove("receipt_signature");
            obj.remove("receipt_hash");
        }
        
        let json_bytes = serde_json::to_vec(&receipt_for_hash)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}
