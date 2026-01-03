// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/collector.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence collector - gathers evidence from various sources and prepares it for preservation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::errors::ReportingError;
use crate::hasher::EvidenceHasher;

/// Evidence collector - gathers and prepares evidence for preservation
/// Ensures all evidence is properly attributed and timestamped
#[cfg(feature = "future-reporting")]
pub struct EvidenceCollector {
    hasher: EvidenceHasher,
    engine_version: String,
    policy_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "future-reporting")]
pub struct CollectedEvidence {
    pub evidence_id: String,
    pub source: String,
    pub source_type: String,
    pub timestamp: DateTime<Utc>,
    pub kill_chain_stage: Option<String>,
    pub data: Value,
    pub metadata: HashMap<String, String>,
    pub integrity_hash: String,
}

#[cfg(feature = "future-reporting")]
impl EvidenceCollector {
    pub fn new(engine_version: &str, policy_version: &str) -> Self {
        Self {
            hasher: EvidenceHasher::new(),
            engine_version: engine_version.to_string(),
            policy_version: policy_version.to_string(),
        }
    }
    
    /// Collect evidence from a source
    /// All evidence must have explicit timestamps (UTC) and source attribution
    pub fn collect(
        &self,
        source: &str,
        source_type: &str,
        data: Value,
        kill_chain_stage: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<CollectedEvidence, ReportingError> {
        let evidence_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Build evidence document
        let mut evidence_doc = serde_json::Map::new();
        evidence_doc.insert("evidence_id".to_string(), Value::String(evidence_id.clone()));
        evidence_doc.insert("source".to_string(), Value::String(source.to_string()));
        evidence_doc.insert("source_type".to_string(), Value::String(source_type.to_string()));
        evidence_doc.insert("timestamp".to_string(), Value::String(timestamp.to_rfc3339()));
        evidence_doc.insert("engine_version".to_string(), Value::String(self.engine_version.clone()));
        evidence_doc.insert("policy_version".to_string(), Value::String(self.policy_version.clone()));
        evidence_doc.insert("data".to_string(), data.clone());
        
        if let Some(stage) = &kill_chain_stage {
            evidence_doc.insert("kill_chain_stage".to_string(), Value::String(stage.clone()));
        }
        
        // Add metadata
        let mut metadata_value = serde_json::Map::new();
        for (k, v) in &metadata {
            metadata_value.insert(k.clone(), Value::String(v.clone()));
        }
        evidence_doc.insert("metadata".to_string(), Value::Object(metadata_value));
        
        // Compute integrity hash
        let evidence_value = Value::Object(evidence_doc);
        let integrity_hash = self.hasher.hash_evidence(&evidence_value);
        
        debug!("Collected evidence {} from {} (hash: {})", evidence_id, source, integrity_hash);
        
        Ok(CollectedEvidence {
            evidence_id,
            source: source.to_string(),
            source_type: source_type.to_string(),
            timestamp,
            kill_chain_stage,
            data,
            metadata,
            integrity_hash,
        })
    }
    
    /// Collect evidence with explicit timestamp (for historical reconstruction)
    pub fn collect_with_timestamp(
        &self,
        source: &str,
        source_type: &str,
        data: Value,
        timestamp: DateTime<Utc>,
        kill_chain_stage: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<CollectedEvidence, ReportingError> {
        // Validate timestamp is not in the future
        let now = Utc::now();
        if timestamp > now {
            return Err(ReportingError::InvalidTimestamp(
                format!("Timestamp {} is in the future", timestamp)
            ));
        }
        
        let evidence_id = Uuid::new_v4().to_string();
        
        // Build evidence document
        let mut evidence_doc = serde_json::Map::new();
        evidence_doc.insert("evidence_id".to_string(), Value::String(evidence_id.clone()));
        evidence_doc.insert("source".to_string(), Value::String(source.to_string()));
        evidence_doc.insert("source_type".to_string(), Value::String(source_type.to_string()));
        evidence_doc.insert("timestamp".to_string(), Value::String(timestamp.to_rfc3339()));
        evidence_doc.insert("engine_version".to_string(), Value::String(self.engine_version.clone()));
        evidence_doc.insert("policy_version".to_string(), Value::String(self.policy_version.clone()));
        evidence_doc.insert("data".to_string(), data.clone());
        
        if let Some(stage) = &kill_chain_stage {
            evidence_doc.insert("kill_chain_stage".to_string(), Value::String(stage.clone()));
        }
        
        // Add metadata
        let mut metadata_value = serde_json::Map::new();
        for (k, v) in &metadata {
            metadata_value.insert(k.clone(), Value::String(v.clone()));
        }
        evidence_doc.insert("metadata".to_string(), Value::Object(metadata_value));
        
        // Compute integrity hash
        let evidence_value = Value::Object(evidence_doc);
        let integrity_hash = self.hasher.hash_evidence(&evidence_value);
        
        debug!("Collected evidence {} from {} with timestamp {} (hash: {})", 
               evidence_id, source, timestamp, integrity_hash);
        
        Ok(CollectedEvidence {
            evidence_id,
            source: source.to_string(),
            source_type: source_type.to_string(),
            timestamp,
            kill_chain_stage,
            data,
            metadata,
            integrity_hash,
        })
    }
}

