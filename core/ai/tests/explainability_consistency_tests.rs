// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/tests/explainability_consistency_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SHAP explainability consistency tests

use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_shap_explanation_validation() {
    let mut explainer = ransomeye_ai_advisory::explainability::shap::SHAPExplainer::new();
    
    let features = vec![0.5, 0.6, 0.7, 0.8, 0.9];
    let feature_names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string(), "f4".to_string(), "f5".to_string()];
    let output = 0.75;
    
    let shap = explainer.compute_shap(&features, &feature_names, output).unwrap();
    
    // Validate SHAP structure
    assert_eq!(shap.feature_contributions.len(), features.len());
    assert_eq!(shap.shap_values.len(), features.len());
    assert_eq!(shap.output_value, output);
    
    // Validate SHAP values sum to output - baseline
    let shap_sum: f64 = shap.shap_values.iter().sum();
    let expected_sum = shap.output_value - shap.baseline_value;
    assert!((shap_sum - expected_sum).abs() < 0.1); // Allow small floating point error
    
    // Validate each feature has contribution
    for contribution in &shap.feature_contributions {
        assert!(!contribution.feature_name.is_empty());
        assert!(contribution.signal_reference.is_some());
        assert!(contribution.timestamp_reference.is_some());
    }
    
    // Validate SHAP explanation
    let validation_result = explainer.validate(&shap);
    assert!(validation_result.is_ok());
}

#[test]
fn test_rationale_generation() {
    let generator = ransomeye_ai_advisory::explainability::rationale::RationaleGenerator::new();
    
    let mut explainer = ransomeye_ai_advisory::explainability::shap::SHAPExplainer::new();
    let features = vec![0.8, 0.9, 0.95];
    let feature_names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string()];
    let output = 0.9;
    
    let shap = explainer.compute_shap(&features, &feature_names, output).unwrap();
    
    // Generate rationale
    let rationale = generator.generate_rationale(&shap, 0.9).unwrap();
    
    // Verify rationale is human-readable
    assert!(!rationale.is_empty());
    assert!(rationale.len() > 20); // Should be substantial
    
    // Verify rationale mentions alignment
    assert!(rationale.contains("Aligned") || rationale.contains("aligned"));
    
    // Verify rationale doesn't contain enforcement language
    assert!(!rationale.to_lowercase().contains("enforce"));
    assert!(!rationale.to_lowercase().contains("block"));
}

