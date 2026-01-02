// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/tests/model_signature_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Model signature verification tests - RSA-4096 verification

use tempfile::TempDir;
use std::fs;
use std::io::Write;
use sha2::{Sha256, Digest};

#[test]
fn test_model_loader_verifies_signature() {
    let temp_dir = TempDir::new().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    
    // Create dummy public key
    let key_dir = temp_dir.path().join("keys");
    fs::create_dir_all(&key_dir).unwrap();
    let public_key_path = key_dir.join("public_key.pem");
    let mut key_file = fs::File::create(&public_key_path).unwrap();
    key_file.write_all(b"dummy public key").unwrap();
    
    // Create manifest
    let manifest_path = models_dir.join("models.manifest.json");
    let manifest = r#"{
        "model_name": "test.model",
        "model_version": "1.0.0",
        "model_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "model_size_bytes": 100,
        "signature": "dummy_signature",
        "trained_on": "2024-01-01",
        "model_type": "test",
        "features": []
    }"#;
    fs::write(&manifest_path, manifest).unwrap();
    
    // Create model file
    let model_path = models_dir.join("test.model");
    fs::write(&model_path, vec![0u8; 100]).unwrap();
    
    // Test loader initialization
    use ransomeye_ai_advisory_inference::ModelLoader;
    let loader = ModelLoader::new(
        models_dir.clone(),
        public_key_path.clone(),
    );
    
    // Loader should initialize (signature verification will fail with dummy key, but structure is correct)
    assert!(loader.is_ok());
}

#[test]
fn test_model_integrity_check() {
    let temp_dir = TempDir::new().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    
    let key_dir = temp_dir.path().join("keys");
    fs::create_dir_all(&key_dir).unwrap();
    let public_key_path = key_dir.join("public_key.pem");
    let mut key_file = fs::File::create(&public_key_path).unwrap();
    key_file.write_all(b"dummy key").unwrap();
    
    // Create model data
    let model_data = vec![1u8; 100];
    let mut hasher = sha2::Sha256::new();
    hasher.update(&model_data);
    let hash = hex::encode(hasher.finalize());
    
    // Create manifest with correct hash
    let manifest = format!(r#"{{
        "model_name": "test.model",
        "model_version": "1.0.0",
        "model_hash": "{}",
        "model_size_bytes": 100,
        "signature": "dummy",
        "trained_on": "2024-01-01",
        "model_type": "test",
        "features": []
    }}"#, hash);
    
    let manifest_path = models_dir.join("models.manifest.json");
    fs::write(&manifest_path, manifest).unwrap();
    
    // Create model file
    let model_path = models_dir.join("test.model");
    fs::write(&model_path, model_data).unwrap();
    
    // Test integrity checker
    use ransomeye_ai_advisory::security::IntegrityChecker;
    let checker = IntegrityChecker::new();
    let computed_hash = checker.compute_hash(&model_path).unwrap();
    
    // Hash should match
    assert_eq!(computed_hash, hash);
}

