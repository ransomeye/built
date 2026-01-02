// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/explainer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SHAP explainer - mandatory SHAP explanations for all outputs

use sha2::{Sha256, Digest};
use hex;
use tracing::{error, warn, debug};
use crate::errors::AdvisoryError;
use crate::outputs::{SHAPExplanation, FeatureContribution};
use crate::registry::Model;

pub struct SHAPExplainer {
    shap_version: String,
}

impl SHAPExplainer {
    pub fn new() -> Self {
        Self {
            shap_version: "1.0.0".to_string(),
        }
    }
    
    /// Generate SHAP explanation (MANDATORY for all outputs)
    pub fn explain(&self, model: &Model, features: &[f64], output_value: f64) -> Result<SHAPExplanation, AdvisoryError> {
        debug!("Generating SHAP explanation for {} features", features.len());
        
        if features.is_empty() {
            return Err(AdvisoryError::RuntimeError("Empty feature vector".to_string()));
        }
        
        // Calculate baseline (mean of features)
        let baseline_value = features.iter().sum::<f64>() / features.len() as f64;
        
        // Generate feature contributions (simplified SHAP)
        // In production, would use actual SHAP library
        let mut feature_contributions = Vec::new();
        let mut total_contribution = 0.0;
        
        for (i, &feature) in features.iter().enumerate() {
            // Calculate contribution as difference from baseline
            let contribution = feature - baseline_value;
            let importance = contribution.abs();
            
            feature_contributions.push(FeatureContribution {
                feature_name: format!("feature_{}", i),
                contribution,
                importance,
            });
            
            total_contribution += contribution;
        }
        
        // Normalize contributions to match output
        let scale_factor = if total_contribution.abs() > 1e-10 {
            (output_value - baseline_value) / total_contribution
        } else {
            1.0
        };
        
        for contribution in &mut feature_contributions {
            contribution.contribution *= scale_factor;
            contribution.importance = contribution.contribution.abs();
        }
        
        // Sort by importance
        feature_contributions.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        
        // Compute explanation hash
        let explanation_hash = self.compute_explanation_hash(&feature_contributions, baseline_value, output_value);
        
        let explanation = SHAPExplanation {
            feature_contributions,
            baseline_value,
            output_value,
            shap_version: self.shap_version.clone(),
            explanation_hash,
        };
        
        debug!("SHAP explanation generated with {} features", explanation.feature_contributions.len());
        Ok(explanation)
    }
    
    fn compute_explanation_hash(&self, contributions: &[FeatureContribution], baseline: f64, output: f64) -> String {
        use serde_json;
        
        let mut hasher = Sha256::new();
        
        // Hash contributions
        for contrib in contributions {
            hasher.update(contrib.feature_name.as_bytes());
            hasher.update(&contrib.contribution.to_be_bytes());
        }
        
        // Hash baseline and output
        hasher.update(&baseline.to_be_bytes());
        hasher.update(&output.to_be_bytes());
        
        hex::encode(hasher.finalize())
    }
    
    /// Validate SHAP explanation
    pub fn validate(&self, explanation: &SHAPExplanation) -> Result<(), AdvisoryError> {
        if explanation.feature_contributions.is_empty() {
            return Err(AdvisoryError::MissingSHAP("SHAP explanation is empty".to_string()));
        }
        
        // Verify hash
        let computed_hash = self.compute_explanation_hash(
            &explanation.feature_contributions,
            explanation.baseline_value,
            explanation.output_value,
        );
        
        if computed_hash != explanation.explanation_hash {
            return Err(AdvisoryError::SHAPValidationFailed(
                format!("SHAP hash mismatch: expected {}, got {}", explanation.explanation_hash, computed_hash)
            ));
        }
        
        Ok(())
    }
}

