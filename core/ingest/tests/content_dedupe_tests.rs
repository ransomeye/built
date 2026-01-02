// Path and File Name : /home/ransomeye/rebuild/core/ingest/tests/content_dedupe_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for content hash deduplication

use ransomeye_ingestion::dedupe::ContentDeduplicator;
use ransomeye_ingestion::protocol::event_envelope::EventEnvelope;
use ransomeye_ingestion::config::Config;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_same_payload_different_nonce_deduped() {
    let config = Config::load().unwrap();
    let deduplicator = ContentDeduplicator::new(&config).unwrap();
    
    // Create two envelopes with same content but different nonces
    let event_data = r#"{"event": "test", "value": 123}"#;
    
    let envelope1 = EventEnvelope {
        producer_id: "test_producer".to_string(),
        component_type: "dpi_probe".to_string(),
        schema_version: 1,
        timestamp: Utc::now(),
        sequence_number: 1,
        signature: "sig1".to_string(),
        integrity_hash: "hash1".to_string(),
        nonce: Uuid::new_v4().to_string(),
        event_data: event_data.to_string(),
        priority: "INFO".to_string(),
    };
    
    let envelope2 = EventEnvelope {
        producer_id: "test_producer".to_string(),
        component_type: "dpi_probe".to_string(),
        schema_version: 1,
        timestamp: Utc::now(),
        sequence_number: 2,
        signature: "sig2".to_string(),
        integrity_hash: "hash2".to_string(),
        nonce: Uuid::new_v4().to_string(), // Different nonce
        event_data: event_data.to_string(), // Same content
        priority: "INFO".to_string(),
    };
    
    // First should process
    assert!(deduplicator.should_process(&envelope1).await.unwrap());
    
    // Second should be deduplicated (same content, different nonce)
    assert!(!deduplicator.should_process(&envelope2).await.unwrap());
}

