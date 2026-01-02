// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for reporting module - defines all error conditions with fail-closed semantics

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportingError {
    #[error("Evidence corruption detected: {0}")]
    EvidenceCorrupted(String),
    
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    
    #[error("Missing evidence: {0}")]
    MissingEvidence(String),
    
    #[error("Evidence bundle already sealed: {0}")]
    BundleSealed(String),
    
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    #[error("Export format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Report generation failed: {0}")]
    ReportGenerationFailed(String),
    
    #[error("Retention policy violation: {0}")]
    RetentionViolation(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Hash chain broken: {0}")]
    HashChainBroken(String),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Evidence store locked: {0}")]
    StoreLocked(String),
}

