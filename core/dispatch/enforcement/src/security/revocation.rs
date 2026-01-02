// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/security/revocation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Decision revocation checker - validates decisions haven't been revoked

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use tracing::{warn, debug};
use crate::errors::EnforcementError;

#[derive(Debug, Clone)]
pub struct RevokedDecision {
    pub decision_id: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

pub struct RevocationChecker {
    revoked_decisions: Arc<RwLock<HashSet<String>>>,
    revocation_list_path: Option<String>,
}

impl RevocationChecker {
    pub fn new(revocation_list_path: Option<String>) -> Self {
        let checker = Self {
            revoked_decisions: Arc::new(RwLock::new(HashSet::new())),
            revocation_list_path: revocation_list_path.clone(),
        };
        
        // Load revocation list if path provided
        if let Some(path) = revocation_list_path {
            if let Err(e) = checker.load_revocation_list(&path) {
                warn!("Failed to load revocation list from {}: {}", path, e);
            }
        }
        
        checker
    }
    
    pub fn load_revocation_list(&self, path: &str) -> Result<(), EnforcementError> {
        use std::fs;
        
        if !std::path::Path::new(path).exists() {
            debug!("Revocation list file does not exist: {}", path);
            return Ok(()); // Not an error if file doesn't exist
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| EnforcementError::ConfigurationError(format!("Failed to read revocation list: {}", e)))?;
        
        let revoked: Vec<String> = serde_json::from_str(&content)
            .map_err(|e| EnforcementError::ConfigurationError(format!("Invalid revocation list JSON: {}", e)))?;
        
        let mut revoked_set = self.revoked_decisions.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        for decision_id in revoked {
            revoked_set.insert(decision_id);
        }
        
        debug!("Loaded {} revoked decisions", revoked_set.len());
        Ok(())
    }
    
    pub fn is_revoked(&self, decision_id: &str) -> Result<bool, EnforcementError> {
        let revoked_set = self.revoked_decisions.read()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        Ok(revoked_set.contains(decision_id))
    }
    
    pub fn revoke(&self, decision_id: &str, reason: &str) -> Result<(), EnforcementError> {
        let mut revoked_set = self.revoked_decisions.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        revoked_set.insert(decision_id.to_string());
        debug!("Revoked decision {}: {}", decision_id, reason);
        Ok(())
    }
    
    pub fn check_decision(&self, decision_id: &str) -> Result<(), EnforcementError> {
        if self.is_revoked(decision_id)? {
            return Err(EnforcementError::DecisionRevoked(
                format!("Decision {} has been revoked", decision_id)
            ));
        }
        Ok(())
    }
}

