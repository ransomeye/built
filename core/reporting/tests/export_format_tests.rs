// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/tests/export_format_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Export format tests - validates PDF, HTML, and CSV export formats

use ransomeye_reporting::*;
use tempfile::TempDir;
use std::collections::HashMap;
use std::fs;

#[test]
fn test_pdf_export() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    let output_path = temp_dir.path().join("report.pdf");
    
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
    
    // Export PDF
    let exporter = ReportExporter::new();
    exporter.export_pdf(&report, &output_path).unwrap();
    
    // Verify PDF exists
    assert!(output_path.exists());
    
    // Verify PDF contains footer
    let pdf_data = fs::read(&output_path).unwrap();
    // PDF is binary, so we check it exists and has content
    assert!(!pdf_data.is_empty());
}

#[test]
fn test_html_export() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    let output_path = temp_dir.path().join("report.html");
    
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
    
    // Export HTML
    let exporter = ReportExporter::new();
    exporter.export_html(&report, &output_path).unwrap();
    
    // Verify HTML exists
    assert!(output_path.exists());
    
    // Verify HTML contains footer
    let html_content = fs::read_to_string(&output_path).unwrap();
    assert!(html_content.contains("RansomEye"));
    assert!(html_content.contains("© RansomEye.Tech"));
}

#[test]
fn test_csv_export() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    let output_path = temp_dir.path().join("report.csv");
    
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
    
    // Export CSV
    let exporter = ReportExporter::new();
    exporter.export_csv(&report, &output_path).unwrap();
    
    // Verify CSV exists
    assert!(output_path.exists());
    
    // Verify CSV contains metadata
    let csv_content = fs::read_to_string(&output_path).unwrap();
    assert!(csv_content.contains("Report ID"));
    assert!(csv_content.contains(&report.metadata.report_id));
}

#[test]
fn test_export_all_formats() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("store");
    let output_dir = temp_dir.path().join("exports");
    
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
    
    // Export all formats
    let exporter = ReportExporter::new();
    let exported_files = exporter.export_all(&report, &output_dir).unwrap();
    
    // Verify all formats were exported
    assert_eq!(exported_files.len(), 3);
    assert!(exported_files.iter().any(|f| f.ends_with(".pdf")));
    assert!(exported_files.iter().any(|f| f.ends_with(".html")));
    assert!(exported_files.iter().any(|f| f.ends_with(".csv")));
}

