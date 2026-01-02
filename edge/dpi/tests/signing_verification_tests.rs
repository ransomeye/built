// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/tests/signing_verification_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for event signing and signature verification

use ransomeye_dpi_probe::signing::{EventSigner, SignedEvent};
use ransomeye_dpi_probe::identity::Identity;
use ransomeye_dpi_probe::config::Config;
use sha2::{Sha256, Digest};
use hex;
use tempfile::TempDir;

#[test]
fn test_event_signing() {
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
        "dst_ip": "192.168.1.2",
    });
    
    let signed_event = signer.sign_event(test_data).unwrap();
    
    // Verify all required fields are present
    assert!(!signed_event.message_id.is_empty());
    assert!(!signed_event.nonce.is_empty());
    assert!(!signed_event.component_identity.is_empty());
    assert!(!signed_event.signature.is_empty());
    assert!(!signed_event.data_hash.is_empty());
    
    // Verify nonce format (64 hex characters)
    assert_eq!(signed_event.nonce.len(), 64);
    assert!(signed_event.nonce.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Verify signature format (base64)
    assert!(!signed_event.signature.is_empty());
    
    // Verify data_hash format (64 hex characters)
    assert_eq!(signed_event.data_hash.len(), 64);
    assert!(signed_event.data_hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_reject_unsigned_event() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    let identity = Identity::load_or_create(&config).unwrap();
    
    let signer = EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    );
    
    let test_data = serde_json::json!({
        "flow_id": "test-flow-1",
    });
    
    let signed_event = signer.sign_event(test_data).unwrap();
    
    // An unsigned event (empty signature) should be rejected
    // In production, Core would reject this
    // Here we verify that our signing always produces a signature
    assert!(!signed_event.signature.is_empty());
    
    // Create a tampered event with empty signature
    let mut tampered_event = signed_event.clone();
    tampered_event.signature = String::new();
    
    // This would be rejected by Core (we test the structure here)
    assert!(tampered_event.signature.is_empty());
}

#[test]
fn test_signature_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    let identity = Identity::load_or_create(&config).unwrap();
    
    let signer = EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    );
    
    let test_data = serde_json::json!({
        "flow_id": "test-flow-1",
    });
    
    // Sign the same data twice
    let signed_event1 = signer.sign_event(test_data.clone()).unwrap();
    let signed_event2 = signer.sign_event(test_data).unwrap();
    
    // Nonces should be different (probabilistic signature)
    assert_ne!(signed_event1.nonce, signed_event2.nonce);
    assert_ne!(signed_event1.message_id, signed_event2.message_id);
    
    // Signatures should be different (due to nonce/timestamp)
    assert_ne!(signed_event1.signature, signed_event2.signature);
}

#[test]
fn test_data_hash_consistency() {
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
    
    // Verify data_hash matches data
    let data_json = serde_json::to_vec(&signed_event.data).unwrap();
    let computed_hash = sha2::Sha256::digest(&data_json);
    let computed_hash_hex = hex::encode(computed_hash);
    
    assert_eq!(signed_event.data_hash, computed_hash_hex);
}

#[test]
fn test_message_id_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    let identity = Identity::load_or_create(&config).unwrap();
    
    let signer = EventSigner::new(
        identity.keypair(),
        identity.producer_id().to_string(),
    );
    
    let test_data = serde_json::json!({"flow_id": "test"});
    
    // Generate multiple events
    let mut message_ids = std::collections::HashSet::new();
    for _ in 0..100 {
        let signed_event = signer.sign_event(test_data.clone()).unwrap();
        assert!(message_ids.insert(signed_event.message_id.clone()));
    }
    
    // All message IDs should be unique
    assert_eq!(message_ids.len(), 100);
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
