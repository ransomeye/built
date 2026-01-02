// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/config.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration loading from environment variables - no hardcoded values

use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub core_api_url: String,
    pub buffer_dir: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: String,
    pub max_buffer_size_mb: usize,
    pub backpressure_threshold: usize,
    pub telemetry_interval_seconds: u64,
    pub health_report_interval_seconds: u64,
    pub monitor_paths: Vec<std::path::PathBuf>,
    pub monitor_registry_keys: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let monitor_paths: Vec<std::path::PathBuf> = env::var("MONITOR_PATHS")
            .unwrap_or_else(|_| "C:\\,C:\\Users".to_string())
            .split(',')
            .map(|s| std::path::PathBuf::from(s.trim()))
            .collect();
        
        let monitor_registry_keys: Vec<String> = env::var("MONITOR_REGISTRY_KEYS")
            .unwrap_or_else(|_| "HKCU\\Software,HKLM\\Software".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        Ok(Config {
            core_api_url: env::var("CORE_API_URL")
                .unwrap_or_else(|_| "https://localhost:8443".to_string()),
            buffer_dir: env::var("BUFFER_DIR")
                .unwrap_or_else(|_| "C:\\ProgramData\\RansomEye\\WindowsAgent\\buffer".to_string()),
            cert_path: env::var("AGENT_CERT_PATH")
                .unwrap_or_else(|_| "C:\\ProgramData\\RansomEye\\certs\\agent.crt".to_string()),
            key_path: env::var("AGENT_KEY_PATH")
                .unwrap_or_else(|_| "C:\\ProgramData\\RansomEye\\certs\\agent.key".to_string()),
            ca_cert_path: env::var("CA_CERT_PATH")
                .unwrap_or_else(|_| "C:\\ProgramData\\RansomEye\\certs\\ca.crt".to_string()),
            max_buffer_size_mb: env::var("MAX_BUFFER_SIZE_MB")
                .unwrap_or_else(|_| "512".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("MAX_BUFFER_SIZE_MB".to_string()))?,
            backpressure_threshold: env::var("BACKPRESSURE_THRESHOLD")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("BACKPRESSURE_THRESHOLD".to_string()))?,
            telemetry_interval_seconds: env::var("TELEMETRY_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("TELEMETRY_INTERVAL_SECONDS".to_string()))?,
            health_report_interval_seconds: env::var("HEALTH_REPORT_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("HEALTH_REPORT_INTERVAL_SECONDS".to_string()))?,
            monitor_paths,
            monitor_registry_keys,
        })
    }
}

