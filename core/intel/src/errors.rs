// Path and File Name : /home/ransomeye/rebuild/core/intel/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Error types for intel correlation and scoring

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntelError {
    #[error("Insufficient confidence: {0}")]
    InsufficientConfidence(String),
    #[error("No signals to correlate: {0}")]
    NoSignals(String),
    #[error("Correlation failed: {0}")]
    CorrelationFailed(String),
    #[error("Scoring failed: {0}")]
    ScoringFailed(String),
}

