// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/security/integrity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Hash and manifest integrity verification

use std::path::Path;
use std::fs;
use sha2::{Sha256, Digest};
use hex;
use tracing::debug;

pub struct IntegrityChecker;

impl IntegrityChecker {
    pub fn new() -> Self {
        Self
    }
    
    /// Compute SHA-256 hash of file
    pub fn compute_hash(&self, file_path: &Path) -> Result<String, String> {
        let data = fs::read(file_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", file_path, e))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());
        
        debug!("Computed hash for {:?}: {}", file_path, hash);
        Ok(hash)
    }
    
    /// Verify file hash matches expected
    pub fn verify_hash(&self, file_path: &Path, expected_hash: &str) -> Result<bool, String> {
        let computed_hash = self.compute_hash(file_path)?;
        
        if computed_hash == expected_hash {
            debug!("Hash verification successful for {:?}", file_path);
            Ok(true)
        } else {
            Err(format!("Hash mismatch: expected {}, got {}", expected_hash, computed_hash))
        }
    }
    
    /// Verify manifest integrity
    pub fn verify_manifest(&self, manifest_path: &Path, expected_hash: &str) -> Result<bool, String> {
        self.verify_hash(manifest_path, expected_hash)
    }
}

