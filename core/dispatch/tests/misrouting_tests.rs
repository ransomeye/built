// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/misrouting_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for misrouting rejection - wrong agent/platform

use ransomeye_dispatcher::directive_envelope::{DirectiveEnvelope, TargetScope, AuditReceipt};
use ransomeye_dispatcher::{TargetRouter, AgentInfo};
use ransomeye_dispatcher::DispatcherError;
use chrono::Utc;

#[test]
fn test_wrong_platform_rejected() {
    let router = TargetRouter::new();
    
    // Register Linux agent
    router.register_agent(AgentInfo {
        agent_id: "linux-agent-1".to_string(),
        platform: "linux".to_string(),
        capabilities: vec!["block".to_string()],
        api_url: "http://localhost:8080".to_string(),
        asset_class: None,
        environment: None,
    });
    
    // Try to route Windows directive to Linux agent
    let directive = create_directive_with_platform("windows".to_string());
    
    let result = router.resolve_targets(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::TargetResolutionFailed(_) => {}
        _ => panic!("Expected TargetResolutionFailed error"),
    }
}

#[test]
fn test_wrong_capability_rejected() {
    let router = TargetRouter::new();
    
    // Register agent without "isolate" capability
    router.register_agent(AgentInfo {
        agent_id: "agent-1".to_string(),
        platform: "linux".to_string(),
        capabilities: vec!["block".to_string(), "monitor".to_string()],
        api_url: "http://localhost:8080".to_string(),
        asset_class: None,
        environment: None,
    });
    
    // Try to route "isolate" action
    let directive = create_directive_with_action("isolate".to_string());
    
    let result = router.resolve_targets(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::AgentCapabilityMismatch(_) => {}
        _ => panic!("Expected AgentCapabilityMismatch error"),
    }
}

#[test]
fn test_nonexistent_agent_rejected() {
    let router = TargetRouter::new();
    
    // Try to route to non-existent agent
    let directive = create_directive_with_agent_ids(vec!["nonexistent-agent".to_string()]);
    
    let result = router.resolve_targets(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::AgentNotFound(_) => {}
        _ => panic!("Expected AgentNotFound error"),
    }
}

fn create_directive_with_platform(platform: String) -> DirectiveEnvelope {
    DirectiveEnvelope::new(
        "test-policy-1".to_string(),
        "1.0.0".to_string(),
        "test-signature".to_string(),
        "test-hash".to_string(),
        3600,
        uuid::Uuid::new_v4().to_string(),
        TargetScope {
            agent_ids: None,
            host_addresses: None,
            platform: Some(platform),
            asset_class: None,
            environment: None,
        },
        "block".to_string(),
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

fn create_directive_with_agent_ids(agent_ids: Vec<String>) -> DirectiveEnvelope {
    DirectiveEnvelope::new(
        "test-policy-1".to_string(),
        "1.0.0".to_string(),
        "test-signature".to_string(),
        "test-hash".to_string(),
        3600,
        uuid::Uuid::new_v4().to_string(),
        TargetScope {
            agent_ids: Some(agent_ids),
            host_addresses: None,
            platform: None,
            asset_class: None,
            environment: None,
        },
        "block".to_string(),
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
