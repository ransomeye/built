// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/errors/dispatcher_errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for dispatcher - fail-closed error handling

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatcherError {
    #[error("Invalid directive: {0}")]
    InvalidDirective(String),
    
    #[error("Directive signature verification failed: {0}")]
    InvalidSignature(String),
    
    #[error("Directive expired (TTL exceeded)")]
    DirectiveExpired,
    
    #[error("Replay attack detected: {0}")]
    ReplayDetected(String),
    
    #[error("Nonce already used: {0}")]
    NonceReplay(String),
    
    #[error("Trust chain verification failed: {0}")]
    TrustChainFailure(String),
    
    #[error("Audit receipt verification failed: {0}")]
    AuditReceiptInvalid(String),
    
    #[error("Target resolution failed: {0}")]
    TargetResolutionFailed(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent capability mismatch: {0}")]
    AgentCapabilityMismatch(String),
    
    #[error("Platform compatibility error: {0}")]
    PlatformMismatch(String),
    
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    
    #[error("Acknowledgment timeout: {0}")]
    AcknowledgmentTimeout(String),
    
    #[error("Invalid acknowledgment: {0}")]
    InvalidAcknowledgment(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Reentrancy detected: dispatcher triggered itself")]
    ReentrancyDetected,
    
    #[error("Loop detected: circular execution")]
    LoopDetected,
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Precondition check failed: {0}")]
    PreconditionFailed(String),
    
    #[error("Version incompatibility: {0}")]
    VersionMismatch(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

