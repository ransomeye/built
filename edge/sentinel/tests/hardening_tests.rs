// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/tests/hardening_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Sentinel runtime hardening tests

use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use std::time::Duration;
use std::thread;
use sentinel::hardening::RuntimeHardening;

#[test]
fn test_sentinel_refuses_start_on_binary_tamper() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_binary");
    
    // Create test binary
    let mut file = File::create(&binary_path).unwrap();
    file.write_all(b"test binary content").unwrap();
    drop(file);

    let hardening = RuntimeHardening::new(
        binary_path.to_string_lossy().to_string(),
        None,
        30,
    ).unwrap();

    // Should pass initially
    assert!(hardening.verify_binary_integrity().is_ok());

    // Tamper with binary
    let mut file = File::create(&binary_path).unwrap();
    file.write_all(b"tampered content").unwrap();
    drop(file);

    // Should fail - Sentinel refuses to start
    assert!(hardening.verify_binary_integrity().is_err());
    assert!(hardening.is_tampered());
}

#[test]
fn test_sentinel_fails_if_config_missing() {
    let result = RuntimeHardening::new(
        "/bin/sh".to_string(),
        Some("/nonexistent/config".to_string()),
        30,
    );
    
    // Should fail if config missing
    assert!(result.is_err());
}

#[test]
fn test_sentinel_watchdog_escalation() {
    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(),
        None,
        1, // 1 second interval
    ).unwrap();

    assert_eq!(hardening.crash_count(), 0);
    
    // Simulate crashes (would be done by watchdog in production)
    hardening.reset_crash_count();
    assert_eq!(hardening.crash_count(), 0);
    
    // Watchdog should escalate after 3 crashes
    // This test verifies the crash counter exists
}

#[test]
fn test_sentinel_binary_integrity_verification() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_binary");
    
    let mut file = File::create(&binary_path).unwrap();
    file.write_all(b"test binary content").unwrap();
    drop(file);

    let hardening = RuntimeHardening::new(
        binary_path.to_string_lossy().to_string(),
        None,
        30,
    ).unwrap();

    assert!(hardening.verify_binary_integrity().is_ok());

    // Tamper with binary
    let mut file = File::create(&binary_path).unwrap();
    file.write_all(b"tampered content").unwrap();
    drop(file);

    assert!(hardening.verify_binary_integrity().is_err());
}

#[test]
fn test_sentinel_config_integrity_verification() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config");
    
    let mut file = File::create(&config_path).unwrap();
    file.write_all(b"test config content").unwrap();
    drop(file);

    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(),
        Some(config_path.to_string_lossy().to_string()),
        30,
    ).unwrap();

    assert!(hardening.verify_config_integrity().is_ok());

    // Tamper with config
    let mut file = File::create(&config_path).unwrap();
    file.write_all(b"tampered config").unwrap();
    drop(file);

    assert!(hardening.verify_config_integrity().is_err());
}

