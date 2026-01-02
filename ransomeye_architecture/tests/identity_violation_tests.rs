// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/tests/identity_violation_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Functional tests for identity violation detection - verifies identity misuse is detected and blocked

use ransomeye_architecture_enforcement::{IdentityEnforcer, FailClosedGuard, AuditLogger};
use tempfile::TempDir;

#[test]
#[should_panic(expected = "abort")]
fn test_invalid_identity_format() {
    // Test that invalid identity format results in abort
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let logger = AuditLogger::new(log_path).unwrap();
    let guard = FailClosedGuard::new(logger);
    let enforcer = IdentityEnforcer::new(guard);
    
    // Attempt with invalid identity (too short)
    let _ = enforcer.verify_identity(
        "test_component",
        "short",
        None,
    );
}

#[test]
fn test_valid_identity_passes() {
    // Test that valid identity passes verification
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let logger = AuditLogger::new(log_path).unwrap();
    let guard = FailClosedGuard::new(logger);
    let enforcer = IdentityEnforcer::new(guard);
    
    // Valid identity should pass
    let result = enforcer.verify_identity(
        "test_component",
        "valid_identity_hash_12345678901234567890",
        None,
    );
    
    assert!(result.is_ok(), "Valid identity should pass verification");
}

#[test]
fn test_revoked_identity_detection() {
    // Test that revoked identities are detected
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let logger = AuditLogger::new(log_path).unwrap();
    let guard = FailClosedGuard::new(logger);
    let enforcer = IdentityEnforcer::new(guard);
    
    let identity = "revoked_identity_hash_12345678901234567890";
    
    // Revoke identity
    enforcer.revoke_identity(identity);
    
    // Verify it's revoked
    assert!(enforcer.is_revoked(identity), "Revoked identity should be detected");
}

#[test]
#[should_panic(expected = "abort")]
fn test_revoked_identity_blocks_access() {
    // Test that revoked identity results in abort
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let logger = AuditLogger::new(log_path).unwrap();
    let guard = FailClosedGuard::new(logger);
    let enforcer = IdentityEnforcer::new(guard);
    
    let identity = "revoked_identity_hash_12345678901234567890";
    
    // Revoke identity
    enforcer.revoke_identity(identity);
    
    // Attempt to use revoked identity - should abort
    let _ = enforcer.verify_identity(
        "test_component",
        identity,
        None,
    );
}

#[test]
#[should_panic(expected = "abort")]
fn test_empty_signature_rejected() {
    // Test that empty signature results in abort
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let logger = AuditLogger::new(log_path).unwrap();
    let guard = FailClosedGuard::new(logger);
    let enforcer = IdentityEnforcer::new(guard);
    
    // Attempt with empty signature
    let _ = enforcer.verify_signature(
        "test_component",
        b"test_data",
        "",
        "valid_identity_hash_12345678901234567890",
    );
}
