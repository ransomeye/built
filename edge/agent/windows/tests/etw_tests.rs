// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/etw_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: ETW session and event processing tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::etw::{EtwSession, EtwEvent, EtwEventType, EtwEventData};
    
    #[test]
    fn test_etw_session_creation() {
        let session = EtwSession::new();
        assert!(session.is_ok());
    }
    
    #[test]
    fn test_etw_session_start_stop() {
        let mut session = EtwSession::new().unwrap();
        assert!(!session.is_running());
        
        assert!(session.start().is_ok());
        assert!(session.is_running());
        
        assert!(session.stop().is_ok());
        assert!(!session.is_running());
    }
    
    #[test]
    fn test_etw_event_creation() {
        let event = EtwEvent {
            event_type: EtwEventType::ProcessStart,
            timestamp: 1234567890,
            pid: 1234,
            tid: 5678,
            data: EtwEventData::Process {
                image_name: "test.exe".to_string(),
                command_line: Some("test.exe --arg".to_string()),
                ppid: Some(1000),
            },
        };
        
        assert_eq!(event.event_type, EtwEventType::ProcessStart);
        assert_eq!(event.pid, 1234);
    }
}

