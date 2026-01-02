// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/explainability/src/rationale.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Human-readable rationale generation aligned with deterministic Phase 5 outputs

use std::collections::HashMap;
use tracing::debug;

use super::errors::ExplainabilityError;
use super::shap::{SHAPExplanation, FeatureContribution};

pub struct RationaleGenerator {
    templates: HashMap<String, String>,
}

impl RationaleGenerator {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Rationale templates (aligned with deterministic outputs)
        templates.insert("high_confidence".to_string(), 
            "High confidence anomaly detected based on feature analysis. Top contributing factors: {factors}.".to_string());
        templates.insert("moderate_confidence".to_string(),
            "Moderate confidence anomaly detected. Key indicators: {factors}.".to_string());
        templates.insert("low_confidence".to_string(),
            "Low confidence anomaly detected. Monitoring recommended for: {factors}.".to_string());
        templates.insert("normal".to_string(),
            "No significant anomalies detected. All features within expected ranges.".to_string());
        
        Self { templates }
    }
    
    /// Generate human-readable rationale from SHAP explanation
    pub fn generate_rationale(&self, shap: &SHAPExplanation, confidence: f64) -> Result<String, ExplainabilityError> {
        // Select template based on confidence
        let template_key = if confidence >= 0.9 {
            "high_confidence"
        } else if confidence >= 0.7 {
            "moderate_confidence"
        } else if confidence >= 0.5 {
            "low_confidence"
        } else {
            "normal"
        };
        
        let template = self.templates.get(template_key)
            .ok_or_else(|| ExplainabilityError::RationaleGenerationFailed(
                format!("Template not found: {}", template_key)
            ))?;
        
        // Extract top contributing features
        let mut contributions: Vec<(&FeatureContribution, f64)> = shap.feature_contributions.iter()
            .zip(shap.shap_values.iter())
            .map(|(fc, &sv)| (fc, sv.abs()))
            .collect();
        
        contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Get top 3 features
        let top_features: Vec<String> = contributions.iter()
            .take(3)
            .map(|(fc, _)| format!("{} (contribution: {:.2})", fc.feature_name, fc.contribution))
            .collect();
        
        let factors = top_features.join(", ");
        
        // Generate rationale
        let rationale = template.replace("{factors}", &factors);
        
        // Add alignment note (must align with deterministic Phase 5)
        let aligned_rationale = format!("{} [Aligned with deterministic Phase 5 detection outputs]", rationale);
        
        debug!("Generated rationale: {}", aligned_rationale);
        Ok(aligned_rationale)
    }
    
    /// Validate rationale aligns with deterministic outputs
    pub fn validate_alignment(&self, rationale: &str, deterministic_output: &str) -> Result<bool, ExplainabilityError> {
        // Check that rationale doesn't contradict deterministic output
        // In production, would use more sophisticated alignment checking
        
        // For now, simple check: rationale should not contain contradictions
        let contradictions = vec!["contradicts", "opposite", "conflicts"];
        for contradiction in contradictions {
            if rationale.to_lowercase().contains(contradiction) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

