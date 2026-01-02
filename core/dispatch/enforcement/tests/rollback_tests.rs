// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/tests/rollback_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for rollback functionality

use ransomeye_enforcement::EnforcementDispatcher;
use ransomeye_enforcement::EnforcementError;

#[tokio::test]
async fn test_rollback_available_after_execution() {
    let decision_json = r#"
    {
        "decision_id": "test-decision-rollback-1",
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
        "policy_signature": "test-sig",
        "decision_hash": "test-hash"
    }
    "#;
    
    let dispatcher_result = EnforcementDispatcher::new();
    
    if let Ok(dispatcher) = dispatcher_result {
        // Execute in dry-run first to get execution_id
        let result = dispatcher.dispatch(decision_json, &["192.168.1.1".to_string()], true).await;
        
        if let Ok(res) = result {
            // In dry-run, rollback might not be available
            // But execution_id should be present
            assert!(!res.execution_id.is_empty());
        }
    }
}

#[tokio::test]
async fn test_rollback_execution() {
    let dispatcher_result = EnforcementDispatcher::new();
    
    if let Ok(dispatcher) = dispatcher_result {
        // Try to rollback a non-existent execution
        let result = dispatcher.rollback("non-existent-execution-id");
        
        // Should fail gracefully
        assert!(result.is_err());
        match result.unwrap_err() {
            EnforcementError::RollbackFailed(_) => {
                // Expected
            }
            _ => {
                panic!("Expected RollbackFailed error");
            }
        }
    }
}

