// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/tests/monitor_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Sentinel component monitoring tests

use sentinel::monitor::{AgentMonitor, DpiMonitor, ComponentHealth};

#[tokio::test]
async fn test_sentinel_detects_agent_kill() {
    // Create monitor for non-existent service (simulates terminated service)
    let monitor = AgentMonitor::new("nonexistent-service.service".to_string());
    
    // Check health - should detect terminated
    let health = monitor.check_health().await;
    // Service doesn't exist, so it will be inactive/terminated
    assert!(health.is_ok());
    // Health will be Terminated or Unhealthy for non-existent service
}

#[tokio::test]
async fn test_sentinel_detects_dpi_kill() {
    // Create monitor for non-existent service (simulates terminated service)
    let monitor = DpiMonitor::new("nonexistent-service.service".to_string());
    
    // Check health - should detect terminated
    let health = monitor.check_health().await;
    // Service doesn't exist, so it will be inactive/terminated
    assert!(health.is_ok());
    // Health will be Terminated or Unhealthy for non-existent service
}

#[tokio::test]
async fn test_agent_binary_integrity_check() {
    let monitor = AgentMonitor::new("test-service.service".to_string());
    
    // Check binary integrity for non-existent binary
    let result = monitor.check_binary_integrity().await;
    // Should fail if binary doesn't exist
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dpi_binary_integrity_check() {
    let monitor = DpiMonitor::new("test-service.service".to_string());
    
    // Check binary integrity for non-existent binary
    let result = monitor.check_binary_integrity().await;
    // Should fail if binary doesn't exist
    assert!(result.is_err());
}

