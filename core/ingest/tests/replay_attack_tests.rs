// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/tests/replay_attack_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for replay attacks - verifies replay attacks are detected and prevented

/*
 * Replay Attack Tests
 * 
 * Tests that verify replay attacks are detected and prevented.
 * All replay attempts must result in event rejection.
 */

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};
    use ransomeye_ingestion::{
        config::Config,
        ordering::OrderingManager,
        security::replay_protection::ReplayProtector,
        protocol::EventEnvelope,
    };
    use chrono::{Utc, Duration as ChronoDuration};

    async fn create_test_config() -> Config {
        Config::load().unwrap()
    }

    async fn create_test_envelope(producer_id: &str, sequence: u64, nonce: &str) -> EventEnvelope {
        EventEnvelope {
            producer_id: producer_id.to_string(),
            component_type: "dpi_probe".to_string(),
            schema_version: 1,
            timestamp: Utc::now(),
            sequence_number: sequence,
            signature: "test_signature".to_string(),
            integrity_hash: "test_hash".to_string(),
            nonce: nonce.to_string(),
            event_data: r#"{"test": "data"}"#.to_string(),
        }
    }

    #[tokio::test]
    async fn test_duplicate_nonce_rejected() {
        // Test that events with duplicate nonces are rejected
        let replay_protector = Arc::new(ReplayProtector::new().unwrap());
        let producer_id = "test_producer_nonce";
        let nonce = "unique_nonce_123";
        let timestamp = Utc::now();
        
        // First use of nonce should pass
        let result = replay_protector.check_replay(producer_id, nonce, &timestamp, 1).await;
        assert!(result.is_ok(), "First use of nonce should be accepted");
        
        // Duplicate nonce should be rejected
        let result = replay_protector.check_replay(producer_id, nonce, &timestamp, 2).await;
        assert!(result.is_err(), "Duplicate nonce should be rejected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("nonce") || error_msg.contains("Replay") || 
                error_msg.contains("Duplicate"),
                "Error should indicate nonce replay: {}", error_msg);
    }

    #[tokio::test]
    async fn test_out_of_order_sequence_rejected() {
        // Test that out-of-order sequences are rejected
        let config = create_test_config().await;
        let ordering_manager = Arc::new(OrderingManager::new(&config).unwrap());
        let producer_id = "test_producer_sequence";
        
        // Send events in order
        let envelope1 = create_test_envelope(producer_id, 1, "nonce1").await;
        let result1 = ordering_manager.check_ordering(producer_id, &envelope1).await;
        assert!(result1.is_ok(), "First event should be accepted");
        let accepted1 = result1.unwrap();
        assert!(accepted1, "First event should pass ordering check");
        
        let envelope2 = create_test_envelope(producer_id, 2, "nonce2").await;
        let result2 = ordering_manager.check_ordering(producer_id, &envelope2).await;
        assert!(result2.is_ok(), "Second event should be accepted");
        let accepted2 = result2.unwrap();
        assert!(accepted2, "Second event should pass ordering check");
        
        // Out-of-order sequence should be rejected
        let envelope0 = create_test_envelope(producer_id, 0, "nonce0").await;
        let result0 = ordering_manager.check_ordering(producer_id, &envelope0).await;
        assert!(result0.is_ok(), "Ordering manager should return result");
        let accepted0 = result0.unwrap();
        assert!(!accepted0, "Out-of-order sequence should be rejected");
    }

    #[tokio::test]
    async fn test_timestamp_out_of_tolerance_rejected() {
        // Test that events with timestamps out of tolerance are rejected
        let replay_protector = Arc::new(ReplayProtector::new().unwrap());
        let producer_id = "test_producer_timestamp";
        
        // Valid timestamp (now)
        let valid_timestamp = Utc::now();
        let result = replay_protector.check_replay(producer_id, "nonce_valid", &valid_timestamp, 1).await;
        assert!(result.is_ok(), "Valid timestamp should be accepted");
        
        // Timestamp too far in the future (outside tolerance)
        let future_timestamp = Utc::now() + ChronoDuration::minutes(10);
        let result = replay_protector.check_replay(producer_id, "nonce_future", &future_timestamp, 2).await;
        assert!(result.is_err(), "Future timestamp out of tolerance should be rejected");
        
        // Timestamp too far in the past (outside tolerance)
        let past_timestamp = Utc::now() - ChronoDuration::minutes(10);
        let result = replay_protector.check_replay(producer_id, "nonce_past", &past_timestamp, 3).await;
        assert!(result.is_err(), "Past timestamp out of tolerance should be rejected");
    }

    #[tokio::test]
    async fn test_replay_detection() {
        // Test that replay attacks are detected
        let config = create_test_config().await;
        let ordering_manager = Arc::new(OrderingManager::new(&config).unwrap());
        let producer_id = "test_producer_replay";
        
        // Send first event
        let envelope1 = create_test_envelope(producer_id, 1, "nonce_replay_1").await;
        let result1 = ordering_manager.check_ordering(producer_id, &envelope1).await;
        assert!(result1.is_ok(), "First event should be processed");
        let accepted1 = result1.unwrap();
        assert!(accepted1, "First event should be accepted");
        
        // Try to replay the same event (same sequence, same nonce)
        let envelope1_replay = create_test_envelope(producer_id, 1, "nonce_replay_1").await;
        let result_replay = ordering_manager.check_ordering(producer_id, &envelope1_replay).await;
        assert!(result_replay.is_ok(), "Ordering manager should return result");
        let accepted_replay = result_replay.unwrap();
        assert!(!accepted_replay, "Replay attempt should be rejected (sequence already seen)");
        
        // Try with different nonce but same sequence (should also be rejected)
        let envelope1_replay2 = create_test_envelope(producer_id, 1, "nonce_replay_2").await;
        let result_replay2 = ordering_manager.check_ordering(producer_id, &envelope1_replay2).await;
        assert!(result_replay2.is_ok(), "Ordering manager should return result");
        let accepted_replay2 = result_replay2.unwrap();
        assert!(!accepted_replay2, "Replay attempt with different nonce should also be rejected");
    }

    #[tokio::test]
    async fn test_sequence_regression_rejected() {
        // Test that sequence number regression is rejected
        let replay_protector = Arc::new(ReplayProtector::new().unwrap());
        let producer_id = "test_producer_regression";
        let timestamp = Utc::now();
        
        // Send sequence 10
        let result1 = replay_protector.check_replay(producer_id, "nonce_10", &timestamp, 10).await;
        assert!(result1.is_ok(), "Sequence 10 should be accepted");
        
        // Try sequence 5 (regression)
        let result2 = replay_protector.check_replay(producer_id, "nonce_5", &timestamp, 5).await;
        assert!(result2.is_err(), "Sequence regression should be rejected");
        
        let error_msg = result2.unwrap_err().to_string();
        assert!(error_msg.contains("sequence") || error_msg.contains("regression") ||
                error_msg.contains("Sequence"),
                "Error should indicate sequence regression: {}", error_msg);
    }
}
