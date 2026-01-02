// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Windows Agent error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Process monitoring failed: {0}")]
    ProcessMonitoringFailed(String),
    
    #[error("Filesystem monitoring failed: {0}")]
    FilesystemMonitoringFailed(String),
    
    #[error("Registry monitoring failed: {0}")]
    RegistryMonitoringFailed(String),
    
    #[error("Network monitoring failed: {0}")]
    NetworkMonitoringFailed(String),
    
    #[error("ETW initialization failed: {0}")]
    EtwInitializationFailed(String),
    
    #[error("ETW event processing failed: {0}")]
    EtwEventProcessingFailed(String),
    
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
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("WMI fallback failed: {0}")]
    WmiFallbackFailed(String),
    
    #[error("Component identity failure: {0}")]
    ComponentIdentityFailure(String),
}

