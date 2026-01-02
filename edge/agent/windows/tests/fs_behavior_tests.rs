// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/fs_behavior_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Filesystem behavior monitoring tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::filesystem::{FilesystemMonitor, FilesystemEventType};
    
    #[test]
    fn test_filesystem_monitor_creation() {
        let monitor = FilesystemMonitor::new(1000, 50);
        assert_eq!(monitor.events_processed(), 0);
    }
    
    #[test]
    fn test_rename_event() {
        let monitor = FilesystemMonitor::new(1000, 50);
        let event = monitor.get_rename_event(
            "C:\\old.txt".to_string(),
            "C:\\new.txt".to_string(),
            1234,
        ).unwrap();
        
        assert_eq!(event.event_type, FilesystemEventType::Rename);
        assert_eq!(event.path, "C:\\new.txt");
        assert_eq!(event.old_path, Some("C:\\old.txt".to_string()));
    }
    
    #[test]
    fn test_delete_event() {
        let monitor = FilesystemMonitor::new(1000, 50);
        let event = monitor.get_delete_event("C:\\test.txt".to_string(), 1234).unwrap();
        
        assert_eq!(event.event_type, FilesystemEventType::Delete);
        assert_eq!(event.path, "C:\\test.txt");
    }
    
    #[test]
    fn test_mass_write_detection() {
        let monitor = FilesystemMonitor::new(1000, 10);
        
        // Simulate many writes to same path
        for i in 0..15 {
            let result = monitor.track_write(format!("C:\\test{}.txt", i), 1234).unwrap();
            if i >= 9 {
                // Should detect mass write after threshold
                assert!(result.is_some());
            }
        }
    }
}

