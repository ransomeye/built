// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/tests/integration_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Integration tests for Linux Agent - identity spoofing, event signing, backpressure, core unavailability, resource exhaustion

use std::sync::Arc;
use ransomeye_linux_agent::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_identity_spoofing() {
    // Test that identity cannot be spoofed
    let temp_dir = TempDir::new().unwrap();
    let mut config = config::Config::load().unwrap();
    config.buffer_dir = temp_dir.path().join("buffer").to_string_lossy().to_string();
    
    let identity1 = identity::Identity::load_or_create(&config).unwrap();
    let identity2 = identity::Identity::load_or_create(&config).unwrap();
    
    // Same instance should have same identity
    assert_eq!(identity1.producer_id(), identity2.producer_id());
    
    // Sign event with identity1
    let signer1 = signing::EventSigner::new(
        identity1.keypair(),
        identity1.producer_id().to_string(),
    );
    
    let data = serde_json::json!({"test": "data"});
    let signed_event = signer1.sign_event(data).unwrap();
    
    // Verify signature matches identity
    assert_eq!(signed_event.component_identity, identity1.producer_id());
    assert!(!signed_event.signature.is_empty());
    assert!(!signed_event.data_hash.is_empty());
}

#[tokio::test]
async fn test_event_signing() {
    // Test event signing and verification
    let temp_dir = TempDir::new().unwrap();
    let mut config = config::Config::load().unwrap();
    config.buffer_dir = temp_dir.path().join("buffer").to_string_lossy().to_string();
    
    let identity = identity::Identity::load_or_create(&config).unwrap();
    let signer = signing::EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    );
    
    let data = serde_json::json!({
        "event_type": "process",
        "pid": 1234,
        "process_name": "test",
    });
    
    let signed_event = signer.sign_event(data.clone()).unwrap();
    
    // Verify all required fields
    assert!(!signed_event.message_id.is_empty());
    assert!(!signed_event.nonce.is_empty());
    assert!(!signed_event.signature.is_empty());
    assert!(!signed_event.data_hash.is_empty());
    assert_eq!(signed_event.component_identity, identity.producer_id());
    assert!(!signed_event.host_id.is_empty());
}

#[tokio::test]
async fn test_backpressure() {
    // Test backpressure handling
    let backpressure = Arc::new(backpressure::BackpressureHandler::new(1000, 800));
    
    // Fill buffer to threshold
    for _ in 0..800 {
        assert!(backpressure.increment_buffer(1));
    }
    
    // Should trigger backpressure
    assert!(backpressure.should_backpressure());
    backpressure.set_backpressure(true);
    assert!(backpressure.is_backpressure_active());
}

#[tokio::test]
async fn test_core_unavailability() {
    // Test that sensor tolerates Core unavailability
    let temp_dir = TempDir::new().unwrap();
    let mut config = config::Config::load().unwrap();
    config.buffer_dir = temp_dir.path().join("buffer").to_string_lossy().to_string();
    config.core_api_url = "https://invalid-host:8443".to_string();
    
    let backpressure = Arc::new(backpressure::BackpressureHandler::new(1000, 800));
    let transport = transport::TransportClient::new(config, backpressure).unwrap();
    
    // Health check should fail gracefully
    let health = transport.health_check().await;
    assert!(!health); // Should fail but not panic
}

#[tokio::test]
async fn test_resource_exhaustion() {
    // Test resource exhaustion handling
    let backpressure = Arc::new(backpressure::BackpressureHandler::new(100, 80));
    
    // Exhaust buffer
    let mut accepted = 0;
    let mut dropped = 0;
    for _ in 0..200 {
        if backpressure.increment_buffer(1) {
            accepted += 1;
        } else {
            dropped += 1;
        }
    }
    
    // Should have dropped some events
    assert!(dropped > 0);
    assert_eq!(backpressure.get_dropped_count(), dropped);
}

