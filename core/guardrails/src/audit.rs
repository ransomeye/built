// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/audit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit logging with tamper evidence for guardrail violations

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use crate::errors::{GuardrailError, GuardrailResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: String,
    pub violation_type: String,
    pub phase: Option<String>,
    pub module: Option<String>,
    pub file_path: Option<String>,
    pub violation_details: String,
    pub spec_hash: String,
    pub record_hash: String,
    pub previous_hash: Option<String>,
}

pub struct AuditLogger {
    audit_path: PathBuf,
}

impl AuditLogger {
    pub fn new<P: AsRef<std::path::Path>>(audit_path: P) -> Self {
        Self {
            audit_path: audit_path.as_ref().to_path_buf(),
        }
    }

    pub fn default() -> Self {
        Self::new("/var/log/ransomeye/guardrails_audit.jsonl")
    }

    /// Log a violation with tamper evidence
    pub fn log_violation(
        &self,
        violation_type: &str,
        phase: Option<&str>,
        module: Option<&str>,
        file_path: Option<&str>,
        violation_details: &str,
        spec_hash: &str,
    ) -> GuardrailResult<()> {
        // Get previous hash from last record
        let previous_hash = self.get_last_record_hash()?;

        // Create audit record
        let mut record = AuditRecord {
            timestamp: Utc::now().to_rfc3339(),
            violation_type: violation_type.to_string(),
            phase: phase.map(|s| s.to_string()),
            module: module.map(|s| s.to_string()),
            file_path: file_path.map(|s| s.to_string()),
            violation_details: violation_details.to_string(),
            spec_hash: spec_hash.to_string(),
            record_hash: String::new(),
            previous_hash,
        };

        // Compute hash of record (excluding record_hash)
        let record_hash = self.compute_record_hash(&record)?;
        record.record_hash = record_hash.clone();

        // Serialize and append to audit log (append-only)
        let json = serde_json::to_string(&record)
            .map_err(|e| GuardrailError::AuditFailed(format!("JSON serialization: {}", e)))?;

        // Ensure audit directory exists
        if let Some(parent) = self.audit_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GuardrailError::AuditFailed(format!("Create audit dir: {}", e)))?;
        }

        // Append to audit log
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_path)
            .map_err(|e| GuardrailError::AuditFailed(format!("Open audit file: {}", e)))?;

        writeln!(file, "{}", json)
            .map_err(|e| GuardrailError::AuditFailed(format!("Write audit record: {}", e)))?;

        file.sync_all()
            .map_err(|e| GuardrailError::AuditFailed(format!("Sync audit file: {}", e)))?;

        Ok(())
    }

    fn compute_record_hash(&self, record: &AuditRecord) -> GuardrailResult<String> {
        let mut record_for_hash = record.clone();
        record_for_hash.record_hash = String::new();

        let json = serde_json::to_string(&record_for_hash)
            .map_err(|e| GuardrailError::AuditFailed(format!("JSON serialization: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        if let Some(ref prev_hash) = record.previous_hash {
            hasher.update(prev_hash.as_bytes());
        }
        let hash = hasher.finalize();

        Ok(hex::encode(hash))
    }

    fn get_last_record_hash(&self) -> GuardrailResult<Option<String>> {
        if !self.audit_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.audit_path)
            .map_err(|e| GuardrailError::AuditFailed(format!("Read audit file: {}", e)))?;

        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        if let Some(last_line) = lines.last() {
            let record: AuditRecord = serde_json::from_str(last_line)
                .map_err(|e| GuardrailError::AuditFailed(format!("Parse last record: {}", e)))?;
            Ok(Some(record.record_hash))
        } else {
            Ok(None)
        }
    }
}

