// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/security/attestation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component attestation for trust verification

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use hex;
use tracing::info;
use base64::{Engine as _, engine::general_purpose};

use crate::errors::AgentError;
use super::identity::ComponentIdentity;
use ed25519_dalek::VerifyingKey;

/// Component attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub component_id: String,
    pub component_type: String,
    pub hostname: String,
    pub verifying_key: String, // Base64 encoded public key
    pub attestation_hash: String,
    pub timestamp: u64,
}

pub struct AttestationManager;

impl AttestationManager {
    /// Create attestation for component
    pub fn create(identity: &ComponentIdentity, verifying_key: &VerifyingKey) -> Result<Attestation, AgentError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let verifying_key_b64 = general_purpose::STANDARD.encode(verifying_key.to_bytes());
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Time error: {}", e)
            ))?
            .as_secs();
        
        let mut attestation = Attestation {
            component_id: identity.component_id.clone(),
            component_type: identity.component_type.clone(),
            hostname: identity.hostname.clone(),
            verifying_key: verifying_key_b64,
            attestation_hash: String::new(),
            timestamp,
        };
        
        attestation.attestation_hash = Self::compute_hash(&attestation)?;
        
        info!("Component attestation created: {}", attestation.component_id);
        Ok(attestation)
    }
    
    /// Verify attestation
    pub fn verify(attestation: &Attestation) -> Result<bool, AgentError> {
        let computed_hash = Self::compute_hash(attestation)?;
        
        if computed_hash != attestation.attestation_hash {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Compute attestation hash
    fn compute_hash(attestation: &Attestation) -> Result<String, AgentError> {
        let mut hasher = Sha256::new();
        hasher.update(attestation.component_id.as_bytes());
        hasher.update(attestation.component_type.as_bytes());
        hasher.update(attestation.hostname.as_bytes());
        hasher.update(attestation.verifying_key.as_bytes());
        hasher.update(&attestation.timestamp.to_be_bytes());
        
        Ok(hex::encode(hasher.finalize()))
    }
}
