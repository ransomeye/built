// Path and File Name : /home/ransomeye/rebuild/core/dispatch/dispatcher/tests/governor_check_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for explicit governor role check

use dispatch::dispatcher::directive_envelope::DirectiveEnvelope;
use dispatch::dispatcher::verifier::DirectiveVerifier;
use dispatch::dispatcher::trust_chain::TrustChain;
use dispatch::dispatcher::nonce::NonceTracker;
use dispatch::dispatcher::replay_protection::ReplayProtector;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_non_governor_attempts_dispatch_rejected() {
    // Create directive with non-governor role
    let directive = DirectiveEnvelope {
        directive_id: Uuid::new_v4().to_string(),
        policy_id: "policy1".to_string(),
        policy_version: "1.0.0".to_string(),
        signature: "test_sig".to_string(),
        signature_hash: "test_hash".to_string(),
        issued_at: Utc::now(),
        ttl_seconds: 3600,
        nonce: Uuid::new_v4().to_string(),
        target_scope: dispatch::dispatcher::directive_envelope::TargetScope {
            agent_ids: None,
            host_addresses: None,
            platform: None,
            asset_class: None,
            environment: None,
        },
        action: "block".to_string(),
        preconditions_hash: "precond_hash".to_string(),
        audit_receipt: dispatch::dispatcher::directive_envelope::AuditReceipt {
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
        reasoning: "Test".to_string(),
        issuer_role: "POLICY_ENGINE".to_string(), // Non-governor role
    };
    
    // Create verifier (will fail on signature, but should also check role)
    let trust_chain = TrustChain::new();
    let nonce_tracker = NonceTracker::new(3600);
    let replay_protector = ReplayProtector::new(10000);
    let verifier = DirectiveVerifier::new(trust_chain, nonce_tracker, replay_protector);
    
    // Verification should fail due to non-governor role
    let result = verifier.verify(&directive);
    assert!(result.is_err());
    
    // Error should indicate role violation
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("GOVERNOR") || format!("{:?}", e).contains("role"));
    }
}

