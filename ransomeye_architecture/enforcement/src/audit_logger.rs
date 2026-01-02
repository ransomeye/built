// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/audit_logger.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tamper-evident audit logging for all enforcement actions

use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub violation_type: String,
    pub source_component: String,
    pub target_component: Option<String>,
    pub violation_details: String,
    pub action_taken: String,
    pub component_identity: Option<String>,
    pub previous_hash: String,
    pub entry_hash: String,
}

#[derive(Clone)]
pub struct AuditLogger {
    log_path: PathBuf,
    last_hash: Arc<Mutex<String>>,
}

impl AuditLogger {
    pub fn new(log_path: PathBuf) -> Result<Self, std::io::Error> {
        // Ensure log directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Initialize hash chain
        let last_hash = Arc::new(Mutex::new("0".repeat(64))); // Initial hash
        
        Ok(AuditLogger {
            log_path,
            last_hash,
        })
    }
    
    fn get_inner_writer(&self) -> Result<BufWriter<File>, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        Ok(BufWriter::new(file))
    }
    
    pub fn log_violation(
        &self,
        violation_type: &str,
        source_component: &str,
        target_component: Option<&str>,
        violation_details: &str,
        action_taken: &str,
        component_identity: Option<&str>,
    ) -> Result<(), std::io::Error> {
        let timestamp = Utc::now();
        
        // Get previous hash
        let previous_hash = self.last_hash.lock().unwrap().clone();
        
        // Create entry
        let mut entry = AuditEntry {
            timestamp,
            violation_type: violation_type.to_string(),
            source_component: source_component.to_string(),
            target_component: target_component.map(|s| s.to_string()),
            violation_details: violation_details.to_string(),
            action_taken: action_taken.to_string(),
            component_identity: component_identity.map(|s| s.to_string()),
            previous_hash: previous_hash.clone(),
            entry_hash: String::new(), // Will be computed
        };
        
        // Compute hash of entry (excluding entry_hash itself)
        let entry_json = serde_json::to_string(&serde_json::json!({
            "timestamp": entry.timestamp,
            "violation_type": entry.violation_type,
            "source_component": entry.source_component,
            "target_component": entry.target_component,
            "violation_details": entry.violation_details,
            "action_taken": entry.action_taken,
            "component_identity": entry.component_identity,
            "previous_hash": entry.previous_hash,
        }))?;
        
        let mut hasher = Sha256::new();
        hasher.update(previous_hash.as_bytes());
        hasher.update(entry_json.as_bytes());
        let entry_hash = format!("{:x}", hasher.finalize());
        entry.entry_hash = entry_hash.clone();
        
        // Update last hash
        *self.last_hash.lock().unwrap() = entry_hash;
        
        // Write entry
        let mut writer = self.get_inner_writer()?;
        writeln!(writer, "{}", serde_json::to_string(&entry)?)?;
        writer.flush()?;
        
        Ok(())
    }
    
    pub fn log_fail_closed(
        &self,
        component: &str,
        reason: &str,
    ) -> Result<(), std::io::Error> {
        self.log_violation(
            "FAIL_CLOSED",
            component,
            None,
            reason,
            "ABORT",
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(log_path.clone()).unwrap();
        
        logger.log_violation(
            "FORBIDDEN_FLOW",
            "ransomeye_ai_core",
            Some("ransomeye_alert_engine"),
            "AI attempted to access Control Plane",
            "PROCESS_TERMINATED",
            Some("identity_hash_123"),
        ).unwrap();
        
        // Verify log file exists and contains entry
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(log_content.contains("FORBIDDEN_FLOW"));
        assert!(log_content.contains("ransomeye_ai_core"));
    }
}

