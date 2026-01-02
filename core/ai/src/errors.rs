// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for AI advisory system - fail-closed error handling

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdvisoryError {
    #[error("AI subsystem disabled: {0}")]
    AIDisabled(String),
    
    #[error("Missing baseline model: {0}")]
    MissingBaseline(String),
    
    #[error("Unsigned model rejected: {0}")]
    UnsignedModel(String),
    
    #[error("Missing SHAP explanation: {0}")]
    MissingSHAP(String),
    
    #[error("SHAP validation failed: {0}")]
    SHAPValidationFailed(String),
    
    #[error("Model signature invalid: {0}")]
    InvalidModelSignature(String),
    
    #[error("Model revoked: {0}")]
    ModelRevoked(String),
    
    #[error("Model integrity check failed: {0}")]
    ModelIntegrityFailed(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

