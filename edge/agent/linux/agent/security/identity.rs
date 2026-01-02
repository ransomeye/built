// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/security/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component identity for Linux Agent

use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use hex;
use tracing::info;

use crate::errors::AgentError;

/// Component identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIdentity {
    pub component_id: String,
    pub component_type: String,
    pub hostname: String,
    pub instance_id: String,
    pub identity_hash: String,
}

pub struct IdentityManager {
    identity: ComponentIdentity,
}

impl IdentityManager {
    /// Load or create component identity
    pub fn load_or_create(identity_path: Option<&Path>) -> Result<Self, AgentError> {
        if let Some(path) = identity_path {
            if path.exists() {
                return Self::load_from_file(path);
            }
        }
        
        Self::create_new()
    }
    
    /// Load identity from file
    fn load_from_file(path: &Path) -> Result<Self, AgentError> {
        let content = fs::read_to_string(path)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to read identity file: {}", e)
            ))?;
        
        let identity: ComponentIdentity = serde_json::from_str(&content)
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to parse identity: {}", e)
            ))?;
        
        let computed_hash = Self::compute_identity_hash(&identity)?;
        if computed_hash != identity.identity_hash {
            return Err(AgentError::IdentityVerificationFailed(
                "Identity hash mismatch".to_string()
            ));
        }
        
        info!("Component identity loaded: {}", identity.component_id);
        Ok(Self { identity })
    }
    
    /// Create new identity
    fn create_new() -> Result<Self, AgentError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let hostname = hostname::get()
            .map_err(|e| AgentError::IdentityVerificationFailed(
                format!("Failed to get hostname: {}", e)
            ))?
            .to_string_lossy()
            .to_string();
        
        let instance_id = format!("linux-agent-{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AgentError::IdentityVerificationFailed(
                    format!("Time error: {}", e)
                ))?
                .as_secs());
        
        let component_id = format!("{}-{}", hostname, instance_id);
        
        let identity = ComponentIdentity {
            component_id: component_id.clone(),
            component_type: "linux_agent".to_string(),
            hostname,
            instance_id,
            identity_hash: String::new(),
        };
        
        let identity_hash = Self::compute_identity_hash(&identity)?;
        let mut identity = identity;
        identity.identity_hash = identity_hash;
        
        info!("Component identity created: {}", identity.component_id);
        Ok(Self { identity })
    }
    
    /// Compute identity hash
    fn compute_identity_hash(identity: &ComponentIdentity) -> Result<String, AgentError> {
        let mut hasher = Sha256::new();
        hasher.update(identity.component_id.as_bytes());
        hasher.update(identity.component_type.as_bytes());
        hasher.update(identity.hostname.as_bytes());
        hasher.update(identity.instance_id.as_bytes());
        
        Ok(hex::encode(hasher.finalize()))
    }
    
    /// Get component identity
    pub fn identity(&self) -> &ComponentIdentity {
        &self.identity
    }
    
    /// Get component ID
    pub fn component_id(&self) -> &str {
        &self.identity.component_id
    }
}
