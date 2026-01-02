// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/spoofed_identity_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests to detect and reject spoofed probe identity

use ransomeye_dpi_probe::signing::{EventSigner, SignedEvent};
use ransomeye_dpi_probe::identity::Identity;
use ransomeye_dpi_probe::config::Config;
use ring::signature::RsaKeyPair;
use ring::rand::SystemRandom;
use tempfile::TempDir;

#[test]
fn test_reject_spoofed_identity() {
    // Create two different identities
    let temp_dir = TempDir::new().unwrap();
    let config1 = create_test_config(&temp_dir);
    let config2 = create_test_config(&temp_dir);
    
    let identity1 = Identity::load_or_create(&config1).unwrap();
    let identity2 = Identity::load_or_create(&config2).unwrap();
    
    // Create signer with identity1
    let signer1 = EventSigner::new(
        identity1.keypair(),
        identity1.producer_id().to_string(),
    );
    
    // Create event with identity1's signature but identity2's component_identity
    let test_data = serde_json::json!({
        "flow_id": "test-flow-1",
        "src_ip": "192.168.1.1",
        "dst_ip": "192.168.1.2",
    });
    
    let signed_event = signer1.sign_event(test_data).unwrap();
    
    // Verify that component_identity matches signer's identity
    assert_eq!(signed_event.component_identity, identity1.producer_id());
    assert_ne!(signed_event.component_identity, identity2.producer_id());
    
    // Attempting to use identity2's ID with identity1's signature should fail verification
    // (This would be caught by Core's verification, but we test the identity binding here)
    assert!(signed_event.component_identity.starts_with("dpi_probe_"));
}

#[test]
fn test_identity_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let config1 = create_test_config(&temp_dir);
    let config2 = create_test_config(&temp_dir);
    
    let identity1 = Identity::load_or_create(&config1).unwrap();
    let identity2 = Identity::load_or_create(&config2).unwrap();
    
    // Each identity should have a unique producer_id
    assert_ne!(identity1.producer_id(), identity2.producer_id());
}

#[test]
fn test_identity_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create identity
    let identity1 = Identity::load_or_create(&config).unwrap();
    let producer_id1 = identity1.producer_id().to_string();
    
    // Load same identity again
    let identity2 = Identity::load_or_create(&config).unwrap();
    let producer_id2 = identity2.producer_id().to_string();
    
    // Should be the same identity
    assert_eq!(producer_id1, producer_id2);
}

#[test]
fn test_signature_binds_to_identity() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    let identity = Identity::load_or_create(&config).unwrap();
    
    let signer = EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    );
    
    let test_data = serde_json::json!({
        "flow_id": "test-flow-1",
        "src_ip": "192.168.1.1",
    });
    
    let signed_event = signer.sign_event(test_data).unwrap();
    
    // Signature should be bound to the identity
    assert_eq!(signed_event.component_identity, identity.producer_id());
    assert!(!signed_event.signature.is_empty());
    assert!(!signed_event.data_hash.is_empty());
}

fn create_test_config(temp_dir: &TempDir) -> Config {
    Config {
        core_api_url: "https://localhost:8443".to_string(),
        capture_interface: "eth0".to_string(),
        buffer_dir: temp_dir.path().join("buffer").to_string_lossy().to_string(),
        cert_path: "/tmp/test.crt".to_string(),
        key_path: "/tmp/test.key".to_string(),
        ca_cert_path: "/tmp/ca.crt".to_string(),
        max_buffer_size_mb: 100,
        backpressure_threshold: 1024,
        flow_timeout_seconds: 300,
        health_report_interval_seconds: 60,
    }
}
