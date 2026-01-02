// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/determinism_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Determinism tests - verify identical inputs produce identical outputs

use ransomeye_core_correlation::engine::{CorrelationEngine, EngineConfig};
use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use chrono::Utc;
use std::collections::HashMap;

fn create_test_events() -> Vec<ValidatedEvent> {
    vec![
        ValidatedEvent {
            event_id: "event1".to_string(),
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
        },
        ValidatedEvent {
            event_id: "event2".to_string(),
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signal_type: "process_creation".to_string(),
            payload: {
                let mut p = HashMap::new();
                p.insert("confidence".to_string(), serde_json::json!(0.8));
                p
            },
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        },
    ]
}

#[test]
fn test_deterministic_replay() {
    let config = EngineConfig::default();
    let events = create_test_events();

    // First run
    let engine1 = CorrelationEngine::new(config.clone());
    let mut detections1 = Vec::new();
    for event in events.clone() {
        if let Ok(Some(detection)) = engine1.process_event(event) {
            detections1.push(detection);
        }
    }

    // Second run with identical events
    let engine2 = CorrelationEngine::new(config);
    let mut detections2 = Vec::new();
    for event in events {
        if let Ok(Some(detection)) = engine2.process_event(event) {
            detections2.push(detection);
        }
    }

    // Should produce same number of detections
    assert_eq!(
        detections1.len(),
        detections2.len(),
        "Identical inputs should produce same number of detections"
    );

    // If detections exist, they should have same stages
    if !detections1.is_empty() && !detections2.is_empty() {
        let stages1: Vec<_> = detections1.iter().map(|d| d.kill_chain_stage).collect();
        let stages2: Vec<_> = detections2.iter().map(|d| d.kill_chain_stage).collect();
        
        assert_eq!(
            stages1, stages2,
            "Identical inputs should produce same kill-chain stages"
        );
    }
}

#[test]
fn test_deterministic_confidence_calculation() {
    let config = EngineConfig::default();
    let events = create_test_events();

    // Run twice
    let engine1 = CorrelationEngine::new(config.clone());
    let engine2 = CorrelationEngine::new(config);

    let mut confidences1 = Vec::new();
    let mut confidences2 = Vec::new();

    for event in events.clone() {
        if let Ok(Some(detection)) = engine1.process_event(event.clone()) {
            confidences1.push(detection.confidence);
        }
        if let Ok(Some(detection)) = engine2.process_event(event) {
            confidences2.push(detection.confidence);
        }
    }

    // Confidences should match (within floating point precision)
    assert_eq!(
        confidences1.len(),
        confidences2.len(),
        "Should have same number of confidence scores"
    );

    for (c1, c2) in confidences1.iter().zip(confidences2.iter()) {
        assert!(
            (c1 - c2).abs() < 0.0001,
            "Confidence scores should match: {} vs {}",
            c1,
            c2
        );
    }
}

#[test]
fn test_deterministic_state_transitions() {
    let config = EngineConfig::default();
    let events = create_test_events();

    // Run twice and check state transitions are identical
    let engine1 = CorrelationEngine::new(config.clone());
    let engine2 = CorrelationEngine::new(config);

    for event in events.clone() {
        let _ = engine1.process_event(event.clone());
        let _ = engine2.process_event(event);
    }

    // Both engines should have same entity count
    let stats1 = engine1.get_stats();
    let stats2 = engine2.get_stats();

    assert_eq!(
        stats1.entity_count,
        stats2.entity_count,
        "Entity counts should match"
    );
}

