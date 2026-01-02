// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/outputs.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Advisory output structure - read-only advisory outputs with SHAP

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryOutput {
    pub output_id: String,
    pub created_at: DateTime<Utc>,
    pub advisory_score: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub shap_explanation: SHAPExplanation,
    pub evidence_references: Vec<String>,
    pub model_version: String,
    pub model_signature: String,
    pub context_enrichment: Option<ContextEnrichment>,
    pub advisory_type: AdvisoryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SHAPExplanation {
    pub feature_contributions: Vec<FeatureContribution>,
    pub baseline_value: f64,
    pub output_value: f64,
    pub shap_version: String,
    pub explanation_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub feature_name: String,
    pub contribution: f64,
    pub importance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEnrichment {
    pub related_alerts: Vec<String>,
    pub historical_context: Vec<String>,
    pub threat_intel_matches: Vec<String>,
    pub kill_chain_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdvisoryType {
    RiskScore,
    AnomalyDetection,
    BehaviorAnalysis,
    ContextEnrichment,
    SOCAssistance,
}

impl AdvisoryOutput {
    pub fn new(
        advisory_score: f64,
        confidence_lower: f64,
        confidence_upper: f64,
        shap_explanation: SHAPExplanation,
        model_version: &str,
        model_signature: &str,
        advisory_type: AdvisoryType,
    ) -> Self {
        Self {
            output_id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            advisory_score,
            confidence_lower,
            confidence_upper,
            shap_explanation,
            evidence_references: Vec::new(),
            model_version: model_version.to_string(),
            model_signature: model_signature.to_string(),
            context_enrichment: None,
            advisory_type,
        }
    }
    
    /// Verify SHAP explanation is present
    pub fn has_shap(&self) -> bool {
        !self.shap_explanation.feature_contributions.is_empty()
    }
    
    /// Verify output integrity
    pub fn verify_integrity(&self) -> bool {
        // Verify SHAP is present
        if !self.has_shap() {
            return false;
        }
        
        // Verify confidence bounds are valid
        if self.confidence_lower > self.confidence_upper {
            return false;
        }
        
        // Verify score is within bounds
        if self.advisory_score < self.confidence_lower || self.advisory_score > self.confidence_upper {
            return false;
        }
        
        true
    }
}

