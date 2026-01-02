// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration validation - validates environment variable values and constraints, fails startup on violation

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
    #[error("Value out of range for {0}: {1} (min: {2}, max: {3})")]
    ValueOutOfRange(String, usize, usize, usize),
    #[error("Path does not exist: {0}")]
    PathNotFound(String),
    #[error("Path is not absolute: {0}")]
    PathNotAbsolute(String),
    #[error("Path is not readable: {0}")]
    PathNotReadable(String),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
}

pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate buffer capacity
    pub fn validate_buffer_capacity(capacity: usize) -> Result<(), ValidationError> {
        const MIN: usize = 1;
        const MAX: usize = 1_000_000;
        
        if capacity < MIN || capacity > MAX {
            return Err(ValidationError::ValueOutOfRange(
                "RANSOMEYE_BUFFER_CAPACITY".to_string(),
                capacity,
                MIN,
                MAX,
            ));
        }
        
        Ok(())
    }
    
    /// Validate producer rate limit
    pub fn validate_producer_rate_limit(limit: u64) -> Result<(), ValidationError> {
        const MIN: u64 = 1;
        const MAX: u64 = 1_000_000;
        
        if limit < MIN || limit > MAX {
            return Err(ValidationError::ValueOutOfRange(
                "RANSOMEYE_PRODUCER_RATE_LIMIT".to_string(),
                limit as usize,
                MIN as usize,
                MAX as usize,
            ));
        }
        
        Ok(())
    }
    
    /// Validate global rate limit
    pub fn validate_global_rate_limit(limit: u64) -> Result<(), ValidationError> {
        const MIN: u64 = 1;
        const MAX: u64 = 10_000_000;
        
        if limit < MIN || limit > MAX {
            return Err(ValidationError::ValueOutOfRange(
                "RANSOMEYE_GLOBAL_RATE_LIMIT".to_string(),
                limit as usize,
                MIN as usize,
                MAX as usize,
            ));
        }
        
        Ok(())
    }
    
    /// Validate rate limit window (seconds)
    pub fn validate_rate_limit_window(window: u64) -> Result<(), ValidationError> {
        const MIN: u64 = 1;
        const MAX: u64 = 3600;
        
        if window < MIN || window > MAX {
            return Err(ValidationError::ValueOutOfRange(
                "RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS".to_string(),
                window as usize,
                MIN as usize,
                MAX as usize,
            ));
        }
        
        Ok(())
    }
    
    /// Validate backpressure clear seconds
    pub fn validate_backpressure_clear_seconds(seconds: u64) -> Result<(), ValidationError> {
        const MIN: u64 = 1;
        const MAX: u64 = 300;
        
        if seconds < MIN || seconds > MAX {
            return Err(ValidationError::ValueOutOfRange(
                "RANSOMEYE_BACKPRESSURE_CLEAR_SECONDS".to_string(),
                seconds as usize,
                MIN as usize,
                MAX as usize,
            ));
        }
        
        Ok(())
    }
    
    /// Validate that path is absolute
    pub fn validate_absolute_path(path: &str) -> Result<(), ValidationError> {
        let path_obj = Path::new(path);
        if !path_obj.is_absolute() {
            return Err(ValidationError::PathNotAbsolute(path.to_string()));
        }
        Ok(())
    }
    
    /// Validate that directory path exists and is readable
    pub fn validate_readable_directory(path: &str) -> Result<(), ValidationError> {
        Self::validate_absolute_path(path)?;
        
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ValidationError::PathNotFound(path.to_string()));
        }
        
        if !path_obj.is_dir() {
            return Err(ValidationError::InvalidValue(
                path.to_string(),
                "Path is not a directory".to_string(),
            ));
        }
        
        // Check read permissions
        std::fs::metadata(path)
            .map_err(|e| ValidationError::PathNotReadable(format!("{}: {}", path, e)))?;
        
        Ok(())
    }
    
    /// Validate that file path exists and is readable
    pub fn validate_readable_file(path: &str) -> Result<(), ValidationError> {
        Self::validate_absolute_path(path)?;
        
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ValidationError::PathNotFound(path.to_string()));
        }
        
        if !path_obj.is_file() {
            return Err(ValidationError::InvalidValue(
                path.to_string(),
                "Path is not a file".to_string(),
            ));
        }
        
        // Check read permissions
        std::fs::File::open(path)
            .map_err(|e| ValidationError::PathNotReadable(format!("{}: {}", path, e)))?;
        
        Ok(())
    }
    
    /// Validate listen address format (IP:port)
    pub fn validate_address(address: &str) -> Result<(), ValidationError> {
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidAddress(
                format!("Invalid address format: {}", address),
            ));
        }
        
        let port = parts[1].parse::<u16>()
            .map_err(|e| ValidationError::InvalidAddress(
                format!("Invalid port in address {}: {}", address, e),
            ))?;
        
        if port == 0 {
            return Err(ValidationError::InvalidAddress(
                format!("Port cannot be 0 in address: {}", address),
            ));
        }
        
        Ok(())
    }
}

