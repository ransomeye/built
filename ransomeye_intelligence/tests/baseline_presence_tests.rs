// Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/tests/baseline_presence_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests that baseline intelligence pack is present and valid - AI cannot start without baseline

/*
 * Baseline Presence Tests
 * 
 * Tests that verify baseline intelligence pack is present and valid.
 * AI cannot start without valid baseline pack.
 */

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_baseline_pack_exists() {
        // Test that baseline pack directory exists
        let pack_dir = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack");
        assert!(pack_dir.exists(), "Baseline pack directory must exist");
    }

    #[test]
    fn test_model_manifest_exists() {
        // Test that model manifest exists
        let manifest_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/model_manifest.json");
        assert!(manifest_path.exists(), "Model manifest must exist");
    }

    #[test]
    fn test_required_models_exist() {
        // Test that all required models exist
        let required_models = vec![
            "ransomware_behavior.model",
            "anomaly_baseline.model",
            "confidence_calibration.model"
        ];
        
        for model_name in required_models {
            let model_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models").join(model_name);
            assert!(model_path.exists(), "Required model must exist: {}", model_name);
        }
    }

    #[test]
    fn test_shap_baseline_exists() {
        // Test that SHAP baseline exists
        let shap_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/shap/baseline_shap_values.json");
        assert!(shap_path.exists(), "SHAP baseline must exist");
    }

    #[test]
    fn test_training_manifest_exists() {
        // Test that training manifest exists
        let manifest_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/metadata/training_manifest.json");
        assert!(manifest_path.exists(), "Training manifest must exist");
    }

    #[test]
    fn test_no_customer_data() {
        // Test that training manifest indicates no customer data
        let manifest_path = Path::new("/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/metadata/training_manifest.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(manifest_path).unwrap();
            assert!(content.contains("no_customer_data") || content.contains("\"customer_data_used\": false"), 
                   "Training must not use customer data");
        }
    }
}

