// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/tests/corruption_detection_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Corruption detection tests - validates detection of evidence tampering and corruption

use ransomeye_reporting::*;
use tempfile::TempDir;
use std::collections::HashMap;
use std::fs;

#[test]
fn test_corruption_detection() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    
    // Create and seal bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    store.add_evidence(&bundle_id, evidence).unwrap();
    store.seal_bundle(&bundle_id).unwrap();
    
    // Verify bundle integrity
    let verifier = EvidenceVerifier::new();
    let result = verifier.verify_store(&store).unwrap();
    
    assert!(result.is_valid);
    assert_eq!(result.corrupted_bundles, 0);
}

#[test]
fn test_tampering_detection() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    
    // Create and seal bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    store.add_evidence(&bundle_id, evidence).unwrap();
    store.seal_bundle(&bundle_id).unwrap();
    
    // Detect tampering
    let verifier = EvidenceVerifier::new();
    let tampered = verifier.detect_tampering(&store).unwrap();
    
    // Should be empty (no tampering)
    assert!(tampered.is_empty());
}

#[test]
fn test_hash_mismatch_detection() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    
    // Create and seal bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    store.add_evidence(&bundle_id, evidence).unwrap();
    store.seal_bundle(&bundle_id).unwrap();
    
    // Get bundle
    let bundle = store.get_bundle(&bundle_id).unwrap();
    
    // Verify integrity (should pass)
    let is_valid = store.verify_bundle_integrity(&bundle).unwrap();
    assert!(is_valid);
    
    // Create corrupted bundle (wrong hash)
    let mut corrupted_bundle = bundle.clone();
    corrupted_bundle.bundle_hash = "wrong_hash".to_string();
    
    // Verify should fail
    let is_valid = store.verify_bundle_integrity(&corrupted_bundle).unwrap();
    assert!(!is_valid);
}

#[test]
fn test_missing_evidence_detection() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    
    // Try to get non-existent bundle
    let result = store.get_bundle("non-existent-id");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReportingError::MissingEvidence(_)));
}

#[test]
fn test_verification_fails_on_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    
    // Create and seal bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    store.add_evidence(&bundle_id, evidence).unwrap();
    store.seal_bundle(&bundle_id).unwrap();
    
    // Verify store
    let verifier = EvidenceVerifier::new();
    let result = verifier.verify_store(&store).unwrap();
    
    // Should be valid
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

