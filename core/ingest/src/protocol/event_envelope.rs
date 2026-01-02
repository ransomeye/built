// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/protocol/event_envelope.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event envelope structure - defines the structure of all incoming events

/*
 * Event Envelope
 * 
 * Defines the structure of all incoming events.
 * Every event must include all required fields.
 */

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Producer ID (unique per producer instance)
    pub producer_id: String,
    
    /// Component type (dpi_probe, linux_agent, windows_agent)
    pub component_type: String,
    
    /// Schema version
    pub schema_version: u32,
    
    /// Event timestamp (ISO 8601)
    pub timestamp: DateTime<Utc>,
    
    /// Sequence number (per producer, monotonically increasing)
    pub sequence_number: u64,
    
    /// Cryptographic signature (RSA-4096-PSS-SHA256, base64 encoded)
    pub signature: String,
    
    /// Integrity hash (SHA-256 of event data, hex encoded)
    pub integrity_hash: String,
    
    /// Nonce (unique per event, for replay protection)
    pub nonce: String,
    
    /// Event data (JSON string)
    pub event_data: String,
    
    /// Event priority (INFO, WARN, CRITICAL)
    /// Defaults to INFO if not specified
    #[serde(default = "default_priority")]
    pub priority: String,
}

fn default_priority() -> String {
    "INFO".to_string()
}

impl EventEnvelope {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.producer_id.is_empty() {
            return Err("Producer ID is required".into());
        }
        if self.component_type.is_empty() {
            return Err("Component type is required".into());
        }
        if self.signature.is_empty() {
            return Err("Signature is required".into());
        }
        if self.integrity_hash.is_empty() {
            return Err("Integrity hash is required".into());
        }
        if self.nonce.is_empty() {
            return Err("Nonce is required".into());
        }
        if self.event_data.is_empty() {
            return Err("Event data is required".into());
        }
        Ok(())
    }
}

