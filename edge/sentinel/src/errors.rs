// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/errors.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Sentinel error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SentinelError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Hardening error: {0}")]
    HardeningError(String),
    #[error("Monitor error: {0}")]
    MonitorError(String),
    #[error("Alert error: {0}")]
    AlertError(String),
}

