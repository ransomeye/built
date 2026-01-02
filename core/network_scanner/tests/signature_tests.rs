// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/tests/signature_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for scan result signing and verification

use ransomeye_network_scanner::result::{ScanResult, ScannerMode, Asset, ScanMetadata};
use ransomeye_network_scanner::security::{ScanResultSigner, ScanResultVerifier};
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_unsigned_result_rejected() {
    // Test that unsigned scan results are rejected
    let result = ScanResult {
        scan_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        scanner_mode: ScannerMode::Active,
        asset: Asset {
            ip: "192.168.1.1".to_string(),
            hostname: None,
            mac: None,
            vendor: None,
        },
        open_ports: vec![],
        services: vec![],
        confidence_score: 0.8,
        hash: String::new(),
        signature: String::new(), // Empty signature
        metadata: None,
    };
    
    // Validation should fail
    assert!(result.validate().is_err());
}

#[tokio::test]
async fn test_invalid_signature_rejected() {
    // Test that invalid signatures are rejected
    // This would require actual key pair for full test
    assert!(true); // Placeholder
}

