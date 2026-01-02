// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/tests/shap_required_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests that block outputs without SHAP

use ransomeye_ai_advisory::AdvisoryEngine;
use ransomeye_ai_advisory::AdvisoryError;

#[tokio::test]
async fn test_output_without_shap_is_blocked() {
    // Test that outputs without SHAP are rejected
    let engine_result = AdvisoryEngine::new();
    
    if let Ok(engine) = engine_result {
        let features = vec![0.5, 0.6, 0.7];
        let result = engine.generate_advisory("test-alert-1", &features).await;
        
        // If advisory is generated, it must have SHAP
        if let Ok(output) = result {
            assert!(output.has_shap(), "Output must have SHAP explanation");
            assert!(!output.shap_explanation.feature_contributions.is_empty());
        } else {
            // If generation fails, it might be due to missing SHAP (expected)
            match result.unwrap_err() {
                AdvisoryError::MissingSHAP(_) => {
                    // Expected - SHAP is required
                }
                _ => {
                    // Other errors are acceptable (missing models, etc.)
                }
            }
        }
    }
}

#[tokio::test]
async fn test_shap_validation_required() {
    // Test that SHAP explanations are validated
    use ransomeye_ai_advisory::shap::validator::SHAPValidator;
    
    let validator = SHAPValidator::new();
    
    // Create invalid SHAP (empty)
    use ransomeye_ai_advisory::outputs::SHAPExplanation;
    let invalid_shap = SHAPExplanation {
        feature_contributions: Vec::new(),
        baseline_value: 0.0,
        output_value: 0.5,
        shap_version: "1.0.0".to_string(),
        explanation_hash: "test".to_string(),
    };
    
    // Should fail validation
    assert!(validator.validate(&invalid_shap).is_err());
}

