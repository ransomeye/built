// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for network scanner - fail-closed error handling

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Scan timeout: {0}")]
    ScanTimeout(String),
    
    #[error("Invalid CIDR: {0}")]
    InvalidCidr(String),
    
    #[error("Port enumeration limit exceeded: {0}")]
    PortLimitExceeded(String),
    
    #[error("Unsigned scan result rejected: {0}")]
    UnsignedResult(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Replay attempt detected: {0}")]
    ReplayAttempt(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Correlation integration error: {0}")]
    CorrelationError(String),
    
    #[error("Playbook integration error: {0}")]
    PlaybookError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

