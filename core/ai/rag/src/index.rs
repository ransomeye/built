// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/rag/src/index.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Read-only RAG index with integrity verification

use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use tracing::{error, warn, info, debug};

use super::errors::RAGError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_version: String,
    pub index_hash: String,
    pub document_count: usize,
    pub created_at: String,
    pub integrity_hash: String,
}

#[derive(Debug, Clone)]
pub struct IndexDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f64>,
    pub metadata: serde_json::Value,
}

pub struct RAGIndex {
    index_path: PathBuf,
    metadata: IndexMetadata,
    documents: Arc<RwLock<Vec<IndexDocument>>>,
    loaded: bool,
}

impl RAGIndex {
    pub fn new(index_path: PathBuf) -> Result<Self, RAGError> {
        info!("Initializing RAG index from: {:?}", index_path);
        
        // Load metadata
        let metadata_path = index_path.join("metadata.json");
        if !metadata_path.exists() {
            return Err(RAGError::IndexNotFound(
                format!("Metadata file not found: {:?}", metadata_path)
            ));
        }
        
        let metadata_json = fs::read_to_string(&metadata_path)
            .map_err(|e| RAGError::IndexLoadFailed(
                format!("Failed to read metadata: {}", e)
            ))?;
        
        let metadata: IndexMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| RAGError::MetadataInvalid(
                format!("Failed to parse metadata: {}", e)
            ))?;
        
        Ok(Self {
            index_path,
            metadata,
            documents: Arc::new(RwLock::new(Vec::new())),
            loaded: false,
        })
    }
    
    /// Load index (read-only)
    pub fn load(&mut self) -> Result<(), RAGError> {
        if self.loaded {
            debug!("Index already loaded");
            return Ok(());
        }
        
        info!("Loading RAG index");
        
        // Load index file
        let index_file = self.index_path.join("index.bin");
        if !index_file.exists() {
            return Err(RAGError::IndexNotFound(
                format!("Index file not found: {:?}", index_file)
            ));
        }
        
        // Verify integrity
        self.verify_integrity(&index_file)?;
        
        // Load documents (simplified - in production would use actual vector DB format)
        // For now, create placeholder documents
        let mut documents = Vec::new();
        for i in 0..self.metadata.document_count {
            documents.push(IndexDocument {
                id: format!("doc_{}", i),
                content: format!("Document {} content", i),
                embedding: vec![0.0; 384], // Placeholder embedding
                metadata: serde_json::json!({"index": i}),
            });
        }
        
        {
            let mut docs = self.documents.write();
            *docs = documents;
        }
        
        self.loaded = true;
        info!("RAG index loaded: {} documents", self.metadata.document_count);
        Ok(())
    }
    
    /// Verify index integrity
    fn verify_integrity(&self, index_file: &Path) -> Result<(), RAGError> {
        // Compute hash of index file
        let index_data = fs::read(index_file)
            .map_err(|e| RAGError::IndexIntegrityFailed(
                format!("Failed to read index file: {}", e)
            ))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&index_data);
        let computed_hash = hex::encode(hasher.finalize());
        
        // Verify against metadata hash
        if computed_hash != self.metadata.index_hash {
            return Err(RAGError::IndexIntegrityFailed(
                format!("Index hash mismatch: expected {}, got {}", 
                    self.metadata.index_hash, computed_hash)
            ));
        }
        
        debug!("Index integrity verified: hash={}", computed_hash);
        Ok(())
    }
    
    /// Get document count
    pub fn document_count(&self) -> usize {
        self.metadata.document_count
    }
    
    /// Check if index is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
    
    /// Get documents (read-only access)
    pub fn get_documents(&self) -> Vec<IndexDocument> {
        self.documents.read().clone()
    }
}

