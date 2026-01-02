// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/explainability/src/shap.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SHAP-based explanations with feature/signal/timestamp references

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, info, debug};

use super::errors::ExplainabilityError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SHAPExplanation {
    pub feature_contributions: Vec<FeatureContribution>,
    pub baseline_value: f64,
    pub output_value: f64,
    pub shap_values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub feature_name: String,
    pub contribution: f64,
    pub signal_reference: Option<String>,
    pub timestamp_reference: Option<String>,
}

pub struct SHAPExplainer {
    baseline: Option<SHAPBaseline>,
}

#[derive(Debug, Clone, Deserialize)]
struct SHAPBaseline {
    baseline_values: Vec<f64>,
    feature_names: Vec<String>,
}

impl SHAPExplainer {
    pub fn new() -> Self {
        Self {
            baseline: None,
        }
    }
    
    /// Load SHAP baseline from file
    pub fn load_baseline(&mut self, baseline_path: &Path) -> Result<(), ExplainabilityError> {
        if !baseline_path.exists() {
            warn!("SHAP baseline file not found: {:?}, using defaults", baseline_path);
            return Ok(());
        }
        
        let baseline_json = fs::read_to_string(baseline_path)
            .map_err(|e| ExplainabilityError::BaselineMissing(
                format!("Failed to read baseline file {:?}: {}", baseline_path, e)
            ))?;
        
        let baseline: SHAPBaseline = serde_json::from_str(&baseline_json)
            .map_err(|e| ExplainabilityError::BaselineMissing(
                format!("Failed to parse baseline: {}", e)
            ))?;
        
        self.baseline = Some(baseline);
        info!("SHAP baseline loaded from {:?}", baseline_path);
        Ok(())
    }
    
    /// Compute SHAP values for features
    pub fn compute_shap(&self, features: &[f64], feature_names: &[String], output: f64) -> Result<SHAPExplanation, ExplainabilityError> {
        if features.len() != feature_names.len() {
            return Err(ExplainabilityError::FeatureMismatch(
                format!("Feature count mismatch: {} features, {} names", features.len(), feature_names.len())
            ));
        }
        
        // Get baseline value
        let baseline_value = self.get_baseline_value(features.len());
        
        // Simplified SHAP computation (in production, would use actual SHAP library)
        // This computes approximate SHAP values using feature differences from baseline
        let mut shap_values = Vec::with_capacity(features.len());
        let mut feature_contributions = Vec::with_capacity(features.len());
        
        for (i, (&feature_val, feature_name)) in features.iter().zip(feature_names.iter()).enumerate() {
            // Compute contribution as difference from baseline
            let baseline_feature = self.get_baseline_feature(i);
            let contribution = feature_val - baseline_feature;
            
            // Normalize contribution
            let shap_value = contribution / features.len() as f64;
            shap_values.push(shap_value);
            
            feature_contributions.push(FeatureContribution {
                feature_name: feature_name.clone(),
                contribution,
                signal_reference: Some(format!("signal_{}", i)),
                timestamp_reference: Some(chrono::Utc::now().to_rfc3339().to_string()),
            });
        }
        
        debug!("SHAP explanation computed: {} features, output={:.3}", features.len(), output);
        
        Ok(SHAPExplanation {
            feature_contributions,
            baseline_value,
            output_value: output,
            shap_values,
        })
    }
    
    /// Validate SHAP explanation
    pub fn validate(&self, explanation: &SHAPExplanation) -> Result<(), ExplainabilityError> {
        // Validate SHAP values sum to output - baseline
        let shap_sum: f64 = explanation.shap_values.iter().sum();
        let expected_sum = explanation.output_value - explanation.baseline_value;
        
        if (shap_sum - expected_sum).abs() > 0.01 {
            return Err(ExplainabilityError::SHAPValidationFailed(
                format!("SHAP sum mismatch: expected {:.3}, got {:.3}", expected_sum, shap_sum)
            ));
        }
        
        // Validate all features have contributions
        if explanation.feature_contributions.is_empty() {
            return Err(ExplainabilityError::SHAPValidationFailed(
                "No feature contributions in SHAP explanation".to_string()
            ));
        }
        
        // Validate feature contributions match shap values
        if explanation.feature_contributions.len() != explanation.shap_values.len() {
            return Err(ExplainabilityError::SHAPValidationFailed(
                format!("Feature contributions count mismatch: {} != {}", 
                    explanation.feature_contributions.len(), explanation.shap_values.len())
            ));
        }
        
        debug!("SHAP explanation validated successfully");
        Ok(())
    }
    
    fn get_baseline_value(&self, feature_count: usize) -> f64 {
        if let Some(ref baseline) = self.baseline {
            if baseline.baseline_values.len() == feature_count {
                // Return mean of baseline values
                baseline.baseline_values.iter().sum::<f64>() / baseline.baseline_values.len() as f64
            } else {
                0.5 // Default baseline
            }
        } else {
            0.5 // Default baseline
        }
    }
    
    fn get_baseline_feature(&self, index: usize) -> f64 {
        if let Some(ref baseline) = self.baseline {
            if index < baseline.baseline_values.len() {
                baseline.baseline_values[index]
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

