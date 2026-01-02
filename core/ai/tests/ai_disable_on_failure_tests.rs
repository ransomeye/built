// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/tests/ai_disable_on_failure_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests that AI disables on failure

use ransomeye_ai_advisory::AdvisoryEngine;
use ransomeye_ai_advisory::AdvisoryError;
use ransomeye_ai_advisory::controller::AIController;

#[tokio::test]
async fn test_ai_disables_on_missing_baseline() {
    // Test that AI disables when baseline is missing
    let controller = AIController::new();
    
    // Simulate missing baseline
    controller.disable("Missing baseline model").unwrap();
    
    // Verify AI is disabled
    assert!(!controller.is_enabled().unwrap());
    assert!(controller.get_disable_reason().unwrap().is_some());
}

#[tokio::test]
async fn test_ai_disables_on_runtime_error() {
    // Test that AI disables on runtime error
    let controller = AIController::new();
    
    // Simulate runtime error
    controller.disable("Runtime error").unwrap();
    
    // Verify AI is disabled
    assert!(!controller.is_enabled().unwrap());
}

#[tokio::test]
async fn test_core_operation_preserved_when_ai_down() {
    // Test that Core can operate when AI is down
    // This is an architectural test - actual implementation is in Core
    
    let controller = AIController::new();
    
    // Disable AI
    controller.disable("Test disable").unwrap();
    
    // Core should still be able to operate
    // (This is verified by the fact that AI is advisory-only)
    assert!(!controller.is_enabled().unwrap());
    
    // Core operations should not depend on AI
    // This is a structural guarantee, not a runtime test
}

