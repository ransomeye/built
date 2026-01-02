// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/false_positive_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: False positive suppression tests - ensure legitimate activity doesn't trigger detections

use ransomeye_core_correlation::engine::{CorrelationEngine, EngineConfig};
use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Create legitimate activity events (should NOT trigger detection)
fn create_legitimate_activity(entity_id: &str) -> Vec<ValidatedEvent> {
    let mut events = Vec::new();
    let mut timestamp = Utc::now();

    // Normal network connection
    events.push(ValidatedEvent {
        event_id: format!("{}_legit_network", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "network_connection".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.3)); // Low confidence
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Normal process creation
    timestamp = timestamp + Duration::minutes(5);
    events.push(ValidatedEvent {
        event_id: format!("{}_legit_process", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "process_creation".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.2)); // Low confidence
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    events
}

/// Create ambiguous activity (should NOT trigger detection)
fn create_ambiguous_activity(entity_id: &str) -> Vec<ValidatedEvent> {
    let mut events = Vec::new();
    let timestamp = Utc::now();

    // Single isolated signal with low confidence
    events.push(ValidatedEvent {
        event_id: format!("{}_ambiguous", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "file_modification".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.4)); // Below threshold
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    events
}

#[test]
fn test_legitimate_activity_no_detection() {
    let config = EngineConfig {
        min_confidence_threshold: 0.6,
        ..Default::default()
    };
    let engine = CorrelationEngine::new(config);

    let events = create_legitimate_activity("legit_entity");

    let mut detections = Vec::new();
    for event in events {
        match engine.process_event(event) {
            Ok(Some(detection)) => {
                detections.push(detection);
            }
            Ok(None) => {} // Expected - no detection
            Err(e) => panic!("Error processing event: {:?}", e),
        }
    }

    // Should NOT detect legitimate activity
    assert_eq!(
        detections.len(),
        0,
        "Legitimate activity should not trigger detection"
    );
}

#[test]
fn test_ambiguous_activity_no_detection() {
    let config = EngineConfig {
        min_confidence_threshold: 0.6,
        ..Default::default()
    };
    let engine = CorrelationEngine::new(config);

    let events = create_ambiguous_activity("ambiguous_entity");

    let mut detections = Vec::new();
    for event in events {
        match engine.process_event(event) {
            Ok(Some(detection)) => {
                detections.push(detection);
            }
            Ok(None) => {} // Expected - ambiguous correlation → NO ALERT
            Err(e) => panic!("Error processing event: {:?}", e),
        }
    }

    // Ambiguous correlation → NO ALERT
    assert_eq!(
        detections.len(),
        0,
        "Ambiguous activity should not trigger detection (fail-closed)"
    );
}

#[test]
fn test_insufficient_signals_no_detection() {
    let config = EngineConfig {
        min_confidence_threshold: 0.5,
        ..Default::default()
    };
    let engine = CorrelationEngine::new(config);

    // Single signal - insufficient for detection
    let event = ValidatedEvent {
        event_id: "single_signal".to_string(),
        entity_id: "entity1".to_string(),
        timestamp: Utc::now(),
        signal_type: "network_connection".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.7));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: Utc::now(),
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    match engine.process_event(event) {
        Ok(Some(_)) => panic!("Single signal should not trigger detection"),
        Ok(None) => {} // Expected
        Err(_) => {} // Also acceptable (invariant violation)
    }
}

