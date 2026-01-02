// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/input/validated_events.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Validated event types from Phase 4 - events that passed validation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validated event from Phase 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedEvent {
    /// Event identifier
    pub event_id: String,
    /// Entity identifier (endpoint, process, etc.)
    pub entity_id: String,
    /// Event timestamp (event time, not processing time)
    pub timestamp: DateTime<Utc>,
    /// Signal type
    pub signal_type: String,
    /// Event data payload
    pub payload: HashMap<String, serde_json::Value>,
    /// Validation metadata from Phase 4
    pub validation_metadata: ValidationMetadata,
}

/// Validation metadata from Phase 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetadata {
    /// Validation timestamp
    pub validated_at: DateTime<Utc>,
    /// Validator version
    pub validator_version: String,
    /// Validation checks passed
    pub checks_passed: Vec<String>,
    /// Validation signature/hash
    pub validation_hash: Option<String>,
}

/// Event validation result
#[derive(Debug, Clone)]
pub enum EventValidationResult {
    Valid(ValidatedEvent),
    Invalid(String),
    OutOfOrder(String),
}

/// Event validator (validates events from Phase 4)
pub struct EventValidator;

impl EventValidator {
    /// Validate event structure
    pub fn validate(event: &ValidatedEvent) -> EventValidationResult {
        // Check required fields
        if event.event_id.is_empty() {
            return EventValidationResult::Invalid("Empty event_id".to_string());
        }

        if event.entity_id.is_empty() {
            return EventValidationResult::Invalid("Empty entity_id".to_string());
        }

        if event.signal_type.is_empty() {
            return EventValidationResult::Invalid("Empty signal_type".to_string());
        }

        // Check timestamp is reasonable (not too far in future)
        let now = Utc::now();
        let max_future_seconds = 300; // 5 minutes
        if (event.timestamp - now).num_seconds() > max_future_seconds {
            return EventValidationResult::Invalid(format!(
                "Event timestamp too far in future: {:?}",
                event.timestamp
            ));
        }

        EventValidationResult::Valid(event.clone())
    }

    /// Check if event is out of order (relative to processing time)
    pub fn check_ordering(
        event: &ValidatedEvent,
        last_processed_timestamp: Option<DateTime<Utc>>,
    ) -> bool {
        if let Some(last_ts) = last_processed_timestamp {
            // Allow small clock skew (5 minutes)
            let skew_allowance = chrono::Duration::minutes(5);
            event.timestamp >= last_ts - skew_allowance
        } else {
            true // First event is always in order
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_validation() {
        let event = ValidatedEvent {
            event_id: "e1".to_string(),
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signal_type: "test".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        match EventValidator::validate(&event) {
            EventValidationResult::Valid(_) => {}
            _ => panic!("Valid event should pass validation"),
        }
    }
}

