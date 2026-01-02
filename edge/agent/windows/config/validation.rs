// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration validation for Windows Agent

use serde::{Serialize, Deserialize};
use tracing::error;

#[path = "../agent/src/errors.rs"]
mod errors;
use errors::AgentError;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_buffer_size_mb: u64,
    pub backpressure_threshold: f64,
    pub max_events_per_second: u64,
    pub max_processes: usize,
    pub max_tracked_paths: usize,
    pub max_connections: usize,
    pub mass_write_threshold: u64,
    pub monitored_registry_keys: Vec<String>,
}

impl Config {
    /// Load configuration
    pub fn load() -> Result<Self, AgentError> {
        // In real implementation, would load from file or environment
        Ok(Self::default())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.max_buffer_size_mb == 0 {
            return Err(AgentError::ConfigurationError(
                "max_buffer_size_mb must be greater than 0".to_string()
            ));
        }
        
        if self.backpressure_threshold <= 0.0 || self.backpressure_threshold > 1.0 {
            return Err(AgentError::ConfigurationError(
                "backpressure_threshold must be between 0 and 1".to_string()
            ));
        }
        
        if self.max_events_per_second == 0 {
            return Err(AgentError::ConfigurationError(
                "max_events_per_second must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_buffer_size_mb: 100,
            backpressure_threshold: 0.8,
            max_events_per_second: 10000,
            max_processes: 10000,
            max_tracked_paths: 50000,
            max_connections: 10000,
            mass_write_threshold: 100,
            monitored_registry_keys: vec![
                "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
                "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            ],
        }
    }
}
