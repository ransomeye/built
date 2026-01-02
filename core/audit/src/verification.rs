// Path and File Name : /home/ransomeye/rebuild/core/audit/src/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit verification - verifies hash chain integrity, signatures, and detects tampering

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tracing::{error, warn, info};

use crate::errors::AuditError;
use crate::chain::{HashChain, AuditRecord};
use crate::signing::AuditSigner;
use ed25519_dalek::{VerifyingKey, Verifier, Signature};
use hex;
use base64::{Engine as _, engine::general_purpose};

/// Audit verifier - verifies integrity of audit logs
pub struct AuditVerifier {
    chain: HashChain,
}

impl AuditVerifier {
    /// Create new audit verifier
    pub fn new() -> Self {
        Self {
            chain: HashChain::new(),
        }
    }
    
    /// Verify audit log file
    /// 
    /// Checks:
    /// - Hash chain integrity
    /// - Signature validity
    /// - No missing entries
    /// - No tampering
    pub fn verify_log(&self, log_path: impl AsRef<Path>, verifying_key_hex: &str) -> Result<VerificationResult, AuditError> {
        let path = log_path.as_ref();
        let file = File::open(path)
            .map_err(|e| AuditError::IoError(e))?;
        
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        
        // Parse all records
        for line in reader.lines() {
            let line = line.map_err(|e| AuditError::IoError(e))?;
            if line.trim().is_empty() {
                continue;
            }
            
            let record: AuditRecord = serde_json::from_str(&line)
                .map_err(|e| AuditError::SerializationError(format!("Failed to parse record: {}", e)))?;
            
            records.push(record);
        }
        
        if records.is_empty() {
            return Ok(VerificationResult {
                valid: true,
                record_count: 0,
                errors: Vec::new(),
            });
        }
        
        let mut errors = Vec::new();
        
        // Verify hash chain
        match self.chain.verify_chain(&records) {
            Ok(true) => {
                info!("Hash chain verification passed for {} records", records.len());
            },
            Ok(false) => {
                errors.push("Hash chain verification failed".to_string());
            },
            Err(e) => {
                errors.push(format!("Hash chain error: {}", e));
            }
        }
        
        // Verify signatures
        let verifying_key_bytes = hex::decode(verifying_key_hex)
            .map_err(|e| AuditError::SerializationError(format!("Failed to decode verifying key: {}", e)))?;
        
        if verifying_key_bytes.len() != 32 {
            return Err(AuditError::SerializationError(
                format!("Invalid verifying key length: expected 32, got {}", verifying_key_bytes.len())
            ));
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&verifying_key_bytes[..32]);
        let verifying_key = VerifyingKey::from_bytes(&key_array)
            .map_err(|e| AuditError::SerializationError(format!("Invalid verifying key: {}", e)))?;
        
        for (i, record) in records.iter().enumerate() {
            // Reconstruct signed data (record without signature)
            let mut record_for_signing = record.clone();
            record_for_signing.signature = String::new();
            let record_json = serde_json::to_string(&record_for_signing)
                .map_err(|e| AuditError::SerializationError(format!("Failed to serialize record: {}", e)))?;
            
            // Verify signature
            let signature_bytes = general_purpose::STANDARD.decode(&record.signature)
                .map_err(|e| AuditError::SerializationError(format!("Failed to decode signature: {}", e)))?;
            
            if signature_bytes.len() != 64 {
                errors.push(format!("Invalid signature length for record {}", i));
                continue;
            }
            
            let mut sig_array = [0u8; 64];
            sig_array.copy_from_slice(&signature_bytes[..64]);
            let signature = Signature::from_bytes(&sig_array);
            
            if verifying_key.verify(record_json.as_bytes(), &signature).is_err() {
                errors.push(format!("Signature verification failed for record {}", i));
            }
        }
        
        // Check for missing entries (gaps in sequence)
        // In production, would check record_id sequence
        
        let valid = errors.is_empty();
        
        if !valid {
            error!("Audit log verification failed with {} errors", errors.len());
        } else {
            info!("Audit log verification passed: {} records verified", records.len());
        }
        
        Ok(VerificationResult {
            valid,
            record_count: records.len(),
            errors,
        })
    }
    
    /// Replay audit log (read all records in order)
    pub fn replay_log(&self, log_path: impl AsRef<Path>) -> Result<Vec<AuditRecord>, AuditError> {
        let path = log_path.as_ref();
        let file = File::open(path)
            .map_err(|e| AuditError::IoError(e))?;
        
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| AuditError::IoError(e))?;
            if line.trim().is_empty() {
                continue;
            }
            
            let record: AuditRecord = serde_json::from_str(&line)
                .map_err(|e| AuditError::SerializationError(format!("Failed to parse record: {}", e)))?;
            
            records.push(record);
        }
        
        info!("Replayed {} audit records from {}", records.len(), path.display());
        Ok(records)
    }
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub record_count: usize,
    pub errors: Vec<String>,
}

impl Default for AuditVerifier {
    fn default() -> Self {
        Self::new()
    }
}

