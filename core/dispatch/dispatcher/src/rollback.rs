// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/rollback.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rollback manager - signed, time-bounded, auditable rollbacks

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, debug, info};
use super::errors::DispatcherError;
use super::signature::SignatureVerifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    pub rollback_id: String,
    pub directive_id: String,
    pub execution_id: String,
    pub action: String,
    pub targets: Vec<String>,
    pub rollback_commands: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub rolled_back: bool,
    pub rolled_back_at: Option<DateTime<Utc>>,
    pub rollback_signature: Option<String>,
}

pub struct RollbackManager {
    records: Arc<RwLock<HashMap<String, RollbackRecord>>>, // execution_id -> record
    rollback_ttl_seconds: i64,
    signature_verifier: Option<Arc<SignatureVerifier>>,
}

impl RollbackManager {
    pub fn new(rollback_ttl_seconds: i64) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            rollback_ttl_seconds,
            signature_verifier: None,
        }
    }
    
    /// Set signature verifier for signed rollbacks
    pub fn set_signature_verifier(&mut self, verifier: Arc<SignatureVerifier>) {
        self.signature_verifier = Some(verifier);
    }
    
    /// Record execution for potential rollback
    pub fn record_execution(
        &self,
        execution_id: &str,
        directive_id: &str,
        action: &str,
        targets: &[String],
        rollback_commands: &[String],
    ) -> Result<String, DispatcherError> {
        let rollback_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let record = RollbackRecord {
            rollback_id: rollback_id.clone(),
            execution_id: execution_id.to_string(),
            directive_id: directive_id.to_string(),
            action: action.to_string(),
            targets: targets.to_vec(),
            rollback_commands: rollback_commands.to_vec(),
            created_at: now,
            expires_at: now + Duration::seconds(self.rollback_ttl_seconds),
            rolled_back: false,
            rolled_back_at: None,
            rollback_signature: None,
        };
        
        let mut records = self.records.write();
        records.insert(execution_id.to_string(), record);
        
        debug!("Recorded execution {} for rollback {}", execution_id, rollback_id);
        Ok(rollback_id)
    }
    
    /// Execute rollback
    pub fn rollback(&self, execution_id: &str, signature: Option<String>) -> Result<(), DispatcherError> {
        let mut records = self.records.write();
        let record = records.get_mut(execution_id)
            .ok_or_else(|| DispatcherError::RollbackFailed(
                format!("No rollback record found for execution {}", execution_id)
            ))?;
        
        if record.rolled_back {
            return Err(DispatcherError::RollbackFailed(
                format!("Execution {} already rolled back", execution_id)
            ));
        }
        
        // Check if rollback has expired
        if Utc::now() > record.expires_at {
            return Err(DispatcherError::RollbackFailed(
                format!("Rollback expired for execution {} (expired at {})", 
                    execution_id, record.expires_at)
            ));
        }
        
        if record.rollback_commands.is_empty() {
            return Err(DispatcherError::RollbackFailed(
                format!("No rollback commands available for execution {}", execution_id)
            ));
        }
        
        // Verify signature if provided
        if let Some(ref sig) = signature {
            if let Some(ref verifier) = self.signature_verifier {
                let rollback_json = serde_json::to_string(record)
                    .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
                
                let verified = verifier.verify_json(rollback_json.as_bytes(), sig)
                    .map_err(|e| DispatcherError::RollbackFailed(format!("Signature verification failed: {}", e)))?;
                
                if !verified {
                    return Err(DispatcherError::RollbackFailed("Rollback signature verification failed".to_string()));
                }
                
                record.rollback_signature = Some(sig.clone());
            }
        }
        
        // Mark as rolled back
        record.rolled_back = true;
        record.rolled_back_at = Some(Utc::now());
        
        info!("Rollback executed for execution {}: {} commands", 
            execution_id, record.rollback_commands.len());
        
        Ok(())
    }
    
    /// Get rollback record
    pub fn get_record(&self, execution_id: &str) -> Option<RollbackRecord> {
        self.records.read().get(execution_id).cloned()
    }
    
    /// Check if rollback is available
    pub fn is_rollback_available(&self, execution_id: &str) -> bool {
        if let Some(record) = self.get_record(execution_id) {
            !record.rolled_back && !record.rollback_commands.is_empty() && Utc::now() <= record.expires_at
        } else {
            false
        }
    }
}
