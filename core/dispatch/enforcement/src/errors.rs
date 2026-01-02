// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for enforcement dispatcher - fail-closed error handling

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnforcementError {
    #[error("Unsigned decision rejected: {0}")]
    UnsignedDecision(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Decision revoked: {0}")]
    DecisionRevoked(String),
    
    #[error("Missing required approval: {0}")]
    MissingApproval(String),
    
    #[error("Guardrail violation: {0}")]
    GuardrailViolation(String),
    
    #[error("Blast radius limit exceeded: {0}")]
    BlastRadiusExceeded(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Decision integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    
    #[error("Adapter failure: {0}")]
    AdapterFailure(String),
    
    #[error("Partial execution detected, rollback required: {0}")]
    PartialExecution(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Invalid decision format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

