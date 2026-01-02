// Path and File Name : /home/ransomeye/rebuild/core/audit/src/chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Hash chain implementation - Merkle-style hash chaining for tamper-proof audit logs

use sha2::{Sha256, Digest};
use hex;
use std::sync::Arc;
use parking_lot::RwLock;

/// Hash chain for tamper-proof audit logging
pub struct HashChain {
    previous_hash: Arc<RwLock<Option<String>>>,
}

impl HashChain {
    /// Create new hash chain
    pub fn new() -> Self {
        Self {
            previous_hash: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Compute hash for audit record
    /// Includes previous hash in computation for chain integrity
    pub fn compute_hash(&self, record_data: &[u8], previous_hash: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        
        // Include previous hash in computation
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        } else {
            // Genesis hash
            hasher.update(b"RANSOMEYE_AUDIT_GENESIS");
        }
        
        // Include record data
        hasher.update(record_data);
        
        hex::encode(hasher.finalize())
    }
    
    /// Get previous hash
    pub fn get_previous_hash(&self) -> Option<String> {
        self.previous_hash.read().clone()
    }
    
    /// Set previous hash (after successful write)
    pub fn set_previous_hash(&self, hash: String) {
        *self.previous_hash.write() = Some(hash);
    }
    
    /// Verify hash chain integrity
    pub fn verify_chain(&self, records: &[AuditRecord]) -> Result<bool, String> {
        if records.is_empty() {
            return Ok(true);
        }
        
        let mut previous_hash: Option<String> = None;
        
        for (i, record) in records.iter().enumerate() {
            // Compute expected hash
            let record_data = serde_json::to_string(record)
                .map_err(|e| format!("Failed to serialize record: {}", e))?;
            
            let expected_hash = self.compute_hash(record_data.as_bytes(), previous_hash.as_deref());
            
            // Verify hash matches
            if record.hash != expected_hash {
                return Err(format!("Hash mismatch at record {}: expected {}, got {}", 
                                 i, expected_hash, record.hash));
            }
            
            // Verify previous hash matches (except for first record)
            if i > 0 {
                let expected_prev = previous_hash.as_ref().map(|s| s.as_str()).unwrap_or("");
                if record.previous_hash != expected_prev {
                    return Err(format!("Previous hash mismatch at record {}: expected {}, got {}", 
                                     i, expected_prev, record.previous_hash));
                }
            }
            
            previous_hash = Some(record.hash.clone());
        }
        
        Ok(true)
    }
}

/// Audit record with hash chain
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditRecord {
    pub record_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub event_type: String,
    pub actor: String,
    pub host: String,
    pub previous_hash: String,
    pub hash: String,
    pub signature: String,
    pub data: serde_json::Value,
}

impl Default for HashChain {
    fn default() -> Self {
        Self::new()
    }
}

