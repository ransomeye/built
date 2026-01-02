// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/security/revocation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model revocation checker - validates models haven't been revoked

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tracing::{warn, debug};
use crate::errors::AdvisoryError;

pub struct ModelRevocationChecker {
    revoked_models: Arc<RwLock<HashSet<String>>>, // model_hash -> revoked
    revocation_list_path: Option<String>,
}

impl ModelRevocationChecker {
    pub fn new(revocation_list_path: Option<String>) -> Self {
        let checker = Self {
            revoked_models: Arc::new(RwLock::new(HashSet::new())),
            revocation_list_path: revocation_list_path.clone(),
        };
        
        if let Some(path) = revocation_list_path {
            if let Err(e) = checker.load_revocation_list(&path) {
                warn!("Failed to load revocation list from {}: {}", path, e);
            }
        }
        
        checker
    }
    
    pub fn load_revocation_list(&self, path: &str) -> Result<(), AdvisoryError> {
        use std::fs;
        
        if !std::path::Path::new(path).exists() {
            debug!("Revocation list file does not exist: {}", path);
            return Ok(());
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| AdvisoryError::ConfigurationError(format!("Failed to read revocation list: {}", e)))?;
        
        let revoked: Vec<String> = serde_json::from_str(&content)
            .map_err(|e| AdvisoryError::ConfigurationError(format!("Invalid revocation list JSON: {}", e)))?;
        
        let mut revoked_set = self.revoked_models.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        for model_hash in revoked {
            revoked_set.insert(model_hash);
        }
        
        debug!("Loaded {} revoked models", revoked_set.len());
        Ok(())
    }
    
    pub fn is_revoked(&self, model_hash: &str) -> Result<bool, AdvisoryError> {
        let revoked_set = self.revoked_models.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        Ok(revoked_set.contains(model_hash))
    }
    
    pub fn check_model(&self, model_hash: &str) -> Result<(), AdvisoryError> {
        if self.is_revoked(model_hash)? {
            return Err(AdvisoryError::ModelRevoked(
                format!("Model {} has been revoked", model_hash)
            ));
        }
        Ok(())
    }
}

