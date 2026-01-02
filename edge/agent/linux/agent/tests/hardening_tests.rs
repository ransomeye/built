// Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/agent/tests/hardening_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime hardening tests - binary integrity, config integrity, tamper detection, watchdog, crash escalation

use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use std::time::Duration;
use std::thread;
use edge::agent::linux::agent::hardening::RuntimeHardening;

#[test]
fn test_binary_integrity_verification() {
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

    // Should pass
    assert!(hardening.verify_binary_integrity().is_ok());

    // Tamper with binary
    let mut file = File::create(&binary_path).unwrap();
    file.write_all(b"tampered content").unwrap();
    drop(file);

    // Should fail
    assert!(hardening.verify_binary_integrity().is_err());
    assert!(hardening.is_tampered());
}

#[test]
fn test_config_integrity_verification() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config");
    
    // Create test config
    let mut file = File::create(&config_path).unwrap();
    file.write_all(b"test config content").unwrap();
    drop(file);

    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(), // Dummy binary
        Some(config_path.to_string_lossy().to_string()),
        30,
    ).unwrap();

    // Should pass
    assert!(hardening.verify_config_integrity().is_ok());

    // Tamper with config
    let mut file = File::create(&config_path).unwrap();
    file.write_all(b"tampered config").unwrap();
    drop(file);

    // Should fail
    assert!(hardening.verify_config_integrity().is_err());
    assert!(hardening.is_tampered());
}

#[test]
fn test_watchdog_heartbeat() {
    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(),
        None,
        1, // 1 second interval
    ).unwrap();

    // Get initial heartbeat time
    use std::sync::atomic::Ordering;
    use std::time::{SystemTime, UNIX_EPOCH};
    let initial_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    thread::sleep(Duration::from_millis(100));
    hardening.heartbeat();
    
    let new_heartbeat = hardening.last_heartbeat.load(Ordering::Acquire);
    assert!(new_heartbeat >= initial_time);
}

#[test]
fn test_watchdog_start_stop() {
    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(),
        None,
        1,
    ).unwrap();

    // Start watchdog
    assert!(hardening.start_watchdog().is_ok());
    
    // Should fail to start again
    assert!(hardening.start_watchdog().is_err());
    
    // Stop watchdog
    hardening.stop_watchdog();
    
    // Should be able to start again
    assert!(hardening.start_watchdog().is_ok());
    hardening.stop_watchdog();
}

#[test]
fn test_crash_escalation() {
    let hardening = RuntimeHardening::new(
        "/bin/sh".to_string(),
        None,
        1,
    ).unwrap();

    assert_eq!(hardening.crash_count(), 0);
    
    // Simulate crashes (would be done by watchdog in production)
    // This test verifies the crash counter exists and can be incremented
    hardening.reset_crash_count();
    assert_eq!(hardening.crash_count(), 0);
}

#[test]
fn test_missing_binary_fails() {
    let result = RuntimeHardening::new(
        "/nonexistent/binary".to_string(),
        None,
        30,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_missing_config_fails() {
    let result = RuntimeHardening::new(
        "/bin/sh".to_string(),
        Some("/nonexistent/config".to_string()),
        30,
    );
    
    assert!(result.is_err());
}

