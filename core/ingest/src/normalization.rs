// Path and File Name : /home/ransomeye/rebuild/core/ingest/src/normalization.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event normalization - normalizes events to standard format with fail-closed deception enforcement

/*
 * Event Normalization
 * 
 * Normalizes events to standard format.
 * Ensures consistent event structure.
 * FAIL-CLOSED: Deception events are always HIGH/CRITICAL and cannot be downgraded.
 */

use crate::protocol::event_envelope::EventEnvelope;
use serde_json::Value;
use tracing::{warn, error};

pub struct EventNormalizer;

impl EventNormalizer {
    pub fn normalize(envelope: &EventEnvelope) -> Result<EventEnvelope, Box<dyn std::error::Error>> {
        // Normalize envelope fields
        let mut normalized = envelope.clone();
        
        // Normalize component type to lowercase
        normalized.component_type = normalized.component_type.to_lowercase();
        
        // Normalize producer ID (trim whitespace)
        normalized.producer_id = normalized.producer_id.trim().to_string();
        
        // FAIL-CLOSED: Detect and enforce deception events
        if let Err(e) = Self::enforce_deception_priority(&mut normalized) {
            error!("Failed to enforce deception priority: {}", e);
            return Err(e);
        }
        
        Ok(normalized)
    }
    
    /// FAIL-CLOSED: Enforce that deception events are always HIGH or CRITICAL
    /// Deception events CANNOT be downgraded to INFO or WARN
    fn enforce_deception_priority(envelope: &mut EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Parse event data to check if it's a deception event
        let event_data: Value = serde_json::from_str(&envelope.event_data)
            .map_err(|e| format!("Failed to parse event data: {}", e))?;
        
        // Check if event_type indicates deception
        let is_deception = event_data.get("event_type")
            .and_then(|v| v.as_str())
            .map(|s| s == "deception")
            .unwrap_or(false);
        
        // Check if event contains deception metadata
        let has_deception_metadata = event_data.get("severity")
            .and_then(|v| v.as_str())
            .map(|s| s == "HIGH" || s == "CRITICAL")
            .unwrap_or(false);
        
        if is_deception || has_deception_metadata {
            // This is a deception event - enforce HIGH or CRITICAL priority
            let current_priority = envelope.priority.to_uppercase();
            
            // FAIL-CLOSED: If priority is INFO or WARN, upgrade to HIGH
            if current_priority == "INFO" || current_priority == "WARN" {
                warn!("DECEPTION EVENT DETECTED: Upgrading priority from {} to HIGH (fail-closed enforcement)", 
                      current_priority);
                envelope.priority = "HIGH".to_string();
            }
            
            // Ensure priority is HIGH or CRITICAL (never downgrade CRITICAL)
            if current_priority != "CRITICAL" && current_priority != "HIGH" {
                envelope.priority = "HIGH".to_string();
            }
            
            // Add deception marker to event data
            if let Ok(mut data) = serde_json::from_str::<Value>(&envelope.event_data) {
                data.as_object_mut()
                    .and_then(|obj| {
                        obj.insert("deception_event".to_string(), Value::Bool(true));
                        obj.insert("deception_enforced".to_string(), Value::Bool(true));
                        Some(())
                    });
                envelope.event_data = serde_json::to_string(&data)
                    .unwrap_or_else(|_| envelope.event_data.clone());
            }
        }
        
        Ok(())
    }
}

