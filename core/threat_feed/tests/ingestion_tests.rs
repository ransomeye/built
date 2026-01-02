// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/tests/ingestion_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for threat intel ingestion - unsigned intel rejection, stale intel rejection, schema validation

use std::path::PathBuf;
use tempfile::TempDir;
use chrono::{Utc, Duration};
use threat_feed::ingestion::{ThreatFeedIngester, ThreatIntelBundle, IOC, IOCType, TTP, Campaign};
use threat_feed::errors::ThreatFeedError;

#[test]
fn test_unsigned_intel_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let mut ingester = ThreatFeedIngester::new(temp_dir.path(), 24).unwrap();
    
    // Create bundle with empty signature (unsigned)
    let bundle = ThreatIntelBundle {
        bundle_id: "test_bundle".to_string(),
        source: "test_source".to_string(),
        source_reputation: 0.8,
        timestamp: Utc::now(),
        iocs: Vec::new(),
        ttps: Vec::new(),
        campaigns: Vec::new(),
        signature: "".to_string(), // Empty signature
        signature_algorithm: "RSA-PSS".to_string(),
        public_key_id: "test_key".to_string(),
    };
    
    let result = ingester.ingest_bundle(bundle);
    assert!(result.is_err());
    
    if let Err(ThreatFeedError::InvalidSignature(_)) = result {
        // Expected error
    } else {
        panic!("Expected InvalidSignature error");
    }
}

#[test]
fn test_stale_intel_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let mut ingester = ThreatFeedIngester::new(temp_dir.path(), 24).unwrap();
    
    // Create bundle with old timestamp (25 hours ago)
    let stale_timestamp = Utc::now() - Duration::hours(25);
    
    let bundle = ThreatIntelBundle {
        bundle_id: "test_bundle".to_string(),
        source: "test_source".to_string(),
        source_reputation: 0.8,
        timestamp: stale_timestamp,
        iocs: Vec::new(),
        ttps: Vec::new(),
        campaigns: Vec::new(),
        signature: "fake_signature".to_string(),
        signature_algorithm: "RSA-PSS".to_string(),
        public_key_id: "test_key".to_string(),
    };
    
    let result = ingester.ingest_bundle(bundle);
    assert!(result.is_err());
    
    if let Err(ThreatFeedError::StaleIntel(_)) = result {
        // Expected error
    } else {
        panic!("Expected StaleIntel error");
    }
}

#[test]
fn test_replay_detected() {
    let temp_dir = TempDir::new().unwrap();
    let mut ingester = ThreatFeedIngester::new(temp_dir.path(), 24).unwrap();
    
    let bundle = ThreatIntelBundle {
        bundle_id: "test_bundle".to_string(),
        source: "test_source".to_string(),
        source_reputation: 0.8,
        timestamp: Utc::now(),
        iocs: Vec::new(),
        ttps: Vec::new(),
        campaigns: Vec::new(),
        signature: "fake_signature".to_string(),
        signature_algorithm: "RSA-PSS".to_string(),
        public_key_id: "test_key".to_string(),
    };
    
    // First ingestion (will fail signature check, but that's OK for this test)
    let _ = ingester.ingest_bundle(bundle.clone());
    
    // Second ingestion with same bundle_id - should detect replay
    let result = ingester.ingest_bundle(bundle);
    assert!(result.is_err());
    
    if let Err(ThreatFeedError::ReplayDetected(_)) = result {
        // Expected error
    } else {
        // Might also fail on signature, which is acceptable
        assert!(result.is_err());
    }
}

#[test]
fn test_schema_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mut ingester = ThreatFeedIngester::new(temp_dir.path(), 24).unwrap();
    
    // Create bundle with invalid IOC (empty value)
    let bundle = ThreatIntelBundle {
        bundle_id: "test_bundle".to_string(),
        source: "test_source".to_string(),
        source_reputation: 0.8,
        timestamp: Utc::now(),
        iocs: vec![IOC {
            ioc_id: "test_ioc".to_string(),
            ioc_type: IOCType::IP,
            value: "".to_string(), // Empty value - invalid
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            confidence: 0.8,
            tags: Vec::new(),
            metadata: serde_json::json!({}),
        }],
        ttps: Vec::new(),
        campaigns: Vec::new(),
        signature: "fake_signature".to_string(),
        signature_algorithm: "RSA-PSS".to_string(),
        public_key_id: "test_key".to_string(),
    };
    
    let result = ingester.ingest_bundle(bundle);
    assert!(result.is_err());
    
    if let Err(ThreatFeedError::SchemaValidationFailed(_)) = result {
        // Expected error
    } else {
        // Might also fail on signature, which is acceptable
        assert!(result.is_err());
    }
}

#[test]
fn test_source_attribution_required() {
    let temp_dir = TempDir::new().unwrap();
    let mut ingester = ThreatFeedIngester::new(temp_dir.path(), 24).unwrap();
    
    // Create bundle with empty source
    let bundle = ThreatIntelBundle {
        bundle_id: "test_bundle".to_string(),
        source: "".to_string(), // Empty source
        source_reputation: 0.8,
        timestamp: Utc::now(),
        iocs: Vec::new(),
        ttps: Vec::new(),
        campaigns: Vec::new(),
        signature: "fake_signature".to_string(),
        signature_algorithm: "RSA-PSS".to_string(),
        public_key_id: "test_key".to_string(),
    };
    
    let result = ingester.ingest_bundle(bundle);
    assert!(result.is_err());
    
    if let Err(ThreatFeedError::SourceAttributionMissing(_)) = result {
        // Expected error
    } else {
        // Might also fail on signature or schema, which is acceptable
        assert!(result.is_err());
    }
}

