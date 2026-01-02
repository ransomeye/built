// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/tests/report_reproducibility_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Report reproducibility tests - validates that reports can be regenerated identically

use ransomeye_reporting::*;
use tempfile::TempDir;
use std::collections::HashMap;

#[test]
fn test_report_reproducibility() {
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
    
    // Build report
    let builder = ReportBuilder::new("1.0.0", "1.0.0", "build_hash", None);
    let bundles = vec![store.get_bundle(&bundle_id).unwrap()];
    let report1 = builder.build_report(
        "Test Report",
        "Test Description",
        &bundles,
        None,
    ).unwrap();
    
    // Rebuild report from same bundles
    let bundles2 = vec![store.get_bundle(&bundle_id).unwrap()];
    let report2 = builder.build_report(
        "Test Report",
        "Test Description",
        &bundles2,
        None,
    ).unwrap();
    
    // Reports should have same evidence hashes
    assert_eq!(report1.evidence_hashes, report2.evidence_hashes);
    assert_eq!(report1.evidence_bundle_ids, report2.evidence_bundle_ids);
    assert_eq!(report1.summary.total_evidence_items, report2.summary.total_evidence_items);
}

#[test]
fn test_report_reproducibility_verification() {
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
    
    // Build report
    let builder = ReportBuilder::new("1.0.0", "1.0.0", "build_hash", None);
    let bundles = vec![store.get_bundle(&bundle_id).unwrap()];
    let report = builder.build_report(
        "Test Report",
        "Test Description",
        &bundles,
        None,
    ).unwrap();
    
    // Verify reproducibility
    let is_reproducible = builder.verify_reproducibility(&report, &bundles).unwrap();
    assert!(is_reproducible);
}

#[test]
fn test_unsealed_bundle_fails_reproducibility() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    
    let store = EvidenceStore::new(&store_path, None).unwrap();
    let collector = EvidenceCollector::new("1.0.0", "1.0.0");
    
    // Create bundle but don't seal
    let bundle_id = store.create_bundle("1.0.0", "1.0.0").unwrap();
    let evidence = collector.collect(
        "test_source",
        "test_type",
        serde_json::json!({"test": "data"}),
        None,
        HashMap::new(),
    ).unwrap();
    store.add_evidence(&bundle_id, evidence).unwrap();
    // Don't seal
    
    // Build report
    let builder = ReportBuilder::new("1.0.0", "1.0.0", "build_hash", None);
    let bundles = vec![store.get_bundle(&bundle_id).unwrap()];
    let report = builder.build_report(
        "Test Report",
        "Test Description",
        &bundles,
        None,
    ).unwrap();
    
    // Verify reproducibility should fail (bundle not sealed)
    let is_reproducible = builder.verify_reproducibility(&report, &bundles).unwrap();
    assert!(!is_reproducible);
}

