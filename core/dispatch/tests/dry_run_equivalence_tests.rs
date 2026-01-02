// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/tests/dry_run_equivalence_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for dry-run audit output equivalence

use ransomeye_dispatcher::directive_envelope::{DirectiveEnvelope, TargetScope, AuditReceipt};
use ransomeye_dispatcher::{AuditLogger, AuditEventType};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_dry_run_audit_output() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    
    let logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();
    
    // Log dry-run event
    let details = serde_json::json!({
        "directive_id": "test-directive-1",
        "action": "block",
        "dry_run": true,
    });
    
    let record_id = logger.append(AuditEventType::ExecutionAttempted, details).unwrap();
    assert!(!record_id.is_empty());
    
    // Verify log file exists and contains record
    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("test-directive-1"));
    assert!(content.contains("ExecutionAttempted"));
}

#[test]
fn test_audit_log_hash_chaining() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    
    let logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();
    
    // Append multiple records
    let record1_id = logger.append(
        AuditEventType::DirectiveReceived,
        serde_json::json!({"directive_id": "dir-1"}),
    ).unwrap();
    
    let hash1 = logger.get_last_hash();
    
    let record2_id = logger.append(
        AuditEventType::DirectiveValidated,
        serde_json::json!({"directive_id": "dir-1"}),
    ).unwrap();
    
    let hash2 = logger.get_last_hash();
    
    // Hashes should be different (chained)
    assert_ne!(hash1, hash2);
    assert_ne!(record1_id, record2_id);
}

fn create_test_directive() -> DirectiveEnvelope {
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
