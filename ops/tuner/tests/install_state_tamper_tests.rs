// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/tests/install_state_tamper_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Install state tamper tests - detects tampering with install state

use ransomeye_operations::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_tampered_state_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    
    // Create valid state
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Tamper with state file
    let state_json = fs::read_to_string(&state_path).unwrap();
    let mut tampered_json = state_json.replace("1.0.0", "2.0.0");  // Change version
    fs::write(&state_path, tampered_json).unwrap();
    
    // Attempt to load and verify
    let result = state_manager.load();
    assert!(result.is_err());
    
    // Verify should detect tampering
    if let Ok(loaded_state) = result {
        let verify_result = state_manager.verify(&loaded_state);
        assert!(verify_result.is_err());
    }
}

#[test]
fn test_state_hash_mismatch_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    
    // Create valid state
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let mut state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Tamper with state hash
    state.state_hash = "tampered_hash".to_string();
    
    // Verify should detect tampering
    let result = state_manager.verify(&state);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OperationsError::InstallStateTampered(_)));
}

#[test]
fn test_signature_verification_failure() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    
    // Create valid state
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let mut state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Tamper with signature
    state.signature = "invalid_signature".to_string();
    
    // Verify should detect invalid signature
    let result = state_manager.verify(&state);
    assert!(result.is_err());
}

