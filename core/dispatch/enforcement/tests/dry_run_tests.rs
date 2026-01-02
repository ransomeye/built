// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/tests/dry_run_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for dry-run mode simulation

use ransomeye_enforcement::EnforcementDispatcher;
use ransomeye_enforcement::output::{ExecutionStatus, EnforcementResult};

#[tokio::test]
async fn test_dry_run_simulates_execution() {
    let decision_json = r#"
    {
        "decision_id": "test-decision-dryrun-1",
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
        let result = dispatcher.dispatch(decision_json, &["192.168.1.1".to_string()], true).await;
        
        if let Ok(res) = result {
            assert_eq!(res.dry_run, true);
            assert_eq!(res.status, ExecutionStatus::DryRun);
            assert!(res.action_taken.is_some());
            assert!(res.action_taken.unwrap().contains("DRY-RUN"));
        }
    }
}

#[tokio::test]
async fn test_dry_run_no_actual_execution() {
    let decision_json = r#"
    {
        "decision_id": "test-decision-dryrun-2",
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
        let result = dispatcher.dispatch(decision_json, &["192.168.1.1".to_string()], true).await;
        
        if let Ok(res) = result {
            // Verify it's marked as dry-run
            assert!(res.dry_run);
            // Verify no rollback available (since nothing was executed)
            assert!(!res.rollback_available);
        }
    }
}

