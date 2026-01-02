// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/security/attestation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component attestation for identity verification

use serde::{Serialize, Deserialize};
use tracing::{error, info};
use hostname;

#[path = "../agent/src/errors.rs"]
mod errors;
use errors::AgentError;

pub use super::identity::ComponentIdentity;
pub use super::signing::EventSigner;

/// Component attestation
/// 
/// Provides attestation evidence for component identity verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAttestation {
    component_id: String,
    timestamp: u64,
    attestation_data: AttestationData,
    signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    hostname: String,
    os_version: String,
    agent_version: String,
    key_id: String,
}

impl ComponentAttestation {
    /// Create component attestation
    pub fn create(identity: &ComponentIdentity, signer: &EventSigner) -> Result<Self, AgentError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let hostname = hostname::get()
            .map_err(|_| AgentError::IdentityVerificationFailed(
                "Failed to get hostname".to_string()
            ))?
            .to_string_lossy()
            .to_string();
        
        let attestation_data = AttestationData {
            hostname,
            os_version: Self::get_os_version()?,
            agent_version: "0.1.0".to_string(),
            key_id: identity.key_id().to_string(),
        };
        
        // Sign attestation data
        let attestation_json = serde_json::to_string(&attestation_data)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to serialize attestation data: {}", e)
            ))?;
        
        let signature = signer.sign(attestation_json.as_bytes())?;
        
        Ok(Self {
            component_id: identity.component_id().to_string(),
            timestamp,
            attestation_data,
            signature,
        })
    }
    
    /// Get OS version
    fn get_os_version() -> Result<String, AgentError> {
        #[cfg(windows)]
        {
            // In real implementation, would query Windows version
            Ok("Windows 10/11".to_string())
        }
        
        #[cfg(not(windows))]
        {
            Err(AgentError::IdentityVerificationFailed(
                "OS version query only available on Windows".to_string()
            ))
        }
    }
    
    /// Verify attestation
    pub fn verify(&self, signer: &EventSigner) -> Result<bool, AgentError> {
        let attestation_json = serde_json::to_string(&self.attestation_data)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to serialize attestation data: {}", e)
            ))?;
        
        // Note: In real implementation, would need sequence number from attestation
        // For now, use 0 as placeholder
        signer.verify(attestation_json.as_bytes(), &self.signature, 0)
    }
    
    /// Get component ID
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
}

