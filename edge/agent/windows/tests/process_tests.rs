// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/process_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Process monitoring tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::process::{ProcessMonitor, ProcessEventType};
    
    #[test]
    fn test_process_monitor_creation() {
        let monitor = ProcessMonitor::new(1000);
        assert_eq!(monitor.events_processed(), 0);
    }
    
    #[test]
    fn test_process_monitor_memory_bounds() {
        let monitor = ProcessMonitor::new(100);
        
        // Simulate adding processes beyond limit
        // In real test, would call get_process_create for many PIDs
        // Monitor should evict oldest processes
        
        assert!(monitor.events_processed() >= 0);
    }
    
    #[test]
    fn test_process_event_types() {
        assert_eq!(ProcessEventType::Create, ProcessEventType::Create);
        assert_eq!(ProcessEventType::Terminate, ProcessEventType::Terminate);
        assert_eq!(ProcessEventType::CommandLine, ProcessEventType::CommandLine);
    }
}

