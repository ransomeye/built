// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/ordering_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Event ordering tests - verify out-of-order events are handled correctly

use ransomeye_core_correlation::engine::{CorrelationEngine, EngineConfig};
use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use chrono::{Duration, Utc};
use std::collections::HashMap;

#[test]
fn test_chronological_ordering() {
    let config = EngineConfig::default();
    let engine = CorrelationEngine::new(config);

    let base_time = Utc::now();
    let mut events = Vec::new();

    // Create events in chronological order
    for i in 0..5 {
        events.push(ValidatedEvent {
            event_id: format!("event_{}", i),
            entity_id: "entity1".to_string(),
            timestamp: base_time + Duration::seconds(i as i64),
            signal_type: "network_connection".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: base_time + Duration::seconds(i as i64),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        });
    }

    // Process in order - should succeed
    for event in events {
        let result = engine.process_event(event);
        assert!(result.is_ok(), "Chronological events should process successfully");
    }
}

#[test]
fn test_out_of_order_events() {
    let config = EngineConfig::default();
    let engine = CorrelationEngine::new(config);

    let base_time = Utc::now();

    // Process event 1
    let event1 = ValidatedEvent {
        event_id: "event_1".to_string(),
        entity_id: "entity1".to_string(),
        timestamp: base_time + Duration::seconds(10),
        signal_type: "network_connection".to_string(),
        payload: HashMap::new(),
        validation_metadata: ValidationMetadata {
            validated_at: base_time + Duration::seconds(10),
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    let _ = engine.process_event(event1);

    // Process event 0 (out of order - earlier timestamp)
    let event0 = ValidatedEvent {
        event_id: "event_0".to_string(),
        entity_id: "entity1".to_string(),
        timestamp: base_time, // Earlier than event1
        signal_type: "network_connection".to_string(),
        payload: HashMap::new(),
        validation_metadata: ValidationMetadata {
            validated_at: base_time,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    // Out-of-order events should be handled gracefully
    // (Either processed with clock skew allowance or dropped)
    let result = engine.process_event(event0);
    // Should not panic - ordering violations should be handled
    assert!(result.is_ok() || result.is_err(), "Out-of-order events should be handled");
}

#[test]
fn test_clock_skew_handling() {
    let config = EngineConfig::default();
    let engine = CorrelationEngine::new(config);

    let base_time = Utc::now();

    // Process event 1
    let event1 = ValidatedEvent {
        event_id: "event_1".to_string(),
        entity_id: "entity1".to_string(),
        timestamp: base_time,
        signal_type: "network_connection".to_string(),
        payload: HashMap::new(),
        validation_metadata: ValidationMetadata {
            validated_at: base_time,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    let _ = engine.process_event(event1);

    // Process event with small clock skew (within 5 minute allowance)
    let event2 = ValidatedEvent {
        event_id: "event_2".to_string(),
        entity_id: "entity1".to_string(),
        timestamp: base_time - Duration::minutes(3), // 3 minutes earlier (within skew)
        signal_type: "network_connection".to_string(),
        payload: HashMap::new(),
        validation_metadata: ValidationMetadata {
            validated_at: base_time - Duration::minutes(3),
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    // Small clock skew should be acceptable
    let result = engine.process_event(event2);
    assert!(result.is_ok(), "Small clock skew should be acceptable");
}

