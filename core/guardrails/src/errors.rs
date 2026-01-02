// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for guardrails enforcement engine

use thiserror::Error;

pub type GuardrailResult<T> = Result<T, GuardrailError>;

#[derive(Error, Debug)]
pub enum GuardrailError {
    #[error("Guardrail specification not found: {0}")]
    SpecNotFound(String),
    
    #[error("Guardrail specification signature invalid: {0}")]
    InvalidSignature(String),
    
    #[error("Guardrail specification schema validation failed: {0}")]
    SchemaValidationFailed(String),
    
    #[error("Guardrail specification unsigned or tampered")]
    UnsignedSpec,
    
    #[error("Phantom module detected: {0}")]
    PhantomModule(String),
    
    #[error("Forbidden module detected: {0}")]
    ForbiddenModule(String),
    
    #[error("Hardcoded configuration detected in {0}: {1}")]
    HardcodedConfig(String, String),
    
    #[error("Systemd service file misplaced: {0} (must be in /home/ransomeye/rebuild/systemd/)")]
    SystemdMisplacement(String),
    
    #[error("Unsigned artifact detected: {0}")]
    UnsignedArtifact(String),
    
    #[error("Missing required ENV variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Model missing SHAP explainability: {0}")]
    MissingShap(String),
    
    #[error("Model missing metadata: {0}")]
    MissingModelMetadata(String),
    
    #[error("File missing mandatory header: {0}")]
    MissingHeader(String),
    
    #[error("Requirements.txt in forbidden location: {0}")]
    ForbiddenRequirementsTxt(String),
    
    #[error("Service can start without guardrails - integration missing")]
    ServiceBypassPossible,
    
    #[error("CI can pass with violation - validation missing")]
    CIBypassPossible,
    
    #[error("Guardrail enforcement cannot be cryptographically enforced")]
    CryptoEnforcementFailed,
    
    #[error("Violation can be bypassed: {0}")]
    BypassPossible(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    #[error("Audit logging failed: {0}")]
    AuditFailed(String),
}

