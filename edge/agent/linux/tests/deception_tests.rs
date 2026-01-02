// Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/tests/deception_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Comprehensive tests for deception engine - honeyfiles, honeycredentials, fake services, allowlist bypass

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;
use crossbeam_channel::bounded;
use agent_linux::deception::{DeceptionEngine, DeceptionError};
use agent_linux::event::AgentEvent;

#[tokio::test]
async fn test_honeyfile_read_alert() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate honeyfiles
    let honeyfiles = engine.generate_honeyfiles(1).unwrap();
    assert!(!honeyfiles.is_empty());
    
    let honeyfile = &honeyfiles[0];
    
    // Simulate file read
    let result = engine.check_file_access(
        &honeyfile.path,
        "read",
        1234,
        "attacker_process",
        1000,
    );
    
    assert!(result.is_ok());
    
    // Check that event was sent
    let event = _rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
    
    match event {
        AgentEvent::File(file_event) => {
            assert_eq!(file_event.operation, "HoneyfileRead");
            assert!(file_event.path.contains(&honeyfile.id) || file_event.path.contains(&honeyfile.name));
        }
        _ => panic!("Expected File event"),
    }
}

#[tokio::test]
async fn test_honeyfile_delete_critical() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate honeyfiles
    let honeyfiles = engine.generate_honeyfiles(1).unwrap();
    let honeyfile = &honeyfiles[0];
    
    // Simulate file delete
    let result = engine.check_file_access(
        &honeyfile.path,
        "delete",
        1234,
        "attacker_process",
        1000,
    );
    
    assert!(result.is_ok());
    
    // Check that CRITICAL event was sent
    let event = _rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
    
    match event {
        AgentEvent::File(file_event) => {
            assert_eq!(file_event.operation, "HoneyfileDelete");
        }
        _ => panic!("Expected File event"),
    }
}

#[tokio::test]
async fn test_honeycredential_use_critical() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate honeycredentials
    let honeycreds = engine.generate_honeycredentials(1).unwrap();
    assert!(!honeycreds.is_empty());
    
    let honeycred = &honeycreds[0];
    
    // Simulate credential use
    let result = engine.check_credential_use(
        &honeycred.location,
        1234,
        "attacker_process",
        1000,
    );
    
    assert!(result.is_ok());
    
    // Check that CRITICAL event was sent
    let event = _rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
    
    match event {
        AgentEvent::Auth(auth_event) => {
            assert_eq!(auth_event.event_type, "deception");
            assert!(!auth_event.success); // Always false for honeycredentials
        }
        _ => panic!("Expected Auth event"),
    }
}

#[tokio::test]
async fn test_fake_service_connection_alert() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate fake services
    let fake_services = engine.generate_fake_services(1).unwrap();
    assert!(!fake_services.is_empty());
    
    let fake_service = &fake_services[0];
    
    // Simulate service connection
    let result = engine.check_service_connection(
        fake_service.port,
        &fake_service.protocol,
        "192.168.1.100",
        1234,
        "attacker_process",
    );
    
    assert!(result.is_ok());
    
    // Check that event was sent
    let event = _rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
    
    match event {
        AgentEvent::Network(network_event) => {
            assert_eq!(network_event.event_type, "deception");
        }
        _ => panic!("Expected Network event"),
    }
}

#[tokio::test]
async fn test_allowlisted_process_bypass() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    // Set allowlist environment variable
    std::env::set_var("RANSOMEYE_DECEPTION_ALLOWLIST", "/usr/bin/legitimate_process");
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate honeyfiles
    let honeyfiles = engine.generate_honeyfiles(1).unwrap();
    let honeyfile = &honeyfiles[0];
    
    // Simulate file access by allowlisted process
    let result = engine.check_file_access(
        &honeyfile.path,
        "read",
        1234,
        "/usr/bin/legitimate_process",
        1000,
    );
    
    assert!(result.is_ok());
    
    // Check that NO event was sent (allowlisted process bypasses deception)
    assert!(_rx.try_recv().is_err());
    
    // Cleanup
    std::env::remove_var("RANSOMEYE_DECEPTION_ALLOWLIST");
}

#[tokio::test]
async fn test_deception_artifacts_not_visible_to_legitimate() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _rx) = bounded(1000);
    let host_id = "test_host".to_string();
    
    // Set allowlist
    std::env::set_var("RANSOMEYE_DECEPTION_ALLOWLIST", "/usr/bin/legitimate_process,/usr/bin/systemd");
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    // Generate all deception artifacts
    engine.generate_honeyfiles(5).unwrap();
    engine.generate_honeycredentials(3).unwrap();
    engine.generate_fake_services(2).unwrap();
    
    // Legitimate processes should not trigger alerts
    let legitimate_processes = vec![
        "/usr/bin/legitimate_process",
        "/usr/bin/systemd",
    ];
    
    for process in legitimate_processes {
        // Try to access honeyfile
        let honeyfiles = engine.get_honeyfiles();
        if !honeyfiles.is_empty() {
            let result = engine.check_file_access(
                &honeyfiles[0].path,
                "read",
                1234,
                process,
                1000,
            );
            assert!(result.is_ok());
        }
        
        // Check no events were sent
        assert!(_rx.try_recv().is_err());
    }
    
    // Cleanup
    std::env::remove_var("RANSOMEYE_DECEPTION_ALLOWLIST");
}

#[test]
fn test_deception_engine_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path());
    assert!(engine.is_ok());
}

#[test]
fn test_honeyfile_generation() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    let honeyfiles = engine.generate_honeyfiles(5).unwrap();
    assert_eq!(honeyfiles.len(), 5);
    
    // Verify files exist
    for honeyfile in &honeyfiles {
        assert!(honeyfile.path.exists());
    }
}

#[test]
fn test_honeycredential_generation() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    let honeycreds = engine.generate_honeycredentials(3).unwrap();
    assert_eq!(honeycreds.len(), 3);
    
    // Verify all have fake values
    for honeycred in &honeycreds {
        assert!(honeycred.fake_value.starts_with("FAKE_"));
    }
}

#[test]
fn test_fake_service_generation() {
    let temp_dir = TempDir::new().unwrap();
    let (tx, _) = bounded(1000);
    let host_id = "test_host".to_string();
    
    let engine = DeceptionEngine::new(tx, host_id, temp_dir.path()).unwrap();
    
    let fake_services = engine.generate_fake_services(2).unwrap();
    assert_eq!(fake_services.len(), 2);
}

