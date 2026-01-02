// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/explainability/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory-only explainability module - explains AI recommendations without enforcement

/// Advisory-only explainability module.
/// This module provides SHAP-based explanations for AI recommendations.
/// It does NOT perform enforcement, policy execution, or dispatcher actions.

pub mod errors;
pub mod shap;
pub mod rationale;

pub use errors::ExplainabilityError;
pub use shap::{SHAPExplainer, SHAPExplanation, FeatureContribution};
pub use rationale::RationaleGenerator;
