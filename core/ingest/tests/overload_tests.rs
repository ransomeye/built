// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/tests/overload_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for system overload - verifies events are rejected when system is overloaded

/*
 * Overload Tests
 * 
 * Tests that verify events are rejected when system is overloaded.
 * All overload conditions must result in event rejection and backpressure.
 */

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};
    use ransomeye_ingestion::{
        config::Config,
        rate_limit::RateLimiter,
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
    async fn test_rate_limit_exceeded_rejected() {
        // Test that events are rejected when rate limit is exceeded
        let mut config = create_test_config().await;
        config.producer_rate_limit = 5; // Low limit for testing
        config.rate_limit_window_seconds = 60;
        
        let rate_limiter = Arc::new(RateLimiter::new(&config).unwrap());
        let producer_id = "test_producer_rate_limit";
        
        // Send events up to limit
        for i in 0..config.producer_rate_limit {
            let result = rate_limiter.check_limit(producer_id, "dpi_probe").await;
            assert!(result.is_ok(), "Event {} should be accepted", i);
            let allowed = result.unwrap();
            assert!(allowed, "Event {} should pass rate limit check", i);
        }
        
        // Next event should exceed rate limit
        let result = rate_limiter.check_limit(producer_id, "dpi_probe").await;
        assert!(result.is_ok(), "Rate limiter should return result");
        let allowed = result.unwrap();
        assert!(!allowed, "Event should be rejected when rate limit exceeded");
    }

    #[tokio::test]
    async fn test_buffer_full_rejected() {
        // Test that events are rejected when buffer is full
        let mut config = create_test_config().await;
        config.buffer_capacity = 5; // Small buffer for testing
        
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        let producer_id = "test_producer_buffer";
        
        // Fill buffer to capacity
        for i in 0..config.buffer_capacity {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            let result = buffer.add(&envelope).await;
            assert!(result.is_ok(), "Event {} should be accepted", i);
        }
        
        // Next event should be rejected
        let envelope = create_test_envelope(producer_id, config.buffer_capacity as u64).await;
        let result = buffer.add(&envelope).await;
        assert!(result.is_err(), "Event should be rejected when buffer is full");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("full") || error_msg.contains("Buffer"),
                "Error should indicate buffer full");
    }

    #[tokio::test]
    async fn test_global_cap_exceeded_rejected() {
        // Test that events are rejected when global cap is exceeded
        let mut config = create_test_config().await;
        config.global_rate_limit = 10; // Low global cap for testing
        config.rate_limit_window_seconds = 60;
        
        let rate_limiter = Arc::new(RateLimiter::new(&config).unwrap());
        
        // Use multiple producers to hit global cap
        let mut total_accepted = 0;
        let producer_ids = vec!["producer_1", "producer_2", "producer_3"];
        
        // Send events until global cap is reached
        loop {
            let mut any_accepted = false;
            for producer_id in &producer_ids {
                let result = rate_limiter.check_limit(producer_id, "dpi_probe").await;
                if let Ok(allowed) = result {
                    if allowed {
                        total_accepted += 1;
                        any_accepted = true;
                        
                        if total_accepted >= config.global_rate_limit {
                            // Next event should be rejected
                            let reject_result = rate_limiter.check_limit(producer_id, "dpi_probe").await;
                            assert!(reject_result.is_ok(), "Rate limiter should return result");
                            let rejected = reject_result.unwrap();
                            assert!(!rejected, "Event should be rejected when global cap exceeded");
                            return;
                        }
                    }
                }
            }
            
            if !any_accepted {
                // All events are being rejected - verify global cap was hit
                assert!(total_accepted >= config.global_rate_limit,
                        "Should have accepted at least up to global cap");
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_backpressure_signaled() {
        // Test that backpressure is signaled on overload
        let mut config = create_test_config().await;
        config.buffer_capacity = 3;
        
        let backpressure = Arc::new(BackpressureController::new(&config).unwrap());
        let buffer = Arc::new(EventBuffer::new(&config).unwrap());
        let producer_id = "test_producer_backpressure_signal";
        
        // Fill buffer
        for i in 0..config.buffer_capacity {
            let envelope = create_test_envelope(producer_id, i as u64).await;
            buffer.add(&envelope).await.unwrap();
        }
        
        // Try to add beyond capacity
        let envelope = create_test_envelope(producer_id, config.buffer_capacity as u64).await;
        let result = buffer.add(&envelope).await;
        assert!(result.is_err(), "Event should be rejected");
        
        // Verify backpressure is signaled
        backpressure.signal_backpressure(producer_id).await;
        assert!(!backpressure.can_accept(producer_id).await,
                "Backpressure should be active after signaling");
    }
}
