// Path and File Name : /home/ransomeye/rebuild/core/forensics/src/evidence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence collection - collects and preserves forensic evidence

use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::errors::ForensicsError;
use crate::integrity::{EvidenceIntegrity, EvidenceItem};

/// Evidence collector - collects forensic evidence
pub struct EvidenceCollector {
    integrity: EvidenceIntegrity,
}

impl EvidenceCollector {
    /// Create new evidence collector
    pub fn new() -> Self {
        Self {
            integrity: EvidenceIntegrity::new(),
        }
    }
    
    /// Collect evidence
    /// 
    /// Evidence is:
    /// - Content-addressed (hash-based ID)
    /// - Signed
    /// - Immutable once stored
    pub fn collect(
        &self,
        evidence_type: &str,
        source: &str,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Result<EvidenceItem, ForensicsError> {
        // Serialize data for hashing
        let data_json = serde_json::to_string(&data)
            .map_err(|e| ForensicsError::SerializationError(format!("Failed to serialize data: {}", e)))?;
        
        // Compute content hash
        let content_hash = self.integrity.compute_hash(data_json.as_bytes());
        
        // Use hash as evidence ID (content-addressed)
        let evidence_id = format!("ev_{}", content_hash);
        
        // Sign evidence
        let signature = self.integrity.sign(data_json.as_bytes());
        
        let evidence = EvidenceItem {
            evidence_id,
            content_hash,
            signature,
            timestamp: Utc::now(),
            evidence_type: evidence_type.to_string(),
            source: source.to_string(),
            data,
            metadata,
        };
        
        Ok(evidence)
    }
    
    /// Verify evidence integrity
    pub fn verify(&self, evidence: &EvidenceItem) -> Result<(), ForensicsError> {
        // Serialize data
        let data_json = serde_json::to_string(&evidence.data)
            .map_err(|e| ForensicsError::SerializationError(format!("Failed to serialize data: {}", e)))?;
        
        // Verify hash
        self.integrity.verify_hash(data_json.as_bytes(), &evidence.content_hash)
            .map_err(|e| ForensicsError::IntegrityFailed(format!("Hash verification failed: {}", e)))?;
        
        // Verify signature
        self.integrity.verify(data_json.as_bytes(), &evidence.signature)
            .map_err(|e| ForensicsError::IntegrityFailed(format!("Signature verification failed: {}", e)))?;
        
        Ok(())
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}

