// Path and File Name : /home/ransomeye/rebuild/core/deception/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for Phase 16 - Deception Framework

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeceptionError {
    #[error("Asset signature verification failed: {0}")]
    InvalidSignature(String),
    
    #[error("Asset schema validation failed: {0}")]
    SchemaValidationFailed(String),
    
    #[error("Asset overlaps with real production service: {0}")]
    OverlapsProduction(String),
    
    #[error("Forbidden asset type: {0}")]
    ForbiddenAssetType(String),
    
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),
    
    #[error("Teardown failed: {0}")]
    TeardownFailed(String),
    
    #[error("Signal generation failed: {0}")]
    SignalGenerationFailed(String),
    
    #[error("Signal signature missing or invalid: {0}")]
    SignalSignatureInvalid(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    
    #[error("Asset lifetime exceeded: {0}")]
    LifetimeExceeded(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Safe-halt state entered due to: {0}")]
    SafeHalt(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Network error: {0}")]
    Network(String),
}

