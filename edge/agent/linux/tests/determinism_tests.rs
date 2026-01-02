// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/tests/determinism_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Determinism tests for Linux Agent

use ransomeye_linux_agent::process::{ProcessMonitor, ProcessEventType};
use ransomeye_linux_agent::filesystem::FilesystemMonitor;
use ransomeye_linux_agent::features::FeatureExtractor;

#[test]
fn test_process_event_determinism() {
    let monitor = ProcessMonitor::new(1000);
    
    let event1 = monitor.record_exec(
        1234,
        Some(1000),
        1000,
        1000,
        "/usr/bin/test".to_string(),
        Some("test --arg".to_string()),
    ).unwrap();
    
    let event2 = monitor.record_exec(
        1234,
        Some(1000),
        1000,
        1000,
        "/usr/bin/test".to_string(),
        Some("test --arg".to_string()),
    ).unwrap();
    
    // Same inputs should produce same event structure
    assert_eq!(event1.event_type, event2.event_type);
    assert_eq!(event1.pid, event2.pid);
    assert_eq!(event1.executable, event2.executable);
}

#[test]
fn test_filesystem_event_determinism() {
    let monitor = FilesystemMonitor::new(1000);
    
    let event1 = monitor.record_rename(
        1234,
        1000,
        1000,
        "/tmp/old.txt".to_string(),
        "/tmp/new.txt".to_string(),
    ).unwrap();
    
    let event2 = monitor.record_rename(
        1234,
        1000,
        1000,
        "/tmp/old.txt".to_string(),
        "/tmp/new.txt".to_string(),
    ).unwrap();
    
    // Same inputs should produce same event structure
    assert_eq!(event1.event_type, event2.event_type);
    assert_eq!(event1.path, event2.path);
    assert_eq!(event1.old_path, event2.old_path);
}

#[test]
fn test_feature_extraction_determinism() {
    let extractor = FeatureExtractor::new();
    let monitor = ProcessMonitor::new(1000);
    
    let event = monitor.record_exec(
        1234,
        Some(1000),
        1000,
        1000,
        "/usr/bin/test".to_string(),
        None,
    ).unwrap();
    
    let features1 = extractor.extract_from_process(&event).unwrap();
    let features2 = extractor.extract_from_process(&event).unwrap();
    
    // Same event should produce same features
    assert_eq!(features1.pid, features2.pid);
    assert_eq!(features1.uid, features2.uid);
    assert_eq!(features1.event_type, features2.event_type);
    assert_eq!(features1.process_activity, features2.process_activity);
}

