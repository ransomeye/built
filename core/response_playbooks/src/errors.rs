// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for playbook engine - fail-closed error handling

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlaybookError {
    #[error("Unsigned playbook rejected: {0}")]
    UnsignedPlaybook(String),
    
    #[error("Invalid playbook signature: {0}")]
    InvalidSignature(String),
    
    #[error("Playbook schema validation failed: {0}")]
    SchemaValidationFailed(String),
    
    #[error("Playbook not found: {0}")]
    PlaybookNotFound(String),
    
    #[error("Playbook execution already in progress: {0}")]
    ExecutionInProgress(String),
    
    #[error("Replay attempt detected: {0}")]
    ReplayAttempt(String),
    
    #[error("Playbook execution timeout: {0}")]
    ExecutionTimeout(String),
    
    #[error("Step execution failed: {0}")]
    StepExecutionFailed(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Rollback state not found: {0}")]
    RollbackStateNotFound(String),
    
    #[error("Policy binding not found: {0}")]
    PolicyBindingNotFound(String),
    
    #[error("Missing step handler: {0}")]
    MissingStepHandler(String),
    
    #[error("Invalid execution state: {0}")]
    InvalidExecutionState(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

