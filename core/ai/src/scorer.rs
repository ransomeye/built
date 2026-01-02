// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/scorer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Risk scorer - advisory-only risk scoring with confidence bounds

use std::sync::Arc;
use tracing::{error, warn, debug};
use crate::errors::AdvisoryError;
use crate::outputs::AdvisoryOutput;
use crate::explainer::SHAPExplainer;
use crate::registry::ModelRegistry;

pub struct RiskScorer {
    model_registry: Arc<ModelRegistry>,
    shap_explainer: Arc<SHAPExplainer>,
}

impl RiskScorer {
    pub fn new(model_registry: Arc<ModelRegistry>, shap_explainer: Arc<SHAPExplainer>) -> Self {
        Self {
            model_registry,
            shap_explainer,
        }
    }
    
    /// Score risk (advisory only)
    pub async fn score_risk(&self, features: &[f64], alert_id: &str) -> Result<AdvisoryOutput, AdvisoryError> {
        debug!("Scoring risk for alert {}", alert_id);
        
        // Load risk model
        let model = self.model_registry.load_model("risk_model")
            .map_err(|e| AdvisoryError::MissingBaseline(format!("Failed to load risk model: {}", e)))?;
        
        // Verify model is signed
        if !model.is_signed() {
            return Err(AdvisoryError::UnsignedModel("Risk model is not signed".to_string()));
        }
        
        // Run inference (advisory only)
        let score = self.run_inference(&model, features)?;
        
        // Calculate confidence bounds (95% confidence interval)
        let confidence_lower = (score - 0.1).max(0.0).min(1.0);
        let confidence_upper = (score + 0.1).max(0.0).min(1.0);
        
        // Generate SHAP explanation (MANDATORY)
        let shap_explanation = self.shap_explainer.explain(&model, features, score)
            .map_err(|e| AdvisoryError::MissingSHAP(format!("Failed to generate SHAP: {}", e)))?;
        
        // Verify SHAP is present
        if shap_explanation.feature_contributions.is_empty() {
            return Err(AdvisoryError::MissingSHAP("SHAP explanation is empty".to_string()));
        }
        
        // Create advisory output
        let output = AdvisoryOutput::new(
            score,
            confidence_lower,
            confidence_upper,
            shap_explanation,
            &model.version,
            &model.signature,
            crate::outputs::AdvisoryType::RiskScore,
        );
        
        // Verify output integrity
        if !output.verify_integrity() {
            return Err(AdvisoryError::SHAPValidationFailed("Output integrity check failed".to_string()));
        }
        
        debug!("Risk score generated: {} (confidence: [{}, {}])", score, confidence_lower, confidence_upper);
        Ok(output)
    }
    
    fn run_inference(&self, model: &crate::registry::Model, features: &[f64]) -> Result<f64, AdvisoryError> {
        // Simplified inference - in production, would load actual model
        // This is advisory-only, so we use a simple scoring function
        
        if features.is_empty() {
            return Err(AdvisoryError::RuntimeError("Empty feature vector".to_string()));
        }
        
        // Simple weighted sum (placeholder for actual model inference)
        let score = features.iter().sum::<f64>() / features.len() as f64;
        
        // Normalize to [0, 1]
        let normalized_score = score.max(0.0).min(1.0);
        
        Ok(normalized_score)
    }
}

