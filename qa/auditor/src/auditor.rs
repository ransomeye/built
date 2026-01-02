// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/src/auditor.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit trail auditor - validates evidence integrity, retention enforcement, and audit completeness

use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Evidence integrity check failed: {0}")]
    IntegrityFailed(String),
    #[error("Retention violation: {0}")]
    RetentionViolation(String),
    #[error("Audit trail incomplete: {0}")]
    IncompleteTrail(String),
    #[error("Reproducibility check failed: {0}")]
    ReproducibilityFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub outcome: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub total_entries: usize,
    pub integrity_passed: bool,
    pub retention_passed: bool,
    pub completeness_passed: bool,
    pub violations: Vec<String>,
}

pub struct Auditor {
    retention_years: i64,
    audit_log: Vec<AuditEntry>,
}

impl Auditor {
    pub fn new(retention_years: i64) -> Self {
        Self {
            retention_years,
            audit_log: Vec::new(),
        }
    }
    
    pub fn load_audit_log(&mut self, log_path: &PathBuf) -> Result<(), AuditError> {
        info!("Loading audit log from: {:?}", log_path);
        
        let content = std::fs::read_to_string(log_path)
            .map_err(|e| AuditError::IncompleteTrail(format!("Failed to read audit log: {}", e)))?;
        
        let entries: Vec<AuditEntry> = serde_json::from_str(&content)
            .map_err(|e| AuditError::IncompleteTrail(format!("Failed to parse audit log: {}", e)))?;
        
        self.audit_log = entries;
        info!("Loaded {} audit entries", self.audit_log.len());
        Ok(())
    }
    
    pub fn audit_evidence_integrity(&self) -> Result<(), AuditError> {
        info!("Auditing evidence integrity");
        
        // Check that all entries have signatures
        let unsigned_entries: Vec<_> = self.audit_log.iter()
            .enumerate()
            .filter(|(_, entry)| entry.signature.is_none())
            .map(|(idx, _)| idx)
            .collect();
        
        if !unsigned_entries.is_empty() {
            return Err(AuditError::IntegrityFailed(
                format!("Found {} unsigned audit entries: {:?}", 
                    unsigned_entries.len(), unsigned_entries)
            ));
        }
        
        // Check timestamp ordering
        for i in 1..self.audit_log.len() {
            if self.audit_log[i].timestamp < self.audit_log[i-1].timestamp {
                return Err(AuditError::IntegrityFailed(
                    format!("Timestamp ordering violation at entry {}", i)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn audit_retention(&self) -> Result<(), AuditError> {
        info!("Auditing retention policy ({} years)", self.retention_years);
        
        let cutoff_date = Utc::now() - Duration::days(self.retention_years * 365);
        
        // Check that no entries older than retention period exist
        let old_entries: Vec<_> = self.audit_log.iter()
            .enumerate()
            .filter(|(_, entry)| entry.timestamp < cutoff_date)
            .map(|(idx, entry)| (idx, entry.timestamp))
            .collect();
        
        if !old_entries.is_empty() {
            return Err(AuditError::RetentionViolation(
                format!("Found {} entries older than retention period: {:?}", 
                    old_entries.len(), old_entries)
            ));
        }
        
        Ok(())
    }
    
    pub fn audit_completeness(&self) -> Result<(), AuditError> {
        info!("Auditing trail completeness");
        
        // Check for gaps in timestamps (more than 1 hour)
        let mut gaps = Vec::new();
        for i in 1..self.audit_log.len() {
            let gap = self.audit_log[i].timestamp - self.audit_log[i-1].timestamp;
            if gap > chrono::Duration::hours(1) {
                gaps.push((i, gap));
            }
        }
        
        if !gaps.is_empty() {
            return Err(AuditError::IncompleteTrail(
                format!("Found {} timestamp gaps in audit trail: {:?}", gaps.len(), gaps)
            ));
        }
        
        // Check that critical actions are logged
        let critical_actions = vec!["DELETE", "MODIFY", "EXPORT", "SIGN"];
        let mut missing_critical = Vec::new();
        
        for action in &critical_actions {
            let found = self.audit_log.iter()
                .any(|entry| entry.action.contains(action));
            if !found {
                missing_critical.push(action);
            }
        }
        
        // Fail-closed: missing critical actions is a failure
        if !missing_critical.is_empty() {
            return Err(AuditError::IncompleteTrail(
                format!("Critical actions not found in log: {:?}", missing_critical)
            ));
        }
        
        Ok(())
    }
    
    pub fn audit_reproducibility(&self) -> Result<(), AuditError> {
        info!("Auditing reproducibility");
        
        // Check that all entries have sufficient information for replay
        let incomplete_entries: Vec<_> = self.audit_log.iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.action.is_empty() || 
                entry.actor.is_empty() || 
                entry.resource.is_empty()
            })
            .map(|(idx, _)| idx)
            .collect();
        
        if !incomplete_entries.is_empty() {
            return Err(AuditError::ReproducibilityFailed(
                format!("Found {} incomplete audit entries: {:?}", 
                    incomplete_entries.len(), incomplete_entries)
            ));
        }
        
        Ok(())
    }
    
    pub fn run_full_audit(&self) -> Result<AuditResult, AuditError> {
        let mut violations = Vec::new();
        
        // Integrity check - fail-closed: any error is a failure
        let integrity_passed = match self.audit_evidence_integrity() {
            Ok(()) => true,
            Err(e) => {
                violations.push(format!("Integrity: {}", e));
                false
            }
        };
        
        // Retention check - fail-closed: any error is a failure
        let retention_passed = match self.audit_retention() {
            Ok(()) => true,
            Err(e) => {
                violations.push(format!("Retention: {}", e));
                false
            }
        };
        
        // Completeness check - fail-closed: any error is a failure
        let completeness_passed = match self.audit_completeness() {
            Ok(()) => true,
            Err(e) => {
                violations.push(format!("Completeness: {}", e));
                false
            }
        };
        
        // Reproducibility check - fail-closed: any error is a failure
        let _reproducibility_passed = match self.audit_reproducibility() {
            Ok(()) => true,
            Err(e) => {
                violations.push(format!("Reproducibility: {}", e));
                false
            }
        };
        
        Ok(AuditResult {
            total_entries: self.audit_log.len(),
            integrity_passed,
            retention_passed,
            completeness_passed,
            violations,
        })
    }
}

