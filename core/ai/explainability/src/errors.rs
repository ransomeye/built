// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/explainability/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Explainability error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExplainabilityError {
    #[error("SHAP computation failed: {0}")]
    SHAPComputationFailed(String),
    
    #[error("SHAP validation failed: {0}")]
    SHAPValidationFailed(String),
    
    #[error("Rationale generation failed: {0}")]
    RationaleGenerationFailed(String),
    
    #[error("Feature mismatch: {0}")]
    FeatureMismatch(String),
    
    #[error("Baseline missing: {0}")]
    BaselineMissing(String),
}

