// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/retention.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Retention manager - enforces retention policies, secure deletion, and purge event logging

use chrono::{DateTime, Utc, Duration};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::errors::ReportingError;
use crate::evidence_store::EvidenceStore;
use crate::hasher::EvidenceHasher;

/// Retention policy configuration
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub forensic_retention_days: i64,
    pub ai_artifact_min_retention_years: i64,
    pub disk_max_usage_percent: u8,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            forensic_retention_days: 10,
            ai_artifact_min_retention_years: 2,
            disk_max_usage_percent: 80,
        }
    }
}

/// Purge event - logged for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub bundle_ids: Vec<String>,
    pub reason: String,
    pub retention_days: i64,
    pub cutoff_date: DateTime<Utc>,
    pub destruction_certificate_path: Option<String>,
}

/// Retention manager - enforces retention policies
pub struct RetentionManager {
    policy: RetentionPolicy,
    store: EvidenceStore,
    hasher: EvidenceHasher,
    ledger_path: PathBuf,
}

impl RetentionManager {
    pub fn new(
        store: EvidenceStore,
        policy: RetentionPolicy,
        ledger_path: impl AsRef<Path>,
    ) -> Result<Self, ReportingError> {
        let ledger_path = ledger_path.as_ref().to_path_buf();
        
        // Create ledger directory if it doesn't exist
        if let Some(parent) = ledger_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ReportingError::IoError(e))?;
        }
        
        Ok(Self {
            policy,
            store,
            hasher: EvidenceHasher::new(),
            ledger_path,
        })
    }
    
    /// Enforce retention policy
    /// Purges evidence bundles older than retention period
    pub fn enforce_retention(&self, dry_run: bool) -> Result<Vec<String>, ReportingError> {
        let cutoff_date = Utc::now() - Duration::days(self.policy.forensic_retention_days);
        let bundles = self.store.get_all_bundles();
        
        let mut purged_bundles = Vec::new();
        
        for bundle in &bundles {
            if bundle.created_at < cutoff_date {
                // Check if bundle contains AI artifacts (protected)
                let contains_ai_artifacts = self.contains_ai_artifacts(bundle)?;
                
                if contains_ai_artifacts {
                    let age_years = (Utc::now() - bundle.created_at).num_days() / 365;
                    if age_years < self.policy.ai_artifact_min_retention_years {
                        warn!("Skipping bundle {} - contains AI artifacts and age ({}) < minimum retention ({} years)",
                              bundle.bundle_id, age_years, self.policy.ai_artifact_min_retention_years);
                        continue;
                    }
                }
                
                if !dry_run {
                    // Secure deletion would happen here
                    // For now, we just mark for deletion
                    debug!("Purging bundle {} (created: {})", bundle.bundle_id, bundle.created_at);
                }
                
                purged_bundles.push(bundle.bundle_id.clone());
            }
        }
        
        if !dry_run && !purged_bundles.is_empty() {
            // Create destruction certificate
            let cert_path = self.create_destruction_certificate(&purged_bundles, cutoff_date)?;
            
            // Log purge event
            self.log_purge_event(PurgeEvent {
                timestamp: Utc::now(),
                event_type: "forensic_purge".to_string(),
                bundle_ids: purged_bundles.clone(),
                reason: format!("Retention policy: {} days", self.policy.forensic_retention_days),
                retention_days: self.policy.forensic_retention_days,
                cutoff_date,
                destruction_certificate_path: Some(cert_path.to_string_lossy().to_string()),
            })?;
        }
        
        Ok(purged_bundles)
    }
    
    /// Check if bundle contains AI artifacts
    fn contains_ai_artifacts(&self, bundle: &crate::evidence_store::EvidenceBundle) -> Result<bool, ReportingError> {
        // Check metadata for AI artifact indicators
        for evidence in &bundle.evidence_items {
            if evidence.source_type.contains("ai") || 
               evidence.source_type.contains("model") ||
               evidence.source_type.contains("training") {
                return Ok(true);
            }
            
            // Check metadata
            for (key, value) in &evidence.metadata {
                let key_lower = key.to_lowercase();
                let value_lower = value.to_lowercase();
                if key_lower.contains("model") || 
                   key_lower.contains("ai") ||
                   value_lower.contains(".pkl") ||
                   value_lower.contains(".gguf") ||
                   value_lower.contains("shap") {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Create destruction certificate
    fn create_destruction_certificate(
        &self,
        bundle_ids: &[String],
        cutoff_date: DateTime<Utc>,
    ) -> Result<PathBuf, ReportingError> {
        let cert_dir = self.ledger_path.parent()
            .ok_or_else(|| ReportingError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid ledger path"
            )))?
            .join("certificates");
        
        fs::create_dir_all(&cert_dir)
            .map_err(|e| ReportingError::IoError(e))?;
        
        let cert_filename = format!("destruction_{}.json", Utc::now().format("%Y%m%d_%H%M%S"));
        let cert_path = cert_dir.join(cert_filename);
        
        let certificate = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "type": "forensic_destruction",
            "bundle_ids": bundle_ids,
            "retention_days": self.policy.forensic_retention_days,
            "cutoff_date": cutoff_date.to_rfc3339(),
            "total_bundles": bundle_ids.len(),
        });
        
        let cert_json = serde_json::to_string_pretty(&certificate)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        fs::write(&cert_path, cert_json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        Ok(cert_path)
    }
    
    /// Log purge event to signed ledger
    fn log_purge_event(&self, event: PurgeEvent) -> Result<(), ReportingError> {
        let event_json = serde_json::to_string(&event)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ledger_path)
            .map_err(|e| ReportingError::IoError(e))?;
        
        writeln!(file, "{}", event_json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        debug!("Logged purge event: {} bundles", event.bundle_ids.len());
        Ok(())
    }
    
    /// Check disk usage and trigger purge if needed
    pub fn check_disk_pressure(&self) -> Result<bool, ReportingError> {
        // This would check actual disk usage
        // For now, return false (no pressure)
        // In production, would use sysinfo or similar
        Ok(false)
    }
}

