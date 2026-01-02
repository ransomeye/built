// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux Agent error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Process monitoring failed: {0}")]
    ProcessMonitoringFailed(String),
    
    #[error("Filesystem monitoring failed: {0}")]
    FilesystemMonitoringFailed(String),
    
    #[error("Network monitoring failed: {0}")]
    NetworkMonitoringFailed(String),
    
    #[error("Syscall monitoring failed: {0}")]
    SyscallMonitoringFailed(String),
    
    #[error("Feature extraction failed: {0}")]
    FeatureExtractionFailed(String),
    
    #[error("Envelope creation failed: {0}")]
    EnvelopeCreationFailed(String),
    
    #[error("Backpressure limit exceeded: {0}")]
    BackpressureExceeded(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    
    #[error("Identity verification failed: {0}")]
    IdentityVerificationFailed(String),
    
    #[error("Transport initialization failed: {0}")]
    TransportInitFailed(String),
    
    #[error("Identity initialization failed: {0}")]
    IdentityInitFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("eBPF error: {0}")]
    EbpfError(String),
    
    #[error("Audit error: {0}")]
    AuditError(String),
}

