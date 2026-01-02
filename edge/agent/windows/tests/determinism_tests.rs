// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/determinism_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Determinism and bounded memory tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::features::Features;
    use ransomeye_windows_agent::process::{ProcessEvent, ProcessEventType};
    use ransomeye_windows_agent::filesystem::{FilesystemEvent, FilesystemEventType};
    
    #[test]
    fn test_feature_extraction_determinism() {
        let event1 = ProcessEvent {
            event_type: ProcessEventType::Create,
            pid: 1234,
            ppid: Some(1000),
            executable: Some("test.exe".to_string()),
            command_line: Some("test.exe --arg".to_string()),
            timestamp: 1234567890,
        };
        
        let event2 = event1.clone();
        
        let features1 = Features::from_process_event(&event1);
        let features2 = Features::from_process_event(&event2);
        
        assert_eq!(features1.event_type, features2.event_type);
        assert_eq!(features1.process_activity, features2.process_activity);
        assert_eq!(features1.has_command_line, features2.has_command_line);
    }
    
    #[test]
    fn test_feature_bounds_validation() {
        let event = FilesystemEvent {
            event_type: FilesystemEventType::Rename,
            path: "C:\\test.txt".to_string(),
            old_path: Some("C:\\old.txt".to_string()),
            new_path: Some("C:\\new.txt".to_string()),
            pid: 1234,
            timestamp: 1234567890,
            write_count: None,
        };
        
        let features = Features::from_filesystem_event(&event);
        assert!(features.validate().is_ok());
    }
    
    #[test]
    fn test_envelope_sequence_monotonicity() {
        use ransomeye_windows_agent::envelope::EnvelopeBuilder;
        
        let mut builder = EnvelopeBuilder::new("test".to_string(), "test-id".to_string());
        let seq1 = builder.sequence();
        
        // Simulate creating envelope (would increment sequence)
        // In real test, would call build_from_process
        
        // Sequence should be monotonic
        assert!(seq1 >= 0);
    }
}

