// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: ENV-only configuration validation - fail-closed

use std::env;

/// DPI Probe configuration
/// 
/// All configuration from environment variables.
/// Missing required ENV → startup FAIL (fail-closed).
pub struct ProbeConfig {
    pub capture_interface: String,
    pub max_flows: usize,
    pub max_queue_size: usize,
    pub rate_limit_tokens: u64,
    pub rate_limit_refill: u64,
    pub identity_path: Option<String>,
    pub signing_key_path: Option<String>,
}

impl ProbeConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let capture_interface = env::var("CAPTURE_IFACE")
            .map_err(|_| "CAPTURE_IFACE environment variable is required but not set")?;
        
        let max_flows = env::var("DPI_MAX_FLOWS")
            .unwrap_or_else(|_| "1000000".to_string())
            .parse::<usize>()
            .map_err(|_| "DPI_MAX_FLOWS must be a valid integer")?;
        
        let max_queue_size = env::var("DPI_MAX_QUEUE_SIZE")
            .unwrap_or_else(|_| "100000".to_string())
            .parse::<usize>()
            .map_err(|_| "DPI_MAX_QUEUE_SIZE must be a valid integer")?;
        
        let rate_limit_tokens = env::var("DPI_RATE_LIMIT_TOKENS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<u64>()
            .map_err(|_| "DPI_RATE_LIMIT_TOKENS must be a valid integer")?;
        
        let rate_limit_refill = env::var("DPI_RATE_LIMIT_REFILL")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .map_err(|_| "DPI_RATE_LIMIT_REFILL must be a valid integer")?;
        
        let identity_path = env::var("DPI_IDENTITY_PATH").ok();
        let signing_key_path = env::var("DPI_SIGNING_KEY_PATH").ok();
        
        Ok(ProbeConfig {
            capture_interface,
            max_flows,
            max_queue_size,
            rate_limit_tokens,
            rate_limit_refill,
            identity_path,
            signing_key_path,
        })
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.capture_interface.is_empty() {
            return Err("CAPTURE_IFACE cannot be empty".to_string());
        }
        
        if self.max_flows == 0 {
            return Err("DPI_MAX_FLOWS must be greater than 0".to_string());
        }
        
        if self.max_queue_size == 0 {
            return Err("DPI_MAX_QUEUE_SIZE must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_requires_capture_interface() {
        env::remove_var("CAPTURE_IFACE");
        assert!(ProbeConfig::from_env().is_err());
    }
    
    #[test]
    fn test_config_succeeds_with_required() {
        env::set_var("CAPTURE_IFACE", "eth0");
        assert!(ProbeConfig::from_env().is_ok());
    }
}
