// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: ENV-only configuration validation - fail-closed

use std::env;

/// Linux Agent configuration
/// 
/// All configuration from environment variables.
/// Missing required ENV → startup FAIL (fail-closed).
pub struct AgentConfig {
    pub max_processes: usize,
    pub max_connections: usize,
    pub max_queue_size: usize,
    pub rate_limit_tokens: u64,
    pub rate_limit_refill: u64,
    pub mass_write_threshold: u64,
    pub identity_path: Option<String>,
    pub signing_key_path: Option<String>,
    pub enable_ebpf: bool,
    pub enable_auditd: bool,
    pub core_api_url: String,
}

impl AgentConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let max_processes = env::var("AGENT_MAX_PROCESSES")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<usize>()
            .map_err(|_| "AGENT_MAX_PROCESSES must be a valid integer")?;
        
        let max_connections = env::var("AGENT_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .map_err(|_| "AGENT_MAX_CONNECTIONS must be a valid integer")?;
        
        let max_queue_size = env::var("AGENT_MAX_QUEUE_SIZE")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<usize>()
            .map_err(|_| "AGENT_MAX_QUEUE_SIZE must be a valid integer")?;
        
        let rate_limit_tokens = env::var("AGENT_RATE_LIMIT_TOKENS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .map_err(|_| "AGENT_RATE_LIMIT_TOKENS must be a valid integer")?;
        
        let rate_limit_refill = env::var("AGENT_RATE_LIMIT_REFILL")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u64>()
            .map_err(|_| "AGENT_RATE_LIMIT_REFILL must be a valid integer")?;
        
        let mass_write_threshold = env::var("AGENT_MASS_WRITE_THRESHOLD")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .map_err(|_| "AGENT_MASS_WRITE_THRESHOLD must be a valid integer")?;
        
        let identity_path = env::var("AGENT_IDENTITY_PATH").ok();
        let signing_key_path = env::var("AGENT_SIGNING_KEY_PATH").ok();
        
        let enable_ebpf = env::var("ENABLE_EBPF")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let enable_auditd = env::var("ENABLE_AUDITD")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let core_api_url = env::var("CORE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        
        Ok(AgentConfig {
            max_processes,
            max_connections,
            max_queue_size,
            rate_limit_tokens,
            rate_limit_refill,
            mass_write_threshold,
            identity_path,
            signing_key_path,
            enable_ebpf,
            enable_auditd,
            core_api_url,
        })
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_processes == 0 {
            return Err("AGENT_MAX_PROCESSES must be greater than 0".to_string());
        }
        
        if self.max_connections == 0 {
            return Err("AGENT_MAX_CONNECTIONS must be greater than 0".to_string());
        }
        
        if self.max_queue_size == 0 {
            return Err("AGENT_MAX_QUEUE_SIZE must be greater than 0".to_string());
        }
        
        if !self.enable_ebpf && !self.enable_auditd {
            return Err("At least one of ENABLE_EBPF or ENABLE_AUDITD must be true".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_succeeds_with_defaults() {
        env::remove_var("AGENT_MAX_PROCESSES");
        let config = AgentConfig::from_env();
        assert!(config.is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AgentConfig::from_env().unwrap();
        config.max_processes = 0;
        assert!(config.validate().is_err());
    }
}
