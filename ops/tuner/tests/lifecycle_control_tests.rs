// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/tests/lifecycle_control_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Lifecycle control tests - validates service lifecycle management

use ransomeye_operations::*;
use tempfile::TempDir;

#[test]
fn test_startup_requires_valid_state() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let starter = ServiceStarter::new(&state_path, &keys_dir);
    
    // Attempt to start without valid state
    let result = starter.start_all();
    
    // Should fail (state doesn't exist)
    assert!(result.is_err());
}

#[test]
fn test_status_check_requires_valid_state() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let checker = ServiceStatusChecker::new(&state_path, &keys_dir);
    
    // Attempt to check status without valid state
    let result = checker.validate_state();
    
    // Should fail (state doesn't exist)
    assert!(result.is_err());
}

#[test]
fn test_lifecycle_with_valid_state() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    // Create valid state
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let _state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Now lifecycle operations should work
    let checker = ServiceStatusChecker::new(&state_path, &keys_dir);
    let result = checker.validate_state();
    
    // Should succeed with valid state
    assert!(result.is_ok());
}

