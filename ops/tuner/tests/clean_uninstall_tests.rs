// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/tests/clean_uninstall_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Clean uninstall tests - validates clean uninstallation with evidence preservation

use ransomeye_operations::*;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[test]
fn test_uninstall_requires_confirmation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    
    let uninstaller = Uninstaller::new(&project_root);
    
    // Attempt uninstallation without confirmation
    let result = uninstaller.uninstall(false, false, false);
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OperationsError::UninstallVerificationFailed(_)));
}

#[test]
fn test_uninstall_preserves_evidence_by_default() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    // Create install state
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Create evidence directory
    let evidence_dir = format!("{}/evidence", project_root);
    fs::create_dir_all(&evidence_dir).unwrap();
    fs::write(format!("{}/test_evidence.txt", evidence_dir), "test").unwrap();
    
    // Uninstall without removing evidence
    let uninstaller = Uninstaller::new(&project_root);
    let result = uninstaller.uninstall(false, false, true);
    
    // Evidence should still exist
    assert!(Path::new(&evidence_dir).exists());
    assert!(Path::new(&format!("{}/test_evidence.txt", evidence_dir)).exists());
}

#[test]
fn test_uninstall_removes_services() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_string_lossy().to_string();
    let state_path = format!("{}/install_state.json", project_root);
    let keys_dir = format!("{}/keys", project_root);
    
    // Create install state
    let state_manager = InstallStateManager::new(&state_path, &keys_dir);
    let crypto_manager = CryptoIdentityManager::new(&keys_dir);
    let identity = crypto_manager.generate().unwrap();
    let retention = RetentionPolicy::default();
    let state = state_manager.create(
        true,
        Some(chrono::Utc::now()),
        retention,
        identity,
        "1.0.0",
    ).unwrap();
    
    // Create systemd service file
    let systemd_dir = format!("{}/systemd", project_root);
    fs::create_dir_all(&systemd_dir).unwrap();
    fs::write(format!("{}/ransomeye-core.service", systemd_dir), "test").unwrap();
    
    // Uninstall
    let uninstaller = Uninstaller::new(&project_root);
    let cleanup_options = CleanupOptions {
        remove_services: true,
        remove_configs: true,
        remove_evidence: false,
        secure_delete: false,
    };
    
    let cleanup_manager = CleanupManager::new(&project_root);
    let cleanup_log = cleanup_manager.cleanup(&state, &cleanup_options).unwrap();
    
    // Service should be removed
    assert!(cleanup_log.removed_services.contains(&"ransomeye-core.service".to_string()));
}

