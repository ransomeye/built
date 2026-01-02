// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Correlation engine error types

use thiserror::Error;

/// Correlation engine errors
#[derive(Debug, Error)]
pub enum CorrelationError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("State corruption detected: {0}")]
    StateCorruption(String),

    #[error("Event ordering violation: {0}")]
    OrderingViolation(String),

    #[error("Entity state not found: {0}")]
    EntityNotFound(String),

    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),

    #[error("Invalid kill-chain transition: {0}")]
    InvalidTransition(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for correlation operations
pub type CorrelationResult<T> = Result<T, CorrelationError>;

