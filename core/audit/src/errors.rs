// Path and File Name : /home/ransomeye/rebuild/core/audit/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for audit logging

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Audit log write failed: {0}")]
    WriteFailed(String),
    #[error("Hash chain verification failed: {0}")]
    HashChainFailed(String),
    #[error("Signature verification failed: {0}")]
    SignatureFailed(String),
    #[error("Clock rollback detected: {0}")]
    ClockRollback(String),
    #[error("Missing log entry: {0}")]
    MissingEntry(String),
    #[error("Log tampering detected: {0}")]
    TamperingDetected(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

