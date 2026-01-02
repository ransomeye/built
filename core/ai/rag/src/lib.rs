// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/rag/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory-only RAG module - retrieves context for recommendations without enforcement

/// Advisory-only RAG module.
/// This module provides retrieval-augmented generation for AI recommendations.
/// It does NOT perform enforcement, policy execution, or dispatcher actions.

pub mod errors;
pub mod index;
pub mod retrieval;

pub use errors::RAGError;
pub use index::{RAGIndex, IndexDocument, IndexMetadata};
pub use retrieval::RAGRetriever;
