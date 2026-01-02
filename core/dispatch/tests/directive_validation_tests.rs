// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/directive_validation_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for directive validation - TTL, schema, signature

use ransomeye_dispatcher::directive_envelope::{DirectiveEnvelope, TargetScope, AuditReceipt};
use ransomeye_dispatcher::DirectiveVerifier;
use ransomeye_dispatcher::{TrustChain, NonceTracker, ReplayProtector};
use ransomeye_dispatcher::DispatcherError;
use chrono::Utc;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_expired_directive_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test_key.der");
    
    // Create a dummy key file
    fs::write(&key_path, vec![0u8; 512]).unwrap();
    
    let mut trust_chain = TrustChain::new();
    trust_chain.load_policy_key(key_path.to_str().unwrap()).unwrap();
    
    let nonce_tracker = NonceTracker::new(3600);
    let replay_protector = ReplayProtector::new(10000);
    let verifier = DirectiveVerifier::new(trust_chain, nonce_tracker, replay_protector);
    
    // Create expired directive
    let mut directive = create_test_directive();
    directive.issued_at = Utc::now() - chrono::Duration::seconds(100);
    directive.ttl_seconds = 60; // Expired 40 seconds ago
    
    let result = verifier.verify(&directive);
    assert!(result.is_err());
    match result.unwrap_err() {
        DispatcherError::DirectiveExpired => {}
        _ => panic!("Expected DirectiveExpired error"),
    }
}

#[test]
fn test_missing_signature_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test_key.der");
    fs::write(&key_path, vec![0u8; 512]).unwrap();
    
    let mut trust_chain = TrustChain::new();
    trust_chain.load_policy_key(key_path.to_str().unwrap()).unwrap();
    
    let nonce_tracker = NonceTracker::new(3600);
    let replay_protector = ReplayProtector::new(10000);
    let verifier = DirectiveVerifier::new(trust_chain, nonce_tracker, replay_protector);
    
    let mut directive = create_test_directive();
    directive.signature = String::new(); // Empty signature
    
    let result = verifier.verify(&directive);
    assert!(result.is_err());
}

#[test]
fn test_invalid_structure_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test_key.der");
    fs::write(&key_path, vec![0u8; 512]).unwrap();
    
    let mut trust_chain = TrustChain::new();
    trust_chain.load_policy_key(key_path.to_str().unwrap()).unwrap();
    
    let nonce_tracker = NonceTracker::new(3600);
    let replay_protector = ReplayProtector::new(10000);
    let verifier = DirectiveVerifier::new(trust_chain, nonce_tracker, replay_protector);
    
    let mut directive = create_test_directive();
    directive.directive_id = String::new(); // Empty ID
    
    let result = verifier.verify(&directive);
    assert!(result.is_err());
}

fn create_test_directive() -> DirectiveEnvelope {
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
            platform: Some("linux".to_string()),
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
