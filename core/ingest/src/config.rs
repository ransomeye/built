// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/config.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration management - loads and validates configuration

/*
 * Configuration Management
 * 
 * Loads and validates configuration from environment variables.
 * Fails-closed on invalid configuration.
 */

use std::env;
use tracing::error;

// Import validation from config directory
#[path = "../config/validation.rs"]
mod validation;
use validation::{ConfigValidator, ValidationError};

#[derive(Clone)]
pub struct Config {
    pub listen_address: String,
    pub control_plane_address: String,
    pub buffer_capacity: usize,
    pub producer_rate_limit: u64,
    pub global_rate_limit: u64,
    pub rate_limit_window_seconds: u64,
    pub backpressure_clear_seconds: u64,
    pub trust_store_path: String,
    pub crl_path: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let listen_address = env::var("RANSOMEYE_INGESTION_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        ConfigValidator::validate_address(&listen_address)
            .map_err(|e| format!("Invalid listen address: {}", e))?;
        
        let control_plane_address = env::var("RANSOMEYE_CONTROL_PLANE_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:9090".to_string());
        ConfigValidator::validate_address(&control_plane_address)
            .map_err(|e| format!("Invalid control plane address: {}", e))?;
        
        let buffer_capacity = env::var("RANSOMEYE_BUFFER_CAPACITY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<usize>()
            .map_err(|e| format!("Invalid buffer capacity: {}", e))?;
        ConfigValidator::validate_buffer_capacity(buffer_capacity)?;
        
        let producer_rate_limit = env::var("RANSOMEYE_PRODUCER_RATE_LIMIT")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid producer rate limit: {}", e))?;
        ConfigValidator::validate_producer_rate_limit(producer_rate_limit)?;
        
        let global_rate_limit = env::var("RANSOMEYE_GLOBAL_RATE_LIMIT")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid global rate limit: {}", e))?;
        ConfigValidator::validate_global_rate_limit(global_rate_limit)?;
        
        let rate_limit_window_seconds = env::var("RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid rate limit window: {}", e))?;
        ConfigValidator::validate_rate_limit_window(rate_limit_window_seconds)?;
        
        let backpressure_clear_seconds = env::var("RANSOMEYE_BACKPRESSURE_CLEAR_SECONDS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid backpressure clear seconds: {}", e))?;
        ConfigValidator::validate_backpressure_clear_seconds(backpressure_clear_seconds)?;
        
        let trust_store_path = env::var("RANSOMEYE_TRUST_STORE_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/trust_store".to_string());
        // Validate trust store path if it exists (allow non-existent for default)
        if std::path::Path::new(&trust_store_path).exists() {
            ConfigValidator::validate_readable_directory(&trust_store_path)?;
        }
        
        let crl_path = env::var("RANSOMEYE_CRL_PATH").ok();
        if let Some(ref crl) = crl_path {
            if std::path::Path::new(crl).exists() {
                ConfigValidator::validate_readable_file(crl)?;
            }
        }
        
        Ok(Config {
            listen_address,
            control_plane_address,
            buffer_capacity,
            producer_rate_limit,
            global_rate_limit,
            rate_limit_window_seconds,
            backpressure_clear_seconds,
            trust_store_path,
            crl_path,
        })
    }
}

