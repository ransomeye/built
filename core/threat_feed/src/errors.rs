// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for threat feed processing

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThreatFeedError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),
    #[error("Stale intel: {0}")]
    StaleIntel(String),
    #[error("Replay detected: {0}")]
    ReplayDetected(String),
    #[error("Feed unavailable: {0}")]
    FeedUnavailable(String),
    #[error("Malformed intel: {0}")]
    MalformedIntel(String),
    #[error("Source attribution missing: {0}")]
    SourceAttributionMissing(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

