// Path and File Name : /home/ransomeye/rebuild/core/forensics/src/store.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence store - content-addressed immutable storage

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{info, error, warn};

use crate::errors::ForensicsError;
use crate::integrity::EvidenceItem;

/// Evidence store - content-addressed immutable storage
pub struct EvidenceStore {
    store_path: PathBuf,
    index: HashMap<String, PathBuf>, // evidence_id -> file path
}

impl EvidenceStore {
    /// Create new evidence store
    pub fn new(store_path: impl AsRef<Path>) -> Result<Self, ForensicsError> {
        let path = store_path.as_ref();
        
        // Create store directory
        fs::create_dir_all(path)
            .map_err(|e| ForensicsError::IoError(e))?;
        
        // Load existing index
        let index = Self::load_index(path)?;
        
        Ok(Self {
            store_path: path.to_path_buf(),
            index,
        })
    }
    
    /// Load index from disk
    fn load_index(store_path: &Path) -> Result<HashMap<String, PathBuf>, ForensicsError> {
        let index_path = store_path.join("index.json");
        let mut index = HashMap::new();
        
        if index_path.exists() {
            let index_data = fs::read_to_string(&index_path)
                .map_err(|e| ForensicsError::IoError(e))?;
            
            let index_map: HashMap<String, String> = serde_json::from_str(&index_data)
                .map_err(|e| ForensicsError::SerializationError(format!("Failed to parse index: {}", e)))?;
            
            for (evidence_id, path_str) in index_map {
                index.insert(evidence_id, PathBuf::from(path_str));
            }
        }
        
        Ok(index)
    }
    
    /// Save index to disk
    fn save_index(&self) -> Result<(), ForensicsError> {
        let index_path = self.store_path.join("index.json");
        let mut index_map = HashMap::new();
        
        for (evidence_id, path) in &self.index {
            index_map.insert(evidence_id.clone(), path.to_string_lossy().to_string());
        }
        
        let index_json = serde_json::to_string_pretty(&index_map)
            .map_err(|e| ForensicsError::SerializationError(format!("Failed to serialize index: {}", e)))?;
        
        fs::write(&index_path, index_json)
            .map_err(|e| ForensicsError::IoError(e))?;
        
        Ok(())
    }
    
    /// Store evidence (immutable - content-addressed)
    /// 
    /// FAIL-CLOSED: Returns error if evidence already exists (immutability)
    pub fn store(&mut self, evidence: &EvidenceItem) -> Result<PathBuf, ForensicsError> {
        // Check if evidence already exists (immutability)
        if self.index.contains_key(&evidence.evidence_id) {
            return Err(ForensicsError::AlreadyExists(
                format!("Evidence {} already exists (immutable)", evidence.evidence_id)
            ));
        }
        
        // Store in content-addressed location (hash-based path)
        let hash_prefix = &evidence.content_hash[..2];
        let hash_suffix = &evidence.content_hash[2..];
        let evidence_dir = self.store_path.join(hash_prefix);
        let evidence_path = evidence_dir.join(format!("{}.json", hash_suffix));
        
        // Create directory if needed
        if let Some(parent) = evidence_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ForensicsError::IoError(e))?;
        }
        
        // Serialize evidence
        let evidence_json = serde_json::to_string_pretty(evidence)
            .map_err(|e| ForensicsError::SerializationError(format!("Failed to serialize evidence: {}", e)))?;
        
        // Write evidence (immutable - write once)
        fs::write(&evidence_path, evidence_json)
            .map_err(|e| ForensicsError::StorageFailed(format!("Failed to write evidence: {}", e)))?;
        
        // Update index
        self.index.insert(evidence.evidence_id.clone(), evidence_path.clone());
        self.save_index()?;
        
        info!("Stored evidence: {} at {}", evidence.evidence_id, evidence_path.display());
        
        Ok(evidence_path)
    }
    
    /// Retrieve evidence by ID
    pub fn retrieve(&self, evidence_id: &str) -> Result<EvidenceItem, ForensicsError> {
        let evidence_path = self.index.get(evidence_id)
            .ok_or_else(|| ForensicsError::NotFound(format!("Evidence {} not found", evidence_id)))?;
        
        let evidence_data = fs::read_to_string(evidence_path)
            .map_err(|e| ForensicsError::IoError(e))?;
        
        let evidence: EvidenceItem = serde_json::from_str(&evidence_data)
            .map_err(|e| ForensicsError::SerializationError(format!("Failed to parse evidence: {}", e)))?;
        
        Ok(evidence)
    }
    
    /// List all evidence IDs
    pub fn list(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }
}

