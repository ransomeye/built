// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/registry/rollback.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model rollback - rollback to previous model versions

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{error, warn, debug};
use crate::errors::AdvisoryError;

#[derive(Debug, Clone)]
pub struct ModelVersion {
    pub name: String,
    pub version: String,
    pub path: String,
    pub signature: String,
}

pub struct ModelRollback {
    versions: Arc<RwLock<HashMap<String, Vec<ModelVersion>>>>, // model_name -> versions
}

impl ModelRollback {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Record model version for rollback
    pub fn record_version(&self, model_name: &str, version: &str, path: &str, signature: &str) -> Result<(), AdvisoryError> {
        let mut versions_map = self.versions.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let versions = versions_map.entry(model_name.to_string())
            .or_insert_with(Vec::new);
        
        versions.push(ModelVersion {
            name: model_name.to_string(),
            version: version.to_string(),
            path: path.to_string(),
            signature: signature.to_string(),
        });
        
        debug!("Recorded model version {} for {}", version, model_name);
        Ok(())
    }
    
    /// Get previous version for rollback
    pub fn get_previous_version(&self, model_name: &str) -> Result<Option<ModelVersion>, AdvisoryError> {
        let versions_map = self.versions.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        if let Some(versions) = versions_map.get(model_name) {
            if versions.len() > 1 {
                return Ok(Some(versions[versions.len() - 2].clone()));
            }
        }
        
        Ok(None)
    }
}

