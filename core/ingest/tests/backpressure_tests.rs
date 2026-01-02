// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/tests/backpressure_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Comprehensive backpressure tests - verifies backpressure activation, signaling, and clearing

/*
 * Backpressure Tests
 * 
 * Tests that verify backpressure behavior:
 * - Buffer full triggers backpressure
 * - Backpressure rejects events
 * - Backpressure clears after timeout
 * - System remains alive during backpressure
 * - No silent drops occur
 */

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};
    use ransomeye_ingestion::{
        config::Config,
        backpressure::BackpressureController,
        buffer::EventBuffer,
        protocol::EventEnvelope,
    };
    use chrono::Utc;

    async fn create_test_config() -> Config {
        Config::load().unwrap()
    }

    async fn create_test_envelope(producer_id: &str, sequence: u64) -> EventEnvelope {
        EventEnvelope {
            producer_id: producer_id.to_string(),
            component_type: "dpi_probe".to_string(),
            schema_version: 1,
            timestamp: Utc::now(),
            sequence_number: sequence,
            signature: "test_signature".to_string(),
            integrity_hash: "test_hash".to_string(),
            nonce: format!("nonce_{}", sequence),
            event_data: r#"{"test": "data"}"#.to_string(),
        }
    }

    #[tokio::test]
    async fn test_backpressure_on_buffer_full() {
        // Create config with small buffer for testing
        let mut config = create_test_config().await;
        config.buffer_capacity = 5; // Small buffer for testing
        
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        
        let producer_id = "test_producer_001";
        
        // Fill buffer to capacity
        for i in 0..config.buffer_capacity {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            let result = buffer.add(&envelope).await;
            assert!(result.is_ok(), "Event {} should be accepted", i);
        }
        
        // Next event should trigger backpressure
        let envelope = create_test_envelope(producer_id, config.buffer_capacity as u64).await;
        let result = buffer.add(&envelope).await;
        assert!(result.is_err(), "Event should be rejected when buffer is full");
        
        // Verify backpressure is signaled
        backpressure.signal_backpressure(producer_id).await;
        assert!(!backpressure.can_accept(producer_id).await, 
                "Backpressure should be active");
    }

    #[tokio::test]
    async fn test_backpressure_rejects_events() {
        let config = create_test_config().await;
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        
        let producer_id = "test_producer_002";
        
        // Activate backpressure
        backpressure.signal_backpressure(producer_id).await;
        
        // Verify events are rejected
        assert!(!backpressure.can_accept(producer_id).await,
                "Events should be rejected when backpressure is active");
        
        // Clear backpressure
        backpressure.clear_backpressure(producer_id).await;
        
        // Verify events are accepted again
        assert!(backpressure.can_accept(producer_id).await,
                "Events should be accepted after backpressure clears");
    }

    #[tokio::test]
    async fn test_backpressure_auto_clears_after_timeout() {
        let mut config = create_test_config().await;
        config.backpressure_clear_seconds = 1; // Short timeout for testing
        
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        let producer_id = "test_producer_003";
        
        // Activate backpressure
        backpressure.signal_backpressure(producer_id).await;
        assert!(!backpressure.can_accept(producer_id).await,
                "Backpressure should be active initially");
        
        // Wait for timeout
        sleep(Duration::from_secs(config.backpressure_clear_seconds + 1)).await;
        
        // Verify backpressure auto-clears
        assert!(backpressure.can_accept(producer_id).await,
                "Backpressure should auto-clear after timeout");
    }

    #[tokio::test]
    async fn test_global_backpressure() {
        let config = create_test_config().await;
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        
        // Activate global backpressure
        backpressure.set_global_backpressure(true).await;
        
        // Verify all producers are affected
        assert!(!backpressure.can_accept("producer_001").await,
                "Producer 1 should be affected by global backpressure");
        assert!(!backpressure.can_accept("producer_002").await,
                "Producer 2 should be affected by global backpressure");
        
        // Clear global backpressure
        backpressure.set_global_backpressure(false).await;
        
        // Verify events are accepted again
        assert!(backpressure.can_accept("producer_001").await,
                "Events should be accepted after global backpressure clears");
    }

    #[tokio::test]
    async fn test_backpressure_no_silent_drops() {
        let mut config = create_test_config().await;
        config.buffer_capacity = 3;
        
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        let producer_id = "test_producer_004";
        
        // Fill buffer
        for i in 0..config.buffer_capacity {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            let result = buffer.add(&envelope).await;
            assert!(result.is_ok(), "Event should be accepted");
        }
        
        // Attempt to add beyond capacity - should fail explicitly
        let envelope = create_test_envelope(producer_id, config.buffer_capacity as u64).await;
        let result = buffer.add(&envelope).await;
        assert!(result.is_err(), "Event should be explicitly rejected, not silently dropped");
        
        // Verify error message indicates rejection
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("full") || error_msg.contains("Buffer"),
                "Error should indicate buffer full");
    }

    #[tokio::test]
    async fn test_backpressure_per_producer_isolation() {
        let config = create_test_config().await;
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        
        let producer_1 = "producer_001";
        let producer_2 = "producer_002";
        
        // Activate backpressure for producer 1 only
        backpressure.signal_backpressure(producer_1).await;
        
        // Verify producer 1 is affected
        assert!(!backpressure.can_accept(producer_1).await,
                "Producer 1 should have backpressure active");
        
        // Verify producer 2 is NOT affected
        assert!(backpressure.can_accept(producer_2).await,
                "Producer 2 should NOT have backpressure active");
        
        // Clear producer 1 backpressure
        backpressure.clear_backpressure(producer_1).await;
        assert!(backpressure.can_accept(producer_1).await,
                "Producer 1 should accept events after clearing");
    }

    #[tokio::test]
    async fn test_backpressure_system_remains_alive() {
        let mut config = create_test_config().await;
        config.buffer_capacity = 2;
        config.backpressure_clear_seconds = 1;
        
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        let producer_id = "test_producer_005";
        
        // Fill buffer and trigger backpressure
        for i in 0..config.buffer_capacity {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            buffer.add(&envelope).await.unwrap();
        }
        
        // Attempt to add beyond capacity - should reject but not crash
        let envelope = create_test_envelope(producer_id, config.buffer_capacity as u64).await;
        let result = buffer.add(&envelope).await;
        assert!(result.is_err(), "Should reject but not crash");
        
        // Verify system is still operational
        backpressure.signal_backpressure(producer_id).await;
        assert!(!backpressure.can_accept(producer_id).await,
                "System should still be operational");
        
        // Wait and verify backpressure clears
        sleep(Duration::from_secs(config.backpressure_clear_seconds + 1)).await;
        assert!(backpressure.can_accept(producer_id).await,
                "System should recover after backpressure clears");
    }

    #[tokio::test]
    async fn test_backpressure_bounded_buffer_exhaustion() {
        let mut config = create_test_config().await;
        config.buffer_capacity = 10;
        
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        let producer_id = "test_producer_006";
        
        // Flood buffer with events
        let mut accepted = 0;
        let mut rejected = 0;
        
        for i in 0..config.buffer_capacity * 2 {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            match buffer.add(&envelope).await {
                Ok(_) => accepted += 1,
                Err(_) => rejected += 1,
            }
        }
        
        // Verify bounded behavior
        assert!(accepted <= config.buffer_capacity,
                "Should not accept more than buffer capacity");
        assert!(rejected > 0, "Should reject events when buffer is full");
        assert!(accepted + rejected == config.buffer_capacity * 2,
                "All events should be accounted for (accepted or rejected)");
    }
}

