// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/security/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component identity management

use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{error, info};

#[path = "../agent/src/errors.rs"]
mod errors;
use errors::AgentError;

/// Component identity
/// 
/// Unique identity for this Windows Agent instance.
/// Enforced at startup - fail-closed on identity failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIdentity {
    component_id: String,
    created_at: u64,
    key_id: String,
}

impl ComponentIdentity {
    /// Load or create component identity
    pub fn load_or_create() -> Result<Self, AgentError> {
        let identity_path = Self::identity_path()?;
        
        if identity_path.exists() {
            // Load existing identity
            let identity_data = fs::read_to_string(&identity_path)
                .map_err(|e| AgentError::ComponentIdentityFailure(
                    format!("Failed to read identity file: {}", e)
                ))?;
            
            let identity: ComponentIdentity = serde_json::from_str(&identity_data)
                .map_err(|e| AgentError::ComponentIdentityFailure(
                    format!("Failed to parse identity file: {}", e)
                ))?;
            
            info!("Component identity loaded: {}", identity.component_id);
            Ok(identity)
        } else {
            // Create new identity
            let identity = Self::create_new()?;
            
            // Save identity
            let identity_data = serde_json::to_string_pretty(&identity)
                .map_err(|e| AgentError::ComponentIdentityFailure(
                    format!("Failed to serialize identity: {}", e)
                ))?;
            
            fs::create_dir_all(identity_path.parent().unwrap())
                .map_err(|e| AgentError::ComponentIdentityFailure(
                    format!("Failed to create identity directory: {}", e)
                ))?;
            
            fs::write(&identity_path, identity_data)
                .map_err(|e| AgentError::ComponentIdentityFailure(
                    format!("Failed to write identity file: {}", e)
                ))?;
            
            info!("Component identity created: {}", identity.component_id);
            Ok(identity)
        }
    }
    
    /// Create new component identity
    fn create_new() -> Result<Self, AgentError> {
        let component_id = format!("windows-agent-{}", Uuid::new_v4());
        let key_id = format!("key-{}", Uuid::new_v4());
        
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AgentError::ComponentIdentityFailure(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        Ok(Self {
            component_id,
            created_at,
            key_id,
        })
    }
    
    /// Get identity file path
    fn identity_path() -> Result<PathBuf, AgentError> {
        let mut path = PathBuf::from(std::env::var("PROGRAMDATA")
            .unwrap_or_else(|_| "C:\\ProgramData".to_string()));
        path.push("RansomEye");
        path.push("agent");
        path.push("identity.json");
        Ok(path)
    }
    
    /// Get component ID
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
    
    /// Get key ID
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    
    /// Validate identity
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.component_id.is_empty() {
            return Err(AgentError::ComponentIdentityFailure(
                "Component ID is empty".to_string()
            ));
        }
        
        if self.key_id.is_empty() {
            return Err(AgentError::ComponentIdentityFailure(
                "Key ID is empty".to_string()
            ));
        }
        
        Ok(())
    }
}
