// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/audit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Append-only hash-chained audit logging

use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter};
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use tracing::{error, warn, debug, info};
use super::errors::DispatcherError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: String,
    pub previous_hash: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub event_type: AuditEventType,
    pub directive_id: Option<String>,
    pub policy_id: Option<String>,
    pub signature_hash: Option<String>,
    pub action: Option<String>,
    pub execution_result: Option<String>,
    pub agent_id: Option<String>,
    pub details: serde_json::Value,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    DirectiveReceived,
    DirectiveValidated,
    DirectiveRejected,
    ExecutionAttempted,
    ExecutionSucceeded,
    ExecutionFailed,
    AcknowledgmentReceived,
    AcknowledgmentTimeout,
    RollbackInitiated,
    RollbackCompleted,
    RollbackFailed,
    Escalation,
}

pub struct AuditLogger {
    log_file: Arc<RwLock<BufWriter<File>>>,
    last_hash: Arc<RwLock<String>>,
    log_path: String,
}

impl AuditLogger {
    pub fn new(log_path: &str) -> Result<Self, DispatcherError> {
        let path = Path::new(log_path);
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DispatcherError::ConfigurationError(
                    format!("Failed to create audit log directory: {}", e)
                ))?;
        }
        
        // Open file in append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| DispatcherError::ConfigurationError(
                format!("Failed to open audit log file {}: {}", log_path, e)
            ))?;
        
        let writer = BufWriter::new(file);
        
        // Read last hash from file (if exists)
        let last_hash = Self::read_last_hash(path)?;
        
        Ok(Self {
            log_file: Arc::new(RwLock::new(writer)),
            last_hash: Arc::new(RwLock::new(last_hash)),
            log_path: log_path.to_string(),
        })
    }
    
    fn read_last_hash(path: &Path) -> Result<String, DispatcherError> {
        if !path.exists() {
            return Ok("0".repeat(64)); // Initial hash for empty log
        }
        
        // Read last line to get hash
        let content = std::fs::read_to_string(path)
            .map_err(|e| DispatcherError::InternalError(format!("Failed to read audit log: {}", e)))?;
        
        let lines: Vec<&str> = content.lines().collect();
        if let Some(last_line) = lines.last() {
            if let Ok(record) = serde_json::from_str::<AuditRecord>(last_line) {
                return Ok(record.hash);
            }
        }
        
        Ok("0".repeat(64))
    }
    
    /// Append audit record (append-only, hash-chained)
    pub fn append(&self, event_type: AuditEventType, details: serde_json::Value) -> Result<String, DispatcherError> {
        let record_id = uuid::Uuid::new_v4().to_string();
        let previous_hash = self.last_hash.read().clone();
        let timestamp = Utc::now();
        
        // Create record (without hash first)
        let mut record = AuditRecord {
            record_id: record_id.clone(),
            previous_hash: previous_hash.clone(),
            timestamp,
            event_type,
            directive_id: details.get("directive_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            policy_id: details.get("policy_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            signature_hash: details.get("signature_hash").and_then(|v| v.as_str()).map(|s| s.to_string()),
            action: details.get("action").and_then(|v| v.as_str()).map(|s| s.to_string()),
            execution_result: details.get("execution_result").and_then(|v| v.as_str()).map(|s| s.to_string()),
            agent_id: details.get("agent_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            details: details.clone(),
            hash: String::new(), // Will be computed
        };
        
        // Compute hash
        let record_json = serde_json::to_string(&record)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(record_json.as_bytes());
        let hash = hex::encode(hasher.finalize());
        
        record.hash = hash.clone();
        
        // Write to file (append-only)
        let record_json_final = serde_json::to_string(&record)
            .map_err(|e| DispatcherError::InternalError(format!("Serialization failed: {}", e)))?;
        
        {
            let mut writer = self.log_file.write();
            writeln!(writer, "{}", record_json_final)
                .map_err(|e| DispatcherError::InternalError(format!("Failed to write audit log: {}", e)))?;
            writer.flush()
                .map_err(|e| DispatcherError::InternalError(format!("Failed to flush audit log: {}", e)))?;
        }
        
        // Update last hash
        {
            let mut last = self.last_hash.write();
            *last = hash.clone();
        }
        
        debug!("Audit record appended: {} (hash: {})", record_id, hash);
        Ok(record_id)
    }
    
    /// Get last hash (for verification)
    pub fn get_last_hash(&self) -> String {
        self.last_hash.read().clone()
    }
}
