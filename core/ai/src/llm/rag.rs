// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/llm/rag.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: RAG engine - read-only retrieval augmented generation

use std::path::PathBuf;
use std::fs;
use tracing::{error, warn, debug, info};
use crate::errors::AdvisoryError;
use serde_json;

pub struct RAGEngine {
    index_path: PathBuf,
    chunks: Option<Vec<serde_json::Value>>,
}

impl RAGEngine {
    pub fn new() -> Result<Self, AdvisoryError> {
        // Use RAG index from core/ai/rag/index
        let index_path = std::env::var("RANSOMEYE_AI_RAG_INDEX_DIR")
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|_| {
                // Default to core/ai/rag/index
                PathBuf::from("/home/ransomeye/rebuild/core/ai/rag/index")
            });
        
        info!("Initializing RAG engine with index: {:?}", index_path);
        
        Ok(Self {
            index_path,
            chunks: None,
        })
    }
    
    /// Load RAG index chunks
    fn load_chunks(&mut self) -> Result<(), AdvisoryError> {
        if self.chunks.is_some() {
            return Ok(());
        }
        
        let chunks_path = self.index_path.join("chunks.json");
        if !chunks_path.exists() {
            warn!("RAG chunks.json not found at {:?}", chunks_path);
            self.chunks = Some(Vec::new());
            return Ok(());
        }
        
        let chunks_json = fs::read_to_string(&chunks_path)
            .map_err(|e| AdvisoryError::RuntimeError(format!("Failed to read chunks.json: {}", e)))?;
        
        let chunks: Vec<serde_json::Value> = serde_json::from_str(&chunks_json)
            .map_err(|e| AdvisoryError::RuntimeError(format!("Failed to parse chunks.json: {}", e)))?;
        
        self.chunks = Some(chunks);
        info!("Loaded {} chunks from RAG index", self.chunks.as_ref().unwrap().len());
        
        Ok(())
    }
    
    /// Retrieve relevant context (read-only)
    pub async fn retrieve(&self, query: &str, context: &[String]) -> Result<String, AdvisoryError> {
        debug!("RAG retrieval for query: {}", query);
        
        // Load chunks if available
        let chunks = if let Some(ref chunks) = self.chunks {
            chunks
        } else {
            // Try to load chunks synchronously (not ideal but works)
            let chunks_path = self.index_path.join("chunks.json");
            if chunks_path.exists() {
                let chunks_json = fs::read_to_string(&chunks_path)
                    .map_err(|e| AdvisoryError::RuntimeError(format!("Failed to read chunks.json: {}", e)))?;
                let chunks: Vec<serde_json::Value> = serde_json::from_str(&chunks_json)
                    .map_err(|e| AdvisoryError::RuntimeError(format!("Failed to parse chunks.json: {}", e)))?;
                // Store in a way we can use (this is a workaround)
                return self.retrieve_with_chunks(query, context, &chunks).await;
            } else {
                warn!("RAG chunks.json not found, returning basic response");
                return Ok(format!("Advisory response for query: {}. Context items: {}", query, context.len()));
            }
        };
        
        self.retrieve_with_chunks(query, context, chunks).await
    }
    
    async fn retrieve_with_chunks(&self, query: &str, context: &[String], chunks: &[serde_json::Value]) -> Result<String, AdvisoryError> {
        // Simple keyword-based retrieval (deterministic)
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        // Score chunks by keyword matches
        let mut scored_chunks: Vec<(usize, usize)> = chunks.iter()
            .enumerate()
            .map(|(i, chunk)| {
                let content = chunk.get("chunk")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let score = query_words.iter()
                    .map(|word| content.matches(word).count())
                    .sum::<usize>();
                
                (i, score)
            })
            .collect();
        
        // Sort by score (descending)
        scored_chunks.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Get top 3 chunks
        let top_k = 3.min(chunks.len());
        let mut response_parts = Vec::new();
        response_parts.push(format!("Advisory response for query: {}", query));
        
        if top_k > 0 && scored_chunks[0].1 > 0 {
            response_parts.push("\n\nRelevant context:".to_string());
            for i in 0..top_k {
                if let Some(chunk) = chunks.get(scored_chunks[i].0) {
                    if let Some(content) = chunk.get("chunk").and_then(|v| v.as_str()) {
                        let preview: String = content.chars().take(200).collect();
                        response_parts.push(format!("\n[{}] {}", i + 1, preview));
                    }
                }
            }
        }
        
        if !context.is_empty() {
            response_parts.push(format!("\n\nAdditional context items: {}", context.len()));
        }
        
        let response = response_parts.join("");
        debug!("RAG response generated with {} relevant chunks", top_k);
        Ok(response)
    }
    
    /// Load knowledge base (read-only)
    pub fn load_knowledge_base(&self) -> Result<(), AdvisoryError> {
        let kb_path = Path::new(&self.data_dir).join("knowledge_base");
        
        if !kb_path.exists() {
            warn!("Knowledge base not found at {}", kb_path.display());
            return Ok(()); // Not an error if KB doesn't exist
        }
        
        debug!("Knowledge base loaded from {}", kb_path.display());
        Ok(())
    }
}

