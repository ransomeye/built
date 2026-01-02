// Path and File Name : /home/ransomeye/rebuild/core/audit/src/logger.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit logger - append-only, hash-chained, signed audit logging with fail-closed semantics

use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use chrono::Utc;
use tracing::{error, warn, info, debug};
use serde::{Serialize, Deserialize};

use crate::errors::AuditError;
use crate::chain::{HashChain, AuditRecord};
use crate::signing::AuditSigner;
use crate::clock::ClockGuard;

/// Audit logger with tamper-proof guarantees
pub struct AuditLogger {
    log_path: String,
    chain: HashChain,
    signer: AuditSigner,
    clock: ClockGuard,
    writer: Option<BufWriter<std::fs::File>>,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(log_path: impl AsRef<Path>, signer: AuditSigner) -> Result<Self, AuditError> {
        let path = log_path.as_ref();
        
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AuditError::IoError(e))?;
        }
        
        // Open log file in append-only mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| AuditError::IoError(e))?;
        
        let writer = BufWriter::new(file);
        
        Ok(Self {
            log_path: path.to_string_lossy().to_string(),
            chain: HashChain::new(),
            signer,
            clock: ClockGuard::new(),
            writer: Some(writer),
        })
    }
    
    /// Log audit event
    /// 
    /// FAIL-CLOSED: Returns error if:
    /// - Clock rollback detected
    /// - Write fails
    /// - Hash computation fails
    /// - Signature generation fails
    pub fn log(
        &mut self,
        component: &str,
        event_type: &str,
        actor: &str,
        host: &str,
        data: serde_json::Value,
    ) -> Result<String, AuditError> {
        // Get timestamp with rollback protection (FAIL-CLOSED)
        let timestamp = self.clock.get_timestamp()
            .map_err(|e| AuditError::ClockRollback(e))?;
        
        // Get previous hash
        let previous_hash = self.chain.get_previous_hash()
            .unwrap_or_else(|| "GENESIS".to_string());
        
        // Create record (without hash and signature yet)
        let record_id = format!("audit_{}", uuid::Uuid::new_v4().to_string());
        
        let mut record = AuditRecord {
            record_id: record_id.clone(),
            timestamp,
            component: component.to_string(),
            event_type: event_type.to_string(),
            actor: actor.to_string(),
            host: host.to_string(),
            previous_hash: previous_hash.clone(),
            hash: String::new(), // Will be computed
            signature: String::new(), // Will be computed
            data,
        };
        
        // Serialize record (without hash/signature) for hashing
        let record_json = serde_json::to_string(&record)
            .map_err(|e| AuditError::SerializationError(format!("Failed to serialize record: {}", e)))?;
        
        // Compute hash
        let hash = self.chain.compute_hash(record_json.as_bytes(), Some(&previous_hash));
        record.hash = hash.clone();
        
        // Sign record (including hash)
        let record_with_hash_json = serde_json::to_string(&record)
            .map_err(|e| AuditError::SerializationError(format!("Failed to serialize record with hash: {}", e)))?;
        
        let signature = self.signer.sign(record_with_hash_json.as_bytes());
        record.signature = signature;
        
        // Write to log (append-only)
        let final_record_json = serde_json::to_string(&record)
            .map_err(|e| AuditError::SerializationError(format!("Failed to serialize final record: {}", e)))?;
        
        let writer = self.writer.as_mut()
            .ok_or_else(|| AuditError::WriteFailed("Writer not initialized".to_string()))?;
        
        writeln!(writer, "{}", final_record_json)
            .map_err(|e| AuditError::WriteFailed(format!("Failed to write audit record: {}", e)))?;
        
        writer.flush()
            .map_err(|e| AuditError::WriteFailed(format!("Failed to flush audit log: {}", e)))?;
        
        // Update chain
        self.chain.set_previous_hash(hash.clone());
        
        debug!("Audit record logged: {} (hash: {})", record_id, hash);
        
        Ok(record_id)
    }
    
    /// Get log path
    pub fn log_path(&self) -> &str {
        &self.log_path
    }
    
    /// Get verifying key (public key) for signature verification
    pub fn get_verifying_key_hex(&self) -> String {
        self.signer.get_verifying_key_hex()
    }
}

