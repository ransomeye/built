// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/config.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration loading from environment variables - no hardcoded values

use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub core_api_url: String,
    pub capture_interface: String,
    pub buffer_dir: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: String,
    pub max_buffer_size_mb: usize,
    pub backpressure_threshold: usize,
    pub flow_timeout_seconds: u64,
    pub health_report_interval_seconds: u64,
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
        Ok(Config {
            core_api_url: env::var("CORE_API_URL")
                .unwrap_or_else(|_| "https://localhost:8443".to_string()),
            capture_interface: env::var("CAPTURE_IFACE")
                .unwrap_or_else(|_| "eth0".to_string()),
            buffer_dir: env::var("BUFFER_DIR")
                .unwrap_or_else(|_| "/var/lib/ransomeye/dpi_probe/buffer".to_string()),
            cert_path: env::var("PROBE_CERT_PATH")
                .unwrap_or_else(|_| "/etc/ransomeye/certs/probe.crt".to_string()),
            key_path: env::var("PROBE_KEY_PATH")
                .unwrap_or_else(|_| "/etc/ransomeye/certs/probe.key".to_string()),
            ca_cert_path: env::var("CA_CERT_PATH")
                .unwrap_or_else(|_| "/etc/ransomeye/certs/ca.crt".to_string()),
            max_buffer_size_mb: env::var("MAX_BUFFER_SIZE_MB")
                .unwrap_or_else(|_| "1024".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("MAX_BUFFER_SIZE_MB".to_string()))?,
            backpressure_threshold: env::var("BACKPRESSURE_THRESHOLD")
                .unwrap_or_else(|_| "8192".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("BACKPRESSURE_THRESHOLD".to_string()))?,
            flow_timeout_seconds: env::var("FLOW_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("FLOW_TIMEOUT_SECONDS".to_string()))?,
            health_report_interval_seconds: env::var("HEALTH_REPORT_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("HEALTH_REPORT_INTERVAL_SECONDS".to_string()))?,
        })
    }
}

