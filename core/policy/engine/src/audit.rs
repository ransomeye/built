// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/audit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Append-only audit logging with hash-chained records

use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use sha2::{Sha256, Digest};
use hex;
use chrono::{DateTime, Utc};
use tracing::{error, debug, warn};
use parking_lot::RwLock;
use once_cell::sync::Lazy;

use crate::errors::PolicyError;
use crate::decision::PolicyDecision;

static AUDIT_LOG: Lazy<Arc<RwLock<AuditLogger>>> = Lazy::new(|| {
    Arc::new(RwLock::new(AuditLogger::new()))
});

pub struct AuditLogger {
    log_path: Option<String>,
    previous_hash: Option<String>,
}

impl AuditLogger {
    fn new() -> Self {
        Self {
            log_path: None,
            previous_hash: None,
        }
    }

    pub fn initialize(&mut self, log_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(log_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.exists() {
            self.previous_hash = self.load_last_hash(path)?;
        } else {
            std::fs::File::create(path)?;
        }

        self.log_path = Some(log_path.to_string());
        debug!("Audit logger initialized: {}", log_path);
        Ok(())
    }

    fn load_last_hash(&self, path: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
        use std::io::{BufRead, BufReader};
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut last_hash = None;

        for line in reader.lines() {
            let line = line?;
            if let Some(hash) = line.strip_prefix("HASH:") {
                last_hash = Some(hash.trim().to_string());
            }
        }

        Ok(last_hash)
    }

    pub fn log_decision(&mut self, decision: &PolicyDecision, policy_signature_hash: &str) -> Result<String, PolicyError> {
        let log_path = self.log_path.as_ref()
            .ok_or_else(|| PolicyError::AuditLoggingFailed("Audit logger not initialized".to_string()))?;

        let record = AuditRecord {
            timestamp: Utc::now(),
            policy_id: decision.policy_id.clone(),
            policy_version: decision.policy_version.clone(),
            signature_hash: policy_signature_hash.to_string(),
            input_reference: decision.evidence_reference.clone(),
            decision: format!("{:?}", decision.decision),
            decision_id: decision.decision_id.clone(),
            previous_hash: self.previous_hash.clone(),
        };

        let record_hash = record.compute_hash();
        let record_json = serde_json::to_string(&record)
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to serialize audit record: {}", e)))?;

        let path = Path::new(log_path);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to open audit log: {}", e)))?;

        writeln!(file, "{}", record_json)
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to write audit record: {}", e)))?;
        writeln!(file, "HASH:{}", record_hash)
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to write hash: {}", e)))?;

        file.flush()
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to flush audit log: {}", e)))?;

        self.previous_hash = Some(record_hash.clone());
        debug!("Audit record logged: {}", record_hash);

        Ok(record_hash)
    }

    pub fn verify_chain(&self) -> Result<bool, PolicyError> {
        let log_path = self.log_path.as_ref()
            .ok_or_else(|| PolicyError::AuditLoggingFailed("Audit logger not initialized".to_string()))?;

        use std::io::{BufRead, BufReader};
        let file = std::fs::File::open(log_path)
            .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to open audit log: {}", e)))?;
        let reader = BufReader::new(file);

        let mut previous_hash: Option<String> = None;
        let mut records = Vec::new();
        let mut hashes = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("HASH:") {
                if let Some(hash) = line.strip_prefix("HASH:") {
                    hashes.push(hash.trim().to_string());
                }
            } else if !line.trim().is_empty() {
                if let Ok(record) = serde_json::from_str::<AuditRecord>(&line) {
                    records.push(record);
                }
            }
        }

        for (i, record) in records.iter().enumerate() {
            let computed_hash = record.compute_hash_with_previous(previous_hash.clone());
            if i < hashes.len() && computed_hash != hashes[i] {
                error!("Audit chain verification failed at record {}", i);
                return Ok(false);
            }
            previous_hash = Some(computed_hash);
        }

        debug!("Audit chain verified: {} records", records.len());
        Ok(true)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AuditRecord {
    timestamp: DateTime<Utc>,
    policy_id: String,
    policy_version: String,
    signature_hash: String,
    input_reference: String,
    decision: String,
    decision_id: String,
    previous_hash: Option<String>,
}

impl AuditRecord {
    fn compute_hash(&self) -> String {
        self.compute_hash_with_previous(self.previous_hash.clone())
    }

    fn compute_hash_with_previous(&self, previous: Option<String>) -> String {
        let mut hasher = Sha256::new();
        if let Some(prev) = previous {
            hasher.update(prev.as_bytes());
        }
        let json_bytes = serde_json::to_vec(self).expect("Failed to serialize");
        hasher.update(&json_bytes);
        hex::encode(hasher.finalize())
    }
}

pub fn initialize_audit_logger(log_path: &str) -> Result<(), PolicyError> {
    let mut logger = AUDIT_LOG.write();
    logger.initialize(log_path)
        .map_err(|e| PolicyError::AuditLoggingFailed(format!("Failed to initialize audit logger: {}", e)))
}

pub fn log_decision(decision: &PolicyDecision, policy_signature_hash: &str) -> Result<String, PolicyError> {
    let mut logger = AUDIT_LOG.write();
    logger.log_decision(decision, policy_signature_hash)
}

pub fn verify_audit_chain() -> Result<bool, PolicyError> {
    let logger = AUDIT_LOG.read();
    logger.verify_chain()
}

use std::sync::Arc;

