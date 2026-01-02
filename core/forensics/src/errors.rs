// Path and File Name : /home/ransomeye/rebuild/core/forensics/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for forensics

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForensicsError {
    #[error("Evidence storage failed: {0}")]
    StorageFailed(String),
    #[error("Evidence integrity check failed: {0}")]
    IntegrityFailed(String),
    #[error("Evidence not found: {0}")]
    NotFound(String),
    #[error("Evidence already exists: {0}")]
    AlreadyExists(String),
    #[error("Evidence tampering detected: {0}")]
    TamperingDetected(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

