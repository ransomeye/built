// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/common.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Common test utilities

use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use chrono::Utc;
use std::collections::HashMap;

pub fn create_test_event(
    event_id: &str,
    entity_id: &str,
    signal_type: &str,
    confidence: f64,
) -> ValidatedEvent {
    let mut payload = HashMap::new();
    payload.insert("confidence".to_string(), serde_json::json!(confidence));

    ValidatedEvent {
        event_id: event_id.to_string(),
        entity_id: entity_id.to_string(),
        timestamp: Utc::now(),
        signal_type: signal_type.to_string(),
        payload,
        validation_metadata: ValidationMetadata {
            validated_at: Utc::now(),
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    }
}

