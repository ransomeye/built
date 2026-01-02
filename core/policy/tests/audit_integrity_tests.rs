// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/audit_integrity_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests for audit logging integrity

use std::path::Path;
use tempfile::TempDir;
use ransomeye_policy::audit::{initialize_audit_logger, verify_audit_chain, log_decision};
use ransomeye_policy::{PolicyDecision, AllowedAction};
use chrono::Utc;

#[test]
fn test_audit_logging_creates_hash_chain() {
    let temp_dir = TempDir::new().unwrap();
    let audit_log = temp_dir.path().join("audit.log");

    initialize_audit_logger(audit_log.to_str().unwrap()).unwrap();

    let decision = PolicyDecision::new(
        "alert_1",
        "policy_1",
        "1.0.0",
        AllowedAction::Deny,
        vec![AllowedAction::Deny],
        vec![],
        "evidence_1",
        "actions_on_objectives",
        "critical",
        None,
        "Test reasoning",
        "signature_hash_1",
    );

    let hash1 = log_decision(&decision, "signature_hash_1").unwrap();
    assert!(!hash1.is_empty());

    let decision2 = PolicyDecision::new(
        "alert_2",
        "policy_2",
        "1.0.0",
        AllowedAction::Allow,
        vec![AllowedAction::Allow],
        vec![],
        "evidence_2",
        "delivery",
        "high",
        None,
        "Test reasoning 2",
        "signature_hash_2",
    );

    let hash2 = log_decision(&decision2, "signature_hash_2").unwrap();
    assert!(!hash2.is_empty());
    assert_ne!(hash1, hash2);
}

#[test]
fn test_audit_chain_verification() {
    let temp_dir = TempDir::new().unwrap();
    let audit_log = temp_dir.path().join("audit.log");

    initialize_audit_logger(audit_log.to_str().unwrap()).unwrap();

    let decision = PolicyDecision::new(
        "alert_1",
        "policy_1",
        "1.0.0",
        AllowedAction::Deny,
        vec![AllowedAction::Deny],
        vec![],
        "evidence_1",
        "actions_on_objectives",
        "critical",
        None,
        "Test reasoning",
        "signature_hash_1",
    );

    log_decision(&decision, "signature_hash_1").unwrap();

    let verified = verify_audit_chain().unwrap();
    assert!(verified);
}

