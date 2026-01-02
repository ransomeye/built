// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/rag/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: RAG error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RAGError {
    #[error("Index load failed: {0}")]
    IndexLoadFailed(String),
    
    #[error("Index integrity check failed: {0}")]
    IndexIntegrityFailed(String),
    
    #[error("Retrieval failed: {0}")]
    RetrievalFailed(String),
    
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Metadata invalid: {0}")]
    MetadataInvalid(String),
}

