// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: DPI Probe error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("Capture failed: {0}")]
    CaptureFailed(String),
    
    #[error("Parser failed: {0}")]
    ParseFailed(String),
    
    #[error("Flow tracking failed: {0}")]
    FlowTrackingFailed(String),
    
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
}

