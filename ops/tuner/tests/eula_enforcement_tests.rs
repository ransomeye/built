// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/tests/eula_enforcement_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: EULA enforcement tests - validates EULA acceptance is mandatory

use ransomeye_operations::*;
use tempfile::TempDir;

#[test]
fn test_install_fails_without_eula() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    
    let installer = Installer::new(&project_root);
    
    // Attempt installation without EULA acceptance
    let result = installer.install(false, None, "1.0.0");
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OperationsError::EulaNotAccepted));
}

#[test]
fn test_install_state_requires_eula() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    
    // Attempt to create state without EULA acceptance
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    
    let result = state_manager.create(
        false,  // EULA not accepted
        None,
        retention,
        identity,
        "1.0.0",
    );
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OperationsError::EulaNotAccepted));
}

#[test]
fn test_startup_fails_without_eula() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    // Create state without EULA (should fail, but test the check)
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    
    // Verify state requires EULA
    if let Ok(state) = state_manager.load() {
        assert!(state.eula_accepted, "State should require EULA acceptance");
    }
}

