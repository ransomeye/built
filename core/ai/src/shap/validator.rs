// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/shap/validator.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SHAP validator - validates SHAP explanations

use tracing::debug;
use crate::errors::AdvisoryError;
use crate::outputs::SHAPExplanation;
use crate::explainer::SHAPExplainer;

pub struct SHAPValidator;

impl SHAPValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate SHAP explanation
    pub fn validate(&self, explanation: &SHAPExplanation) -> Result<(), AdvisoryError> {
        debug!("Validating SHAP explanation");
        
        let explainer = SHAPExplainer::new();
        explainer.validate(explanation)
    }
    
    /// Check if SHAP is present
    pub fn has_shap(&self, explanation: &SHAPExplanation) -> bool {
        !explanation.feature_contributions.is_empty()
    }
}

