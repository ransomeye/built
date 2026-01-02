// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/inference/src/thresholds.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Read-only threshold management for advisory outputs

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::debug;

use super::errors::InferenceError;

pub struct ThresholdManager {
    thresholds: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl ThresholdManager {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        
        // Default thresholds (read-only, advisory only)
        thresholds.insert("ransomware_behavior.model".to_string(), vec![0.5, 0.7, 0.9]);
        thresholds.insert("anomaly_baseline.model".to_string(), vec![0.4, 0.6, 0.8]);
        thresholds.insert("confidence_calibration.model".to_string(), vec![0.3, 0.5, 0.7]);
        
        Self {
            thresholds: Arc::new(RwLock::new(thresholds)),
        }
    }
    
    /// Get thresholds for model (read-only)
    pub fn get_thresholds(&self, model_name: &str) -> Result<Vec<f64>, InferenceError> {
        let thresholds = self.thresholds.read();
        
        // Try exact match first
        if let Some(thresholds) = thresholds.get(model_name) {
            return Ok(thresholds.clone());
        }
        
        // Try with .model suffix
        let model_name_with_suffix = format!("{}.model", model_name);
        if let Some(thresholds) = thresholds.get(&model_name_with_suffix) {
            return Ok(thresholds.clone());
        }
        
        // Return default thresholds
        debug!("No specific thresholds found for {}, using defaults", model_name);
        Ok(vec![0.5, 0.7, 0.9])
    }
    
    /// Validate threshold (read-only check)
    pub fn validate_threshold(&self, model_name: &str, threshold: f64) -> Result<bool, InferenceError> {
        let thresholds = self.get_thresholds(model_name)?;
        
        // Check if threshold is in valid range [0, 1]
        if threshold < 0.0 || threshold > 1.0 {
            return Ok(false);
        }
        
        // Check if threshold is reasonable (within known thresholds)
        Ok(thresholds.iter().any(|&t| (threshold - t).abs() < 0.1))
    }
}

