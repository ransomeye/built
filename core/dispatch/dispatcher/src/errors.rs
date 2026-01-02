// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Dispatcher error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatcherError {
    #[error("Invalid directive: {0}")]
    InvalidDirective(String),
    
    #[error("Directive expired")]
    DirectiveExpired,
    
    #[error("Replay detected: {0}")]
    ReplayDetected(String),
    
    #[error("Nonce replay: {0}")]
    NonceReplay(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Audit receipt invalid: {0}")]
    AuditReceiptInvalid(String),
    
    #[error("Target resolution failed: {0}")]
    TargetResolutionFailed(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent capability mismatch: {0}")]
    AgentCapabilityMismatch(String),
    
    #[error("Platform mismatch: {0}")]
    PlatformMismatch(String),
    
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    
    #[error("Invalid acknowledgment: {0}")]
    InvalidAcknowledgment(String),
    
    #[error("Acknowledgment timeout: {0}")]
    AcknowledgmentTimeout(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Reentrancy detected")]
    ReentrancyDetected,
    
    #[error("Loop detected")]
    LoopDetected,
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}
