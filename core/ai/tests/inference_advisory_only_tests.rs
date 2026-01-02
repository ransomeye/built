// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/tests/inference_advisory_only_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests proving advisory-only behavior - NO enforcement

use std::sync::Arc;
use tempfile::TempDir;
use std::fs;
use std::io::Write;

#[test]
fn test_inference_returns_advisory_only() {
    // Create test infrastructure
    let temp_dir = TempDir::new().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    
    let key_dir = temp_dir.path().join("keys");
    fs::create_dir_all(&key_dir).unwrap();
    let public_key_path = key_dir.join("public_key.pem");
    let mut key_file = fs::File::create(&public_key_path).unwrap();
    key_file.write_all(b"dummy key").unwrap();
    
    // Initialize components
    use ransomeye_ai_advisory_inference::{ModelLoader, FeatureExtractor, ConfidenceCalibrator, ThresholdManager, AdvisoryInference};
    
    let loader = Arc::new(
        ModelLoader::new(
            models_dir.clone(),
            public_key_path.clone(),
        ).unwrap()
    );
    
    let feature_extractor = Arc::new(FeatureExtractor::new());
    let calibrator = Arc::new(ConfidenceCalibrator::new());
    let threshold_manager = Arc::new(ThresholdManager::new());
    
    let inference = AdvisoryInference::new(
        loader,
        feature_extractor,
        calibrator,
        threshold_manager,
    );
    
    // Run inference (synchronous)
    let features = vec![0.5, 0.6, 0.7, 0.8, 0.9];
    let result = inference.infer("test_model", &features);
    
    // Verify result is advisory-only (contains confidence, recommendation, NO enforcement)
    assert!(result.is_ok());
    let advisory_result = result.unwrap();
    
    // Verify advisory-only fields present
    assert!(advisory_result.confidence >= 0.0 && advisory_result.confidence <= 1.0);
    assert!(advisory_result.calibrated_confidence >= 0.0 && advisory_result.calibrated_confidence <= 1.0);
    assert!(!advisory_result.recommendation.is_empty());
    
    // Verify NO enforcement fields
    // Recommendation should be advisory text, not enforcement command
    assert!(!advisory_result.recommendation.contains("enforce"));
    assert!(!advisory_result.recommendation.contains("block"));
    assert!(!advisory_result.recommendation.contains("isolate"));
    assert!(advisory_result.recommendation.contains("recommend") || 
            advisory_result.recommendation.contains("investigation") ||
            advisory_result.recommendation.contains("monitor"));
}

#[test]
fn test_feature_extraction_bounded() {
    use ransomeye_ai_advisory_inference::FeatureExtractor;
    let extractor = FeatureExtractor::new();
    
    // Test normal features
    let features = vec![0.1, 0.2, 0.3];
    let result = extractor.extract(&features);
    assert!(result.is_ok());
    
    // Test too many features (should fail)
    let too_many = vec![0.0; 2000]; // Exceeds max_features (1000)
    let result = extractor.extract(&too_many);
    assert!(result.is_err());
    
    // Test invalid features (NaN)
    let invalid = vec![0.0, f64::NAN, 0.2];
    let result = extractor.extract(&invalid);
    assert!(result.is_err());
}

