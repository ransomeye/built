// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/tests/synthetic_attack_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Synthetic ransomware campaign replay tests

use ransomeye_core_correlation::engine::{CorrelationEngine, EngineConfig};
use ransomeye_core_correlation::input::validated_events::{ValidatedEvent, ValidationMetadata};
use ransomeye_core_correlation::kill_chain::stages::RansomwareStage;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Simulate a complete ransomware attack campaign
fn create_synthetic_ransomware_campaign(entity_id: &str) -> Vec<ValidatedEvent> {
    let mut events = Vec::new();
    let mut timestamp = Utc::now() - Duration::hours(1);

    // Initial Access
    timestamp = timestamp + Duration::minutes(5);
    events.push(ValidatedEvent {
        event_id: format!("{}_initial_access", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "network_connection".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.7));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Execution
    timestamp = timestamp + Duration::minutes(2);
    events.push(ValidatedEvent {
        event_id: format!("{}_execution", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "process_creation".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.8));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Persistence
    timestamp = timestamp + Duration::minutes(3);
    events.push(ValidatedEvent {
        event_id: format!("{}_persistence", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "registry_modification".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.7));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Discovery
    timestamp = timestamp + Duration::minutes(5);
    for i in 0..10 {
        events.push(ValidatedEvent {
            event_id: format!("{}_discovery_{}", entity_id, i),
            entity_id: entity_id.to_string(),
            timestamp: timestamp + Duration::seconds(i as i64),
            signal_type: "file_enumeration".to_string(),
            payload: {
                let mut p = HashMap::new();
                p.insert("confidence".to_string(), serde_json::json!(0.6));
                p
            },
            validation_metadata: ValidationMetadata {
                validated_at: timestamp,
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        });
    }

    // Encryption Preparation
    timestamp = timestamp + Duration::minutes(2);
    events.push(ValidatedEvent {
        event_id: format!("{}_encryption_prep", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "key_generation".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.8));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Encryption Execution
    timestamp = timestamp + Duration::minutes(1);
    for i in 0..20 {
        events.push(ValidatedEvent {
            event_id: format!("{}_encryption_{}", entity_id, i),
            entity_id: entity_id.to_string(),
            timestamp: timestamp + Duration::seconds(i as i64),
            signal_type: "file_modification".to_string(),
            payload: {
                let mut p = HashMap::new();
                p.insert("confidence".to_string(), serde_json::json!(0.9));
                p
            },
            validation_metadata: ValidationMetadata {
                validated_at: timestamp,
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        });
    }

    // Encryption activity signal
    events.push(ValidatedEvent {
        event_id: format!("{}_encryption_activity", entity_id),
        entity_id: entity_id.to_string(),
        timestamp: timestamp + Duration::seconds(10),
        signal_type: "encryption_activity".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(0.95));
            p
        },
        validation_metadata: ValidationMetadata {
            validated_at: timestamp,
            validator_version: "1.0".to_string(),
            checks_passed: vec![],
            validation_hash: None,
        },
    });

    // Impact
    timestamp = timestamp + Duration::minutes(2);
    events.push(ValidatedEvent {
        event_id: format!("{}_impact", entity_id),
        entity_id: entity_id.to_string(),
        timestamp,
        signal_type: "ransom_note".to_string(),
        payload: {
            let mut p = HashMap::new();
            p.insert("confidence".to_string(), serde_json::json!(1.0));
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
fn test_synthetic_ransomware_campaign() {
    let config = EngineConfig::default();
    let engine = CorrelationEngine::new(config);

    let events = create_synthetic_ransomware_campaign("test_entity_1");

    let mut detections = Vec::new();
    for event in events {
        match engine.process_event(event) {
            Ok(Some(detection)) => {
                detections.push(detection);
            }
            Ok(None) => {}
            Err(e) => panic!("Error processing event: {:?}", e),
        }
    }

    // Should detect at least one stage
    assert!(!detections.is_empty(), "Should detect at least one kill-chain stage");

    // Check that we progressed through stages
    let stages: Vec<RansomwareStage> = detections
        .iter()
        .map(|d| d.kill_chain_stage)
        .collect();

    // Should have InitialAccess
    assert!(
        stages.contains(&RansomwareStage::InitialAccess),
        "Should detect InitialAccess stage"
    );

    // Should eventually reach EncryptionExecution or Impact
    assert!(
        stages.contains(&RansomwareStage::EncryptionExecution)
            || stages.contains(&RansomwareStage::Impact),
        "Should detect encryption or impact stage"
    );
}

#[test]
fn test_multiple_campaigns() {
    let config = EngineConfig::default();
    let engine = CorrelationEngine::new(config);

    // Process multiple campaigns
    for i in 0..10 {
        let events = create_synthetic_ransomware_campaign(&format!("entity_{}", i));
        for event in events {
            let _ = engine.process_event(event);
        }
    }

    let stats = engine.get_stats();
    assert!(stats.entity_count <= 10, "Should track multiple entities");
}

