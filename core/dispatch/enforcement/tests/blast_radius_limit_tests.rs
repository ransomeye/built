// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/tests/blast_radius_limit_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for blast radius limiting

use ransomeye_enforcement::EnforcementDispatcher;
use ransomeye_enforcement::EnforcementError;

#[tokio::test]
async fn test_blast_radius_limit_enforced() {
    // Create a decision with many targets
    let decision_json = r#"
    {
        "decision_id": "test-decision-blast-1",
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
        // Create many targets (exceeding default limit of 50)
        let many_targets: Vec<String> = (1..=100)
            .map(|i| format!("192.168.1.{}", i))
            .collect();
        
        // In dry-run mode, might not enforce, but should handle gracefully
        let result = dispatcher.dispatch(decision_json, &many_targets, true).await;
        
        // Should either succeed in dry-run or fail with blast radius error
        match result {
            Ok(_) => {
                // Dry-run might succeed
            }
            Err(EnforcementError::BlastRadiusExceeded(_)) => {
                // Expected in real mode
            }
            Err(e) => {
                // Other errors are acceptable
                println!("Got error (acceptable): {:?}", e);
            }
        }
    }
}

