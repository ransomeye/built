// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/fail_safe_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Fail-safe behavior tests - validation failure = NO ACTION

use ransomeye_dispatcher::SafetyGuards;
use ransomeye_dispatcher::directive_envelope::{DirectiveEnvelope, TargetScope, AuditReceipt};
use ransomeye_dispatcher::DispatcherError;
use chrono::Utc;

#[test]
fn test_denied_action_rejected() {
    let guards = SafetyGuards::new(100, 60, 1000);
    
    let directive = create_directive_with_action("delete".to_string()); // Denied action
    
    let result = guards.check(&directive);
    assert!(result.is_err());
    if let Err(DispatcherError::InvalidDirective(msg)) = result {
        assert!(msg.contains("denylist") || msg.contains("delete"));
    } else {
        panic!("Expected InvalidDirective error with denylist message");
    }
}

#[test]
fn test_unallowed_action_rejected() {
    let guards = SafetyGuards::new(100, 60, 1000);
    
    let mut directive = create_test_directive();
    directive.action = "unknown_action".to_string(); // Not in allowlist
    
    let result = guards.check(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::InvalidDirective(msg) => {
            assert!(msg.contains("allowlist"));
        },
        _ => panic!("Expected InvalidDirective error"),
    }
}

#[test]
fn test_rate_limit_enforced() {
    let guards = SafetyGuards::new(2, 60, 1000); // Max 2 actions per 60 seconds
    
    let directive = create_test_directive();
    
    // First action - should pass
    assert!(guards.check(&directive).is_ok());
    
    // Second action - should pass
    assert!(guards.check(&directive).is_ok());
    
    // Third action - should fail (rate limit exceeded)
    let result = guards.check(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::InvalidDirective(msg) => {
            assert!(msg.contains("rate limit"));
        },
        _ => panic!("Expected InvalidDirective error for rate limit"),
    }
}

#[test]
fn test_allowed_action_passes() {
    let guards = SafetyGuards::new(100, 60, 1000);
    
    let directive = create_directive_with_action("block".to_string()); // Allowed action
    
    // Should pass safety checks
    assert!(guards.check(&directive).is_ok());
}

fn create_directive_with_action(action: String) -> DirectiveEnvelope {
    DirectiveEnvelope::new(
        "test-policy-1".to_string(),
        "1.0.0".to_string(),
        "test-signature".to_string(),
        "test-hash".to_string(),
        3600,
        uuid::Uuid::new_v4().to_string(),
        TargetScope {
            agent_ids: Some(vec!["agent-1".to_string()]),
            host_addresses: None,
            platform: None,
            asset_class: None,
            environment: None,
        },
        action,
        "precondition-hash".to_string(),
        AuditReceipt {
            receipt_id: "receipt-1".to_string(),
            receipt_signature: "receipt-sig".to_string(),
            receipt_hash: "receipt-hash".to_string(),
            receipt_timestamp: Utc::now(),
        },
        vec!["block".to_string()],
        vec![],
        "evidence-1".to_string(),
        "execution".to_string(),
        "high".to_string(),
        "Test".to_string(),
    )
}

fn create_test_directive() -> DirectiveEnvelope {
    DirectiveEnvelope {
        directive_id: uuid::Uuid::new_v4().to_string(),
        policy_id: "test_policy".to_string(),
        policy_version: "1.0".to_string(),
        signature: "test_signature".to_string(),
        signature_hash: "test_hash".to_string(),
        issued_at: Utc::now(),
        ttl_seconds: 3600,
        nonce: uuid::Uuid::new_v4().to_string(),
        target_scope: TargetScope {
            agent_ids: Some(vec!["agent1".to_string()]),
            host_addresses: None,
            platform: Some("linux".to_string()),
            asset_class: None,
            environment: None,
        },
        action: "block".to_string(),
        preconditions_hash: "preconditions_hash".to_string(),
        audit_receipt: AuditReceipt {
            receipt_id: "receipt1".to_string(),
            receipt_signature: "receipt_sig".to_string(),
            receipt_hash: "receipt_hash".to_string(),
            receipt_timestamp: Utc::now(),
        },
        allowed_actions: vec!["block".to_string()],
        required_approvals: vec![],
        evidence_reference: "evidence1".to_string(),
        kill_chain_stage: "execution".to_string(),
        severity: "high".to_string(),
        reasoning: "test reasoning".to_string(),
    }
}

