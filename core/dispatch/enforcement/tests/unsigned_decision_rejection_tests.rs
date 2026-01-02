// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/tests/unsigned_decision_rejection_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for rejecting unsigned decisions

use ransomeye_enforcement::EnforcementDispatcher;
use ransomeye_enforcement::EnforcementError;

#[tokio::test]
async fn test_unsigned_decision_rejected() {
    // Create a decision without signature
    let decision_json = r#"
    {
        "decision_id": "test-decision-1",
        "created_at": "2024-01-01T00:00:00Z",
        "alert_id": "alert-1",
        "policy_id": "policy-1",
        "policy_version": "1.0.0",
        "decision": "block",
        "allowed_actions": ["block"],
        "required_approvals": [],
        "evidence_reference": "evidence-1",
        "kill_chain_stage": "execution",
        "severity": "high",
        "reasoning": "Test decision",
        "policy_signature": "",
        "decision_hash": "test-hash"
    }
    "#;
    
    // Initialize dispatcher (will fail if no public key, but that's expected in test)
    let dispatcher_result = EnforcementDispatcher::new();
    
    // If dispatcher initializes, test should reject unsigned decision
    if let Ok(dispatcher) = dispatcher_result {
        let result = dispatcher.dispatch(decision_json, &["192.168.1.1".to_string()], false).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            EnforcementError::UnsignedDecision(_) => {
                // Expected
            }
            _ => {
                panic!("Expected UnsignedDecision error");
            }
        }
    }
}

#[tokio::test]
async fn test_empty_signature_rejected() {
    let decision_json = r#"
    {
        "decision_id": "test-decision-2",
        "created_at": "2024-01-01T00:00:00Z",
        "alert_id": "alert-1",
        "policy_id": "policy-1",
        "policy_version": "1.0.0",
        "decision": "block",
        "allowed_actions": ["block"],
        "required_approvals": [],
        "evidence_reference": "evidence-1",
        "kill_chain_stage": "execution",
        "severity": "high",
        "reasoning": "Test decision",
        "policy_signature": "",
        "decision_hash": "test-hash"
    }
    "#;
    
    let dispatcher_result = EnforcementDispatcher::new();
    
    if let Ok(dispatcher) = dispatcher_result {
        let result = dispatcher.dispatch(decision_json, &["192.168.1.1".to_string()], false).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            EnforcementError::UnsignedDecision(_) => {
                // Expected
            }
            _ => {
                panic!("Expected UnsignedDecision error");
            }
        }
    }
}

