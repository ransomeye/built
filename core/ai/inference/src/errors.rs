// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Inference error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Model loading failed: {0}")]
    ModelLoadFailed(String),
    
    #[error("Model signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Model integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    
    #[error("Model manifest missing or invalid: {0}")]
    ManifestInvalid(String),
    
    #[error("Feature extraction failed: {0}")]
    FeatureExtractionFailed(String),
    
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    
    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),
    
    #[error("Threshold validation failed: {0}")]
    ThresholdValidationFailed(String),
    
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

