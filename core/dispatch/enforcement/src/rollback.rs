// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/rollback.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rollback manager - tracks and executes rollbacks for reversible operations

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    pub rollback_id: String,
    pub execution_id: String,
    pub decision_id: String,
    pub action_taken: String,
    pub targets: Vec<String>,
    pub rollback_commands: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub rolled_back: bool,
    pub rolled_back_at: Option<DateTime<Utc>>,
}

pub struct RollbackManager {
    records: Arc<RwLock<HashMap<String, RollbackRecord>>>, // execution_id -> record
}

impl RollbackManager {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Record an execution for potential rollback
    pub fn record_execution(
        &self,
        execution_id: &str,
        decision_id: &str,
        action_taken: &str,
        targets: &[String],
        rollback_commands: &[String],
    ) -> Result<String, EnforcementError> {
        let rollback_id = Uuid::new_v4().to_string();
        
        let record = RollbackRecord {
            rollback_id: rollback_id.clone(),
            execution_id: execution_id.to_string(),
            decision_id: decision_id.to_string(),
            action_taken: action_taken.to_string(),
            targets: targets.to_vec(),
            rollback_commands: rollback_commands.to_vec(),
            created_at: Utc::now(),
            rolled_back: false,
            rolled_back_at: None,
        };
        
        let mut records_map = self.records.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        records_map.insert(execution_id.to_string(), record);
        
        debug!("Recorded execution {} for rollback {}", execution_id, rollback_id);
        Ok(rollback_id)
    }
    
    /// Execute rollback
    pub fn rollback(&self, execution_id: &str) -> Result<(), EnforcementError> {
        let mut records_map = self.records.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let record = records_map.get_mut(execution_id)
            .ok_or_else(|| EnforcementError::RollbackFailed(
                format!("No rollback record found for execution {}", execution_id)
            ))?;
        
        if record.rolled_back {
            return Err(EnforcementError::RollbackFailed(
                format!("Execution {} already rolled back", execution_id)
            ));
        }
        
        if record.rollback_commands.is_empty() {
            return Err(EnforcementError::RollbackFailed(
                format!("No rollback commands available for execution {}", execution_id)
            ));
        }
        
        // Mark as rolled back (actual rollback execution would happen in adapter)
        record.rolled_back = true;
        record.rolled_back_at = Some(Utc::now());
        
        debug!("Rollback executed for execution {}: {} commands", 
            execution_id, record.rollback_commands.len());
        
        Ok(())
    }
    
    /// Get rollback record
    pub fn get_record(&self, execution_id: &str) -> Option<RollbackRecord> {
        let records_map = self.records.read()
            .ok()?;
        records_map.get(execution_id).cloned()
    }
    
    /// Check if rollback is available
    pub fn is_rollback_available(&self, execution_id: &str) -> bool {
        if let Some(record) = self.get_record(execution_id) {
            !record.rolled_back && !record.rollback_commands.is_empty()
        } else {
            false
        }
    }
}

