// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/output.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Telemetry output formatting and serialization for Core transmission

use serde::{Serialize, Deserialize};
use crate::signing::SignedEvent;

/// Formats a signed event for transmission to Core
/// Ensures proper JSON serialization with required fields
pub struct OutputFormatter;

impl OutputFormatter {
    /// Serialize signed event to JSON bytes for transmission
    pub fn to_json_bytes(event: &SignedEvent) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(event)
    }
    
    /// Serialize signed event to JSON string for logging/debugging
    pub fn to_json_string(event: &SignedEvent) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(event)
    }
    
    /// Validate that event has all required fields before transmission
    pub fn validate_event(event: &SignedEvent) -> bool {
        !event.message_id.is_empty()
            && !event.component_identity.is_empty()
            && !event.nonce.is_empty()
            && !event.signature.is_empty()
            && !event.data_hash.is_empty()
            && !event.data.is_null()
    }
    
    /// Get event size in bytes (estimated)
    pub fn estimate_size(event: &SignedEvent) -> usize {
        // Rough estimate - actual size may vary based on JSON formatting
        event.message_id.len()
            + event.component_identity.len()
            + event.nonce.len()
            + event.signature.len()
            + event.data_hash.len()
            + event.data.to_string().len()
            + 200 // Overhead for JSON structure, timestamp, etc.
    }
}
