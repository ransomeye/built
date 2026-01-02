// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/loader.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model loading with RSA-4096 signature verification and integrity checks

use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, warn, info, debug};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use base64::{Engine as _, engine::general_purpose};

use super::errors::InferenceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub model_name: String,
    pub model_version: String,
    pub model_hash: String,
    pub model_size_bytes: u64,
    pub signature: String,
    pub trained_on: String,
    pub model_type: String,
    pub features: Vec<String>,
}

pub struct ModelLoader {
    models_dir: PathBuf,
    public_key_bytes: Vec<u8>,
    loaded_models: Arc<RwLock<std::collections::HashMap<String, LoadedModel>>>,
}

#[derive(Debug, Clone)]
pub struct LoadedModel {
    pub model_name: String,
    pub model_version: String,
    pub model_data: Vec<u8>,
    pub manifest: ModelManifest,
}

impl ModelLoader {
    pub fn new(models_dir: PathBuf, public_key_path: PathBuf) -> Result<Self, InferenceError> {
        // Load public key for signature verification
        let public_key_bytes = fs::read(&public_key_path)
            .map_err(|e| InferenceError::SignatureVerificationFailed(
                format!("Failed to read public key from {:?}: {}", public_key_path, e)
            ))?;
        
        info!("ModelLoader initialized with models_dir: {:?}, public_key: {:?}", models_dir, public_key_path);
        
        Ok(Self {
            models_dir,
            public_key_bytes,
            loaded_models: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    /// Load model with full verification (signature + integrity)
    pub fn load_model(&self, model_name: &str) -> Result<LoadedModel, InferenceError> {
        // Check if already loaded
        {
            let loaded = self.loaded_models.read();
            if let Some(model) = loaded.get(model_name) {
                debug!("Model {} already loaded, returning cached", model_name);
                return Ok(model.clone());
            }
        }
        
        info!("Loading model: {}", model_name);
        
        // Load manifest
        let manifest_path = self.models_dir.join("models.manifest.json");
        let manifest: ModelManifest = self.load_manifest(&manifest_path)?;
        
        // Verify manifest signature
        self.verify_manifest_signature(&manifest)?;
        
        // Load model file
        let model_path = self.models_dir.join(format!("{}.model", model_name));
        if !model_path.exists() {
            return Err(InferenceError::ModelNotFound(
                format!("Model file not found: {:?}", model_path)
            ));
        }
        
        let model_data = fs::read(&model_path)
            .map_err(|e| InferenceError::ModelLoadFailed(
                format!("Failed to read model file {:?}: {}", model_path, e)
            ))?;
        
        // Verify model integrity (hash check)
        self.verify_model_integrity(&model_data, &manifest)?;
        
        // Check memory limit (3GB for core AI)
        if model_data.len() > 3_000_000_000 {
            return Err(InferenceError::MemoryLimitExceeded(
                format!("Model {} exceeds 3GB limit: {} bytes", model_name, model_data.len())
            ));
        }
        
        let loaded_model = LoadedModel {
            model_name: model_name.to_string(),
            model_version: manifest.model_version.clone(),
            model_data,
            manifest: manifest.clone(),
        };
        
        // Cache loaded model
        {
            let mut loaded = self.loaded_models.write();
            loaded.insert(model_name.to_string(), loaded_model.clone());
        }
        
        info!("Model {} loaded successfully ({} bytes)", model_name, loaded_model.model_data.len());
        Ok(loaded_model)
    }
    
    fn load_manifest(&self, manifest_path: &Path) -> Result<ModelManifest, InferenceError> {
        if !manifest_path.exists() {
            return Err(InferenceError::ManifestInvalid(
                format!("Manifest file not found: {:?}", manifest_path)
            ));
        }
        
        let manifest_json = fs::read_to_string(manifest_path)
            .map_err(|e| InferenceError::ManifestInvalid(
                format!("Failed to read manifest {:?}: {}", manifest_path, e)
            ))?;
        
        let manifest: ModelManifest = serde_json::from_str(&manifest_json)
            .map_err(|e| InferenceError::ManifestInvalid(
                format!("Failed to parse manifest: {}", e)
            ))?;
        
        Ok(manifest)
    }
    
    fn verify_manifest_signature(&self, manifest: &ModelManifest) -> Result<(), InferenceError> {
        // Create manifest JSON without signature for verification
        let mut manifest_for_verify = serde_json::to_value(manifest)
            .map_err(|e| InferenceError::SignatureVerificationFailed(
                format!("Failed to serialize manifest: {}", e)
            ))?;
        
        if let Some(obj) = manifest_for_verify.as_object_mut() {
            obj.remove("signature");
        }
        
        let manifest_bytes = serde_json::to_vec(&manifest_for_verify)
            .map_err(|e| InferenceError::SignatureVerificationFailed(
                format!("Failed to serialize manifest for verification: {}", e)
            ))?;
        
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(&manifest_bytes);
        let hash = hasher.finalize();
        
        // Verify signature
        let signature_bytes = general_purpose::STANDARD.decode(&manifest.signature)
            .map_err(|e| InferenceError::SignatureVerificationFailed(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        let public_key = UnparsedPublicKey::new(
            &RSA_PSS_2048_8192_SHA256,
            &self.public_key_bytes,
        );
        
        public_key.verify(&hash, &signature_bytes)
            .map_err(|e| InferenceError::SignatureVerificationFailed(
                format!("RSA-4096 signature verification failed: {:?}", e)
            ))?;
        
        debug!("Manifest signature verified successfully");
        Ok(())
    }
    
    fn verify_model_integrity(&self, model_data: &[u8], manifest: &ModelManifest) -> Result<(), InferenceError> {
        // Compute model hash
        let mut hasher = Sha256::new();
        hasher.update(model_data);
        let computed_hash = hex::encode(hasher.finalize());
        
        if computed_hash != manifest.model_hash {
            return Err(InferenceError::IntegrityCheckFailed(
                format!("Model hash mismatch: expected {}, got {}", manifest.model_hash, computed_hash)
            ));
        }
        
        // Verify size
        if model_data.len() as u64 != manifest.model_size_bytes {
            return Err(InferenceError::IntegrityCheckFailed(
                format!("Model size mismatch: expected {} bytes, got {}", manifest.model_size_bytes, model_data.len())
            ));
        }
        
        debug!("Model integrity verified: hash={}, size={}", computed_hash, model_data.len());
        Ok(())
    }
    
    /// Check if model is loaded
    pub fn is_loaded(&self, model_name: &str) -> bool {
        self.loaded_models.read().contains_key(model_name)
    }
    
    /// Get loaded model count
    pub fn loaded_count(&self) -> usize {
        self.loaded_models.read().len()
    }
}

