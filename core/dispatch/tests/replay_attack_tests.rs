// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/replay_attack_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for replay attack protection

use ransomeye_dispatcher::directive_envelope::DirectiveEnvelope;
use ransomeye_dispatcher::{NonceTracker, ReplayProtector};
use ransomeye_dispatcher::replay::ReplayGuard;
use ransomeye_dispatcher::DispatcherError;

#[test]
fn test_duplicate_directive_id_rejected() {
    let replay_protector = ReplayProtector::new(10000);
    let nonce_tracker = NonceTracker::new(3600);
    let guard = ReplayGuard::new(replay_protector, nonce_tracker);
    
    let directive1 = create_test_directive();
    let directive_id = directive1.directive_id.clone();
    
    // First check should pass
    assert!(guard.check_replay(&directive1).is_ok());
    
    // Second check with same directive ID should fail
    let directive2 = create_test_directive_with_id(directive_id);
    let result = guard.check_replay(&directive2);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::ReplayDetected(_) => {}
        _ => panic!("Expected ReplayDetected error"),
    }
}

#[test]
fn test_duplicate_nonce_rejected() {
    let replay_protector = ReplayProtector::new(10000);
    let nonce_tracker = NonceTracker::new(3600);
    let guard = ReplayGuard::new(replay_protector, nonce_tracker);
    
    let directive1 = create_test_directive();
    let nonce = directive1.nonce.clone();
    
    // First check should pass
    assert!(guard.check_replay(&directive1).is_ok());
    
    // Second check with same nonce should fail
    let directive2 = create_test_directive_with_nonce(nonce);
    let result = guard.check_replay(&directive2);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::NonceReplay(_) => {}
        _ => panic!("Expected NonceReplay error"),
    }
}

fn create_test_directive() -> DirectiveEnvelope {
    use ransomeye_dispatcher::directive_envelope::{TargetScope, AuditReceipt};
    use chrono::Utc;
    
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

fn create_test_directive_with_id(directive_id: String) -> DirectiveEnvelope {
    let mut dir = create_test_directive();
    dir.directive_id = directive_id;
    dir
}

fn create_test_directive_with_nonce(nonce: String) -> DirectiveEnvelope {
    let mut dir = create_test_directive();
    dir.nonce = nonce;
    dir
}
