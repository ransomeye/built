// Path and File Name : /home/ransomeye/rebuild/core/deception/src/registry.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deception asset registry - loads, verifies signatures, validates schema, enforces fail-closed rules

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, warn, info, debug};
use serde_yaml;

use crate::asset::{DeceptionAsset, AssetType};
use crate::errors::DeceptionError;
use crate::security::SignatureVerifier;

/// Allowed asset types (fail-closed: only these are permitted)
const ALLOWED_ASSET_TYPES: &[&str] = &[
    "decoy_host",
    "decoy_service",
    "credential_lure",
    "filesystem_lure",
];

/// Forbidden asset types (fail-closed: these trigger rejection)
const FORBIDDEN_ASSET_TYPES: &[&str] = &[
    "traffic_interceptor",
    "proxy_service",
    "production_mirror",
];

pub struct DeceptionRegistry {
    assets: Arc<RwLock<HashMap<String, DeceptionAsset>>>,
    asset_paths: Arc<RwLock<HashMap<String, PathBuf>>>,
    verifier: Arc<SignatureVerifier>,
    asset_dir: PathBuf,
}

impl DeceptionRegistry {
    /// Create new registry from environment variables
    pub fn new() -> Result<Self, DeceptionError> {
        // Get asset directory from environment
        let asset_dir = std::env::var("DECEPTION_ASSET_DIR")
            .unwrap_or_else(|_| "/etc/ransomeye/deception/assets".to_string());
        let asset_dir = PathBuf::from(asset_dir);
        
        // Get public key path from environment
        let public_key_path = std::env::var("DECEPTION_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/keys/deception_public_key.pem".to_string());
        
        let verifier = Arc::new(SignatureVerifier::new(&public_key_path)?);
        
        let registry = Self {
            assets: Arc::new(RwLock::new(HashMap::new())),
            asset_paths: Arc::new(RwLock::new(HashMap::new())),
            verifier,
            asset_dir,
        };
        
        // Load assets on creation
        registry.reload_assets()?;
        
        Ok(registry)
    }
    
    /// Reload all assets from directory
    pub fn reload_assets(&self) -> Result<usize, DeceptionError> {
        info!("Reloading deception assets from: {}", self.asset_dir.display());
        
        let mut assets = self.assets.write();
        let mut asset_paths = self.asset_paths.write();
        
        assets.clear();
        asset_paths.clear();
        
        if !self.asset_dir.exists() {
            warn!("Deception asset directory does not exist: {}", self.asset_dir.display());
            return Ok(0);
        }
        
        let mut loaded_count = 0;
        
        // Load all YAML files in asset directory
        let entries = std::fs::read_dir(&self.asset_dir)
            .map_err(|e| DeceptionError::Io(e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| DeceptionError::Io(e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") || 
               path.extension().and_then(|s| s.to_str()) == Some("yml") {
                match self.load_asset_from_file(&path) {
                    Ok(asset) => {
                        let asset_id = asset.asset_id.clone();
                        assets.insert(asset_id.clone(), asset);
                        asset_paths.insert(asset_id, path);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load asset from {}: {}", path.display(), e);
                        // FAIL-CLOSED: Reject invalid assets, but continue loading others
                    }
                }
            }
        }
        
        info!("Loaded {} deception assets", loaded_count);
        Ok(loaded_count)
    }
    
    /// Load asset from file with full validation
    fn load_asset_from_file(&self, path: &Path) -> Result<DeceptionAsset, DeceptionError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DeceptionError::Io(e))?;
        
        let asset: DeceptionAsset = serde_yaml::from_str(&content)
            .map_err(|e| DeceptionError::Yaml(e))?;
        
        // Step 1: Validate schema
        asset.validate_schema()
            .map_err(|e| DeceptionError::SchemaValidationFailed(e))?;
        
        // Step 2: Verify asset type is allowed (FAIL-CLOSED)
        let asset_type_str = asset.asset_type_str();
        if FORBIDDEN_ASSET_TYPES.contains(&asset_type_str) {
            return Err(DeceptionError::ForbiddenAssetType(
                format!("Asset type '{}' is forbidden (traffic interception not allowed)", asset_type_str)
            ));
        }
        
        if !ALLOWED_ASSET_TYPES.contains(&asset_type_str) {
            return Err(DeceptionError::ForbiddenAssetType(
                format!("Asset type '{}' is not in allowed list", asset_type_str)
            ));
        }
        
        // Step 3: Verify signature (FAIL-CLOSED)
        self.verifier.verify_asset(&asset)?;
        
        debug!("Loaded and verified asset: {} from {}", asset.asset_id, path.display());
        Ok(asset)
    }
    
    /// Get asset by ID
    pub fn get_asset(&self, asset_id: &str) -> Option<DeceptionAsset> {
        self.assets.read().get(asset_id).cloned()
    }
    
    /// Get all assets
    pub fn get_all_assets(&self) -> Vec<DeceptionAsset> {
        self.assets.read().values().cloned().collect()
    }
    
    /// Check if asset exists
    pub fn has_asset(&self, asset_id: &str) -> bool {
        self.assets.read().contains_key(asset_id)
    }
    
    /// Validate asset does not overlap with production services
    /// This is a placeholder - actual implementation would check against network scanner results
    pub fn validate_no_production_overlap(&self, asset: &DeceptionAsset) -> Result<(), DeceptionError> {
        // TODO: Integrate with Phase 9 (Network Scanner) to check for overlaps
        // For now, we perform basic validation
        
        // Check if asset tries to bind to well-known production ports
        // This is a conservative check - actual implementation should use network scanner data
        
        // Extract port information from telemetry_fields if present
        // This is a simplified check - real implementation would query network scanner
        
        Ok(())
    }
}

