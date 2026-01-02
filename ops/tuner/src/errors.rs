// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for operations module - defines all error conditions with fail-closed semantics

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OperationsError {
    #[error("EULA not accepted")]
    EulaNotAccepted,
    
    #[error("Preflight check failed: {0}")]
    PreflightFailed(String),
    
    #[error("Install state invalid: {0}")]
    InvalidInstallState(String),
    
    #[error("Install state tampered: {0}")]
    InstallStateTampered(String),
    
    #[error("Retention configuration invalid: {0}")]
    InvalidRetention(String),
    
    #[error("Cryptographic identity generation failed: {0}")]
    IdentityGenerationFailed(String),
    
    #[error("Service operation failed: {0}")]
    ServiceOperationFailed(String),
    
    #[error("Uninstall verification failed: {0}")]
    UninstallVerificationFailed(String),
    
    #[error("Upgrade compatibility check failed: {0}")]
    UpgradeCompatibilityFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Systemd operation failed: {0}")]
    SystemdError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("State signature verification failed: {0}")]
    SignatureVerificationFailed(String),
}

