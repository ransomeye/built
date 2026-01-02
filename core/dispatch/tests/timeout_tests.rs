// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/timeout_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for acknowledgment timeout handling

use ransomeye_dispatcher::TimeoutManager;
use ransomeye_dispatcher::acknowledgment_envelope::AcknowledgmentEnvelope;
use ransomeye_dispatcher::DispatcherError;

#[tokio::test]
async fn test_acknowledgment_timeout() {
    let timeout_manager = TimeoutManager::new(1); // 1 second timeout
    
    // Check function that never returns an acknowledgment
    let check_fn = || None;
    
    let result = timeout_manager.wait_for_acknowledgment("test-directive-1", None, check_fn).await;
    
    assert!(result.is_err());
    if let Err(DispatcherError::AcknowledgmentTimeout(_)) = result {
        // Expected
    } else {
        panic!("Expected AcknowledgmentTimeout error");
    }
}

#[tokio::test]
async fn test_acknowledgment_received_before_timeout() {
    let timeout_manager = TimeoutManager::new(5); // 5 second timeout
    
    // Use a simple closure that returns the ack immediately
    let ack = create_test_acknowledgment("test-directive-1");
    let check_fn = move || Some(ack.clone());
    
    let result = timeout_manager.wait_for_acknowledgment("test-directive-1", None, check_fn).await;
    
    // Should succeed if ack received
    assert!(result.is_ok());
}

fn create_test_acknowledgment(directive_id: &str) -> AcknowledgmentEnvelope {
    use ransomeye_dispatcher::acknowledgment_envelope::ExecutionResult;
    
    AcknowledgmentEnvelope::new(
        directive_id.to_string(),
        "agent-1".to_string(),
        ExecutionResult::Success,
        "test-signature".to_string(),
        "test-hash".to_string(),
    )
}
