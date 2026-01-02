// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/features.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Bounded feature extraction for advisory inference

use ndarray::Array1;
use tracing::{error, debug};

use super::errors::InferenceError;

pub struct FeatureExtractor {
    max_features: usize,
    feature_bounds: (f64, f64),
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            max_features: 1000, // Bounded feature count
            feature_bounds: (-1e6, 1e6), // Bounded feature values
        }
    }
    
    /// Extract features with bounds checking
    pub fn extract(&self, input: &[f64]) -> Result<Array1<f64>, InferenceError> {
        if input.is_empty() {
            return Err(InferenceError::FeatureExtractionFailed(
                "Input features are empty".to_string()
            ));
        }
        
        if input.len() > self.max_features {
            return Err(InferenceError::FeatureExtractionFailed(
                format!("Input features exceed maximum: {} > {}", input.len(), self.max_features)
            ));
        }
        
        // Validate and bound feature values
        let mut features = Vec::with_capacity(input.len());
        for (i, &val) in input.iter().enumerate() {
            if val.is_nan() || val.is_infinite() {
                return Err(InferenceError::FeatureExtractionFailed(
                    format!("Invalid feature value at index {}: {}", i, val)
                ));
            }
            
            // Clamp to bounds
            let bounded = val.max(self.feature_bounds.0).min(self.feature_bounds.1);
            features.push(bounded);
        }
        
        let array = Array1::from_vec(features);
        debug!("Extracted {} features (bounded)", array.len());
        
        Ok(array)
    }
    
    /// Get feature count limit
    pub fn max_features(&self) -> usize {
        self.max_features
    }
}

