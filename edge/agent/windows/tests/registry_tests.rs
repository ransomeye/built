// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/tests/registry_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Registry monitoring tests

#[cfg(test)]
mod tests {
    use ransomeye_windows_agent::registry::{RegistryMonitor, RegistryEventType};
    
    #[test]
    fn test_registry_monitor_creation() {
        let monitor = RegistryMonitor::new(vec![]);
        assert_eq!(monitor.events_processed(), 0);
    }
    
    #[test]
    fn test_autorun_key_detection() {
        let monitor = RegistryMonitor::new(vec![]);
        let event = monitor.get_key_create_event(
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run\\Test".to_string(),
            1234,
        ).unwrap();
        
        assert_eq!(event.event_type, RegistryEventType::AutorunCreate);
    }
    
    #[test]
    fn test_persistence_key_detection() {
        let monitor = RegistryMonitor::new(vec![]);
        let event = monitor.get_value_set_event(
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            "TestApp".to_string(),
            "C:\\test.exe".to_string(),
            1234,
        ).unwrap();
        
        assert_eq!(event.event_type, RegistryEventType::AutorunCreate);
    }
    
    #[test]
    fn test_registry_value_set() {
        let monitor = RegistryMonitor::new(vec![]);
        let event = monitor.get_value_set_event(
            "HKEY_CURRENT_USER\\SOFTWARE\\Test".to_string(),
            "ValueName".to_string(),
            "ValueData".to_string(),
            1234,
        ).unwrap();
        
        assert_eq!(event.event_type, RegistryEventType::ValueSet);
        assert_eq!(event.value_name, Some("ValueName".to_string()));
        assert_eq!(event.value_data, Some("ValueData".to_string()));
    }
}

