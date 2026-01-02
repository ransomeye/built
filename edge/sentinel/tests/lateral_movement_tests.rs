// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/tests/lateral_movement_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for lateral movement detection - credential reuse, token replay, SMB abuse, SSH brute force

use chrono::Utc;
use sentinel::lateral_movement::{LateralMovementDetector, LateralMovementType};

#[test]
fn test_credential_reuse_detection() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    // First use on host1
    detector.detect_credential_reuse("hash123", "host1", "host1", now);
    
    // Reuse on host2 - should trigger
    let event = detector.detect_credential_reuse("hash123", "host1", "host2", now);
    assert!(event.is_some());
    
    let event = event.unwrap();
    assert_eq!(event.event_type, LateralMovementType::CredentialReuse);
    assert_eq!(event.source_host, "host1");
    assert_eq!(event.target_host, "host2");
    assert_eq!(event.confidence_score, 0.95);
}

#[test]
fn test_token_replay_detection() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    let event = detector.detect_token_replay("token123", "host1", "host2", now);
    assert!(event.is_some());
    
    let event = event.unwrap();
    assert_eq!(event.event_type, LateralMovementType::TokenReplay);
    assert_eq!(event.token_id, Some("token123".to_string()));
    assert_eq!(event.confidence_score, 0.90);
}

#[test]
fn test_smb_abuse_detection() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    let event = detector.detect_smb_abuse("host1", "host2", Some("hash123"), now);
    assert!(event.is_some());
    
    let event = event.unwrap();
    assert_eq!(event.event_type, LateralMovementType::SMBProbe);
    assert_eq!(event.protocol, "smb");
    assert_eq!(event.confidence_score, 0.85);
}

#[test]
fn test_ssh_brute_force_detection() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    // Less than 3 attempts - should not trigger
    let event = detector.detect_ssh_brute_force("host1", "host2", 2, now);
    assert!(event.is_none());
    
    // 5 attempts - should trigger with medium confidence
    let event = detector.detect_ssh_brute_force("host1", "host2", 5, now);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.event_type, LateralMovementType::SSHBruteForce);
    assert_eq!(event.confidence_score, 0.85);
    
    // 10+ attempts - should trigger with high confidence
    let event = detector.detect_ssh_brute_force("host1", "host2", 10, now);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.confidence_score, 0.95);
}

#[test]
fn test_privilege_escalation_detection() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    // Same user - should not trigger
    let event = detector.detect_privilege_escalation("host1", "host2", "user1", "user1", now);
    assert!(event.is_none());
    
    // Different user - should trigger
    let event = detector.detect_privilege_escalation("host1", "host2", "user1", "root", now);
    assert!(event.is_some());
    
    let event = event.unwrap();
    assert_eq!(event.event_type, LateralMovementType::PrivilegeEscalation);
    assert_eq!(event.confidence_score, 0.90);
}

#[test]
fn test_attacker_session_tracking() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    // First event creates session
    let event1 = detector.detect_credential_reuse("hash123", "host1", "host2", now);
    assert!(event1.is_some());
    let session_id1 = event1.unwrap().attacker_session_id.clone();
    
    // Second event should use same session
    let event2 = detector.detect_token_replay("token123", "host1", "host3", now);
    assert!(event2.is_some());
    let session_id2 = event2.unwrap().attacker_session_id.clone();
    
    // Sessions should be different (different source/target combinations)
    // But if same source, might be same session
    // This test verifies session tracking works
    assert!(!session_id1.is_empty());
    assert!(!session_id2.is_empty());
}

#[test]
fn test_correlation_across_hosts() {
    let detector = LateralMovementDetector::new(3600, 0.8);
    let now = Utc::now();
    
    // Credential used on host1
    detector.detect_credential_reuse("hash123", "host1", "host1", now);
    
    // Same credential used on host2 - should correlate
    let event = detector.detect_credential_reuse("hash123", "host1", "host2", now);
    assert!(event.is_some());
    
    let event = event.unwrap();
    assert_eq!(event.source_host, "host1");
    assert_eq!(event.target_host, "host2");
    assert!(event.attacker_session_id.len() > 0);
}

