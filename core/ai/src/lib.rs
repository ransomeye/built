// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Root library for advisory-only AI module - provides recommendations without enforcement

/// RansomEye AI Advisory Module
/// 
/// This module provides advisory-only AI capabilities:
/// - Inference for recommendations
/// - Explainability (SHAP-based)
/// - RAG for context retrieval
/// 
/// **CRITICAL**: This module does NOT perform enforcement, policy execution, or dispatcher actions.
/// All functionality is advisory-only.

pub mod config;
pub mod security;

pub use config::AdvisoryConfig;

// Note: Workspace members (inference, explainability, rag) are separate crates
// Import them directly: use ransomeye_ai_advisory_inference::*;
