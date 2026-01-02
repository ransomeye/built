// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/tests/fs_behavior_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Filesystem behavior monitoring tests

use ransomeye_linux_agent::filesystem::{FilesystemMonitor, FilesystemEventType};

#[test]
fn test_rename_event() {
    let monitor = FilesystemMonitor::new(1000);
    
    let event = monitor.record_rename(
        1234,
        1000,
        1000,
        "/tmp/old.txt".to_string(),
        "/tmp/new.txt".to_string(),
    ).unwrap();
    
    assert_eq!(event.event_type, FilesystemEventType::Rename);
    assert_eq!(event.pid, 1234);
    assert_eq!(event.path, "/tmp/new.txt");
    assert_eq!(event.old_path, Some("/tmp/old.txt".to_string()));
    assert_eq!(event.new_path, Some("/tmp/new.txt".to_string()));
}

#[test]
fn test_unlink_event() {
    let monitor = FilesystemMonitor::new(1000);
    
    let event = monitor.record_unlink(
        1234,
        1000,
        1000,
        "/tmp/file.txt".to_string(),
    ).unwrap();
    
    assert_eq!(event.event_type, FilesystemEventType::Unlink);
    assert_eq!(event.path, "/tmp/file.txt");
}

#[test]
fn test_chmod_event() {
    let monitor = FilesystemMonitor::new(1000);
    
    let event = monitor.record_chmod(
        1234,
        1000,
        1000,
        "/tmp/file.txt".to_string(),
        0o755,
    ).unwrap();
    
    assert_eq!(event.event_type, FilesystemEventType::Chmod);
    assert_eq!(event.mode, Some(0o755));
}

#[test]
fn test_mass_write_detection() {
    let monitor = FilesystemMonitor::new(1000);
    
    // Write below threshold
    for _ in 0..999 {
        let result = monitor.record_write(1234, 1000, 1000, "/tmp/file.txt".to_string()).unwrap();
        assert!(result.is_none());
    }
    
    // Write at threshold (should trigger mass write event)
    let result = monitor.record_write(1234, 1000, 1000, "/tmp/file.txt".to_string()).unwrap();
    assert!(result.is_some());
    
    let event = result.unwrap();
    assert_eq!(event.event_type, FilesystemEventType::MassWrite);
    assert_eq!(event.write_count, Some(1000));
}

