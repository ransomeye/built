// Path and File Name : /home/ransomeye/rebuild/qa/validation/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for Phase 10 validation

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Contract integrity violation: {0}")]
    ContractIntegrity(String),
    
    #[error("Cryptographic continuity violation: {0}")]
    CryptographicContinuity(String),
    
    #[error("Determinism violation: {0}")]
    Determinism(String),
    
    #[error("Failure isolation violation: {0}")]
    FailureIsolation(String),
    
    #[error("Resource ceiling violation: {0}")]
    ResourceCeiling(String),
    
    #[error("Advisory boundary violation: {0}")]
    AdvisoryBoundary(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

