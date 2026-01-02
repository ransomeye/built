// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/tests/evidence_immutability_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence immutability tests - proves evidence cannot be modified after sealing

use ransomeye_reporting::*;
use tempfile::TempDir;
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_bundle_immutability_after_sealing() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    
    // Create bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    
    // Add evidence
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    
    store.add_evidence(&bundle_id, evidence).unwrap();
    
    // Seal bundle
    store.seal_bundle(&bundle_id).unwrap();
    
    // Attempt to add more evidence (should fail)
    let evidence2 = collector.collect(
        "test_source2",
        "test_type2",
        serde_json::json!({"test": "data2"}),
        None,
        HashMap::new(),
    ).unwrap();
    
    let result = store.add_evidence(&bundle_id, evidence2);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReportingError::BundleSealed(_)));
}

#[test]
fn test_sealed_bundle_cannot_be_modified() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    
    // Create and seal bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    store.seal_bundle(&bundle_id).unwrap();
    
    // Verify bundle is sealed
    let bundle = store.get_bundle(&bundle_id).unwrap();
    assert!(bundle.is_sealed);
    assert!(bundle.sealed_at.is_some());
}

#[test]
fn test_bundle_hash_unchanged_after_sealing() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    
    // Create bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    
    // Add evidence
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    
    store.add_evidence(&bundle_id, evidence).unwrap();
    
    // Seal bundle
    store.seal_bundle(&bundle_id).unwrap();
    
    // Get bundle and record hash
    let bundle = store.get_bundle(&bundle_id).unwrap();
    let original_hash = bundle.bundle_hash.clone();
    
    // Verify hash is unchanged
    let bundle2 = store.get_bundle(&bundle_id).unwrap();
    assert_eq!(bundle2.bundle_hash, original_hash);
}

#[test]
fn test_multiple_seal_attempts_fail() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    
    // Create bundle
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    
    // Seal bundle
    store.seal_bundle(&bundle_id).unwrap();
    
    // Attempt to seal again (should fail)
    let result = store.seal_bundle(&bundle_id);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReportingError::BundleSealed(_)));
}

