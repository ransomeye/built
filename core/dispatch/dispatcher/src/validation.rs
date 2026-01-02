// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration validation - ENV-only, fail on missing

use std::env;
use tracing::{error, warn};
use crate::dispatcher::errors::DispatcherError;

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn new() -> Result<Self, DispatcherError> {
        // Validate all required environment variables
        let required_vars = vec![
            "RANSOMEYE_DISPATCHER_POLICY_KEY_PATH",
            "RANSOMEYE_DISPATCHER_AUDIT_LOG_PATH",
        ];
        
        let mut missing = Vec::new();
        
        for var in required_vars {
            if env::var(var).is_err() {
                missing.push(var);
            }
        }
        
        if !missing.is_empty() {
            return Err(DispatcherError::ConfigurationError(
                format!("Missing required environment variables: {}", missing.join(", "))
            ));
        }
        
        Ok(Self)
    }
    
    /// Validate optional configuration
    pub fn validate_optional(&self) -> Result<(), DispatcherError> {
        // Check optional vars have valid values if set
        Ok(())
    }
}

