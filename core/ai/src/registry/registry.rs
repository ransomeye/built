// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/registry/registry.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model registry - manages signed baseline models

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{error, warn, debug};
use crate::errors::AdvisoryError;
use crate::registry::verification::ModelVerifier as RegistryModelVerifier;
use crate::registry::rollback::ModelRollback;

#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub version: String,
    pub path: String,
    pub signature: String,
    pub signature_hash: String,
    pub model_hash: String,
    pub signed: bool,
    pub baseline: bool,
}

impl Model {
    pub fn is_signed(&self) -> bool {
        self.signed
    }
    
    pub fn is_baseline(&self) -> bool {
        self.baseline
    }
}

pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, Model>>>,
    verifier: Arc<RegistryModelVerifier>,
    rollback: Arc<ModelRollback>,
    models_dir: String,
}

impl ModelRegistry {
    pub fn new() -> Result<Self, AdvisoryError> {
        let models_dir = std::env::var("RANSOMEYE_AI_MODELS_DIR")
            .unwrap_or_else(|_| "/etc/ransomeye/ai/models".to_string());
        
        let verifier = Arc::new(RegistryModelVerifier::new()?);
        let rollback = Arc::new(ModelRollback::new());
        
        let registry = Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            verifier,
            rollback,
            models_dir: models_dir.clone(),
        };
        
        // Load baseline models
        registry.load_baseline_models()?;
        
        Ok(registry)
    }
    
    /// Load baseline models
    fn load_baseline_models(&self) -> Result<(), AdvisoryError> {
        debug!("Loading baseline models from {}", self.models_dir);
        
        let models_path = Path::new(&self.models_dir);
        if !models_path.exists() {
            return Err(AdvisoryError::MissingBaseline(
                format!("Models directory does not exist: {}", self.models_dir)
            ));
        }
        
        // Look for baseline models
        let baseline_models = vec!["risk_model", "anomaly_model", "behavior_model"];
        
        for model_name in baseline_models {
            let model_path = models_path.join(format!("{}.pkl", model_name));
            if model_path.exists() {
                match self.load_model_from_path(&model_path, model_name, true) {
                    Ok(_) => {
                        debug!("Loaded baseline model: {}", model_name);
                    }
                    Err(e) => {
                        warn!("Failed to load baseline model {}: {}", model_name, e);
                    }
                }
            } else {
                warn!("Baseline model not found: {}", model_path.display());
            }
        }
        
        Ok(())
    }
    
    /// Load model from path
    fn load_model_from_path(&self, path: &Path, name: &str, baseline: bool) -> Result<(), AdvisoryError> {
        // Read model metadata
        let metadata_path = path.with_extension("metadata.json");
        if !metadata_path.exists() {
            return Err(AdvisoryError::MissingBaseline(
                format!("Model metadata not found: {}", metadata_path.display())
            ));
        }
        
        let metadata_content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Failed to read model metadata: {}", e)
            ))?;
        
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content)
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Invalid model metadata JSON: {}", e)
            ))?;
        
        let version = metadata.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0");
        
        let signature = metadata.get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let signature_hash = metadata.get("signature_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Verify model signature
        let signed = if !signature.is_empty() {
            self.verifier.verify_model_signature(path, signature).unwrap_or(false)
        } else {
            false
        };
        
        if baseline && !signed {
            return Err(AdvisoryError::UnsignedModel(
                format!("Baseline model {} is not signed", name)
            ));
        }
        
        // Compute model hash
        let model_hash = self.compute_model_hash(path)?;
        
        let model = Model {
            name: name.to_string(),
            version: version.to_string(),
            path: path.to_string_lossy().to_string(),
            signature: signature.to_string(),
            signature_hash: signature_hash.to_string(),
            model_hash,
            signed,
            baseline,
        };
        
        let mut models = self.models.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        models.insert(name.to_string(), model);
        
        Ok(())
    }
    
    /// Load model by name
    pub fn load_model(&self, name: &str) -> Result<Model, AdvisoryError> {
        let models = self.models.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        models.get(name)
            .cloned()
            .ok_or_else(|| AdvisoryError::MissingBaseline(
                format!("Model not found: {}", name)
            ))
    }
    
    /// Check if baseline models are present
    pub fn has_baseline_models(&self) -> Result<bool, AdvisoryError> {
        let models = self.models.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let baseline_models = vec!["risk_model", "anomaly_model", "behavior_model"];
        let mut has_all = true;
        
        for model_name in baseline_models {
            if let Some(model) = models.get(model_name) {
                if !model.is_signed() {
                    warn!("Baseline model {} is not signed", model_name);
                    has_all = false;
                }
            } else {
                warn!("Baseline model {} not found", model_name);
                has_all = false;
            }
        }
        
        Ok(has_all)
    }
    
    fn compute_model_hash(&self, path: &Path) -> Result<String, AdvisoryError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let model_bytes = std::fs::read(path)
            .map_err(|e| AdvisoryError::ConfigurationError(
                format!("Failed to read model file: {}", e)
            ))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&model_bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}

