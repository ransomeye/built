// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/input/normalization.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Event normalization - converts validated events to internal signal format

use crate::input::validated_events::ValidatedEvent;
use crate::kill_chain::rules::Signal;
use chrono::Utc;
use std::collections::HashMap;

/// Event normalizer
pub struct EventNormalizer;

impl EventNormalizer {
    /// Normalize validated event to signal
    pub fn normalize(event: &ValidatedEvent) -> Signal {
        // Extract confidence from payload if present
        let confidence = event
            .payload
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        Signal {
            signal_type: event.signal_type.clone(),
            timestamp: event.timestamp,
            entity_id: event.entity_id.clone(),
            confidence,
            metadata: event
                .payload
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| v.to_string()),
                    )
                })
                .collect(),
        }
    }

    /// Normalize batch of events
    pub fn normalize_batch(events: &[ValidatedEvent]) -> Vec<Signal> {
        events.iter().map(Self::normalize).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::validated_events::ValidationMetadata;

    #[test]
    fn test_normalization() {
        let mut payload = HashMap::new();
        payload.insert("confidence".to_string(), serde_json::json!(0.8));
        
        let event = ValidatedEvent {
            event_id: "e1".to_string(),
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signal_type: "test_signal".to_string(),
            payload,
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        let signal = EventNormalizer::normalize(&event);
        assert_eq!(signal.signal_type, "test_signal");
        assert_eq!(signal.confidence, 0.8);
    }
}

