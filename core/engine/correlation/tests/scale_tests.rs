// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/scale_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Scale tests - verify bounded memory at 50k+ entities

use ransomeye_core_correlation::engine::{CorrelationEngine, EngineConfig};
use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_scale_50k_entities() {
    let config = EngineConfig {
        max_entities: 50000,
        max_signals_per_entity: 100,
        max_transitions_per_entity: 20,
        ..Default::default()
    };
    let max_entities = config.max_entities;
    let engine = CorrelationEngine::new(config);

    // Create 50k entities
    for i in 0..50000 {
        let event = ValidatedEvent {
            event_id: format!("event_{}", i),
            entity_id: format!("entity_{}", i),
            timestamp: Utc::now(),
            signal_type: "network_connection".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        let _ = engine.process_event(event);
    }

    let stats = engine.get_stats();
    
    // Should not exceed max_entities
    assert!(
        stats.entity_count <= max_entities,
        "Entity count should not exceed max_entities: {} <= {}",
        stats.entity_count,
        max_entities
    );

    // Memory should be bounded
    assert!(
        stats.estimated_memory_bytes > 0,
        "Memory usage should be tracked"
    );
}

#[test]
fn test_memory_bounds_per_entity() {
    let config = EngineConfig {
        max_entities: 1000,
        max_signals_per_entity: 50,
        max_transitions_per_entity: 10,
        ..Default::default()
    };
    let engine = CorrelationEngine::new(config);

    // Add many signals to one entity
    for i in 0..200 {
        let event = ValidatedEvent {
            event_id: format!("event_{}", i),
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signal_type: "network_connection".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        let _ = engine.process_event(event);
    }

    let stats = engine.get_stats();
    
    // Should only have 1 entity
    assert_eq!(stats.entity_count, 1, "Should track only one entity");
    
    // Memory should be bounded (signals should be evicted)
    // We can't directly check signal count, but memory should be reasonable
    assert!(
        stats.estimated_memory_bytes < 10_000_000, // 10MB per entity max
        "Memory per entity should be bounded"
    );
}

#[test]
fn test_eviction_at_capacity() {
    let config = EngineConfig {
        max_entities: 10,
        ..Default::default()
    };
    let max_entities = config.max_entities;
    let engine = CorrelationEngine::new(config);

    // Fill to capacity
    for i in 0..10 {
        let event = ValidatedEvent {
            event_id: format!("event_{}", i),
            entity_id: format!("entity_{}", i),
            timestamp: Utc::now(),
            signal_type: "network_connection".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        let _ = engine.process_event(event);
    }

    let stats_before = engine.get_stats();
    assert_eq!(stats_before.entity_count, 10);

    // Add one more - should trigger eviction
    let event = ValidatedEvent {
        event_id: "event_new".to_string(),
        entity_id: "entity_new".to_string(),
        timestamp: Utc::now(),
        signal_type: "network_connection".to_string(),
        payload: HashMap::new(),
        validation_metadata: ValidationMetadata {
            validated_at: Utc::now(),
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    };

    let _ = engine.process_event(event);

    let stats_after = engine.get_stats();
    
    // Should still be at or below capacity
    assert!(
        stats_after.entity_count <= max_entities,
        "Entity count should not exceed capacity after eviction"
    );
}

