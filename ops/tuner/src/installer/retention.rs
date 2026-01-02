// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/retention.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Retention configurator - configures data retention policies with validation

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::errors::OperationsError;

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub telemetry_retention_months: u32,
    pub forensic_retention_days: u32,
    pub disk_max_usage_percent: u8,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            telemetry_retention_months: 6,
            forensic_retention_days: 10,
            disk_max_usage_percent: 80,
        }
    }
}

/// Retention configurator - configures and validates retention policies
pub struct RetentionConfigurator {
    config_path: String,
}

impl RetentionConfigurator {
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }
    
    /// Configure retention with defaults or user input
    pub fn configure(&self, policy: Option<RetentionPolicy>) -> Result<RetentionPolicy, OperationsError> {
        let policy = policy.unwrap_or_else(|| RetentionPolicy::default());
        
        // Validate policy
        self.validate(&policy)?;
        
        // Write configuration
        self.write_config(&policy)?;
        
        debug!("Retention policy configured: {} months telemetry, {} days forensic, {}% disk threshold",
               policy.telemetry_retention_months,
               policy.forensic_retention_days,
               policy.disk_max_usage_percent);
        
        Ok(policy)
    }
    
    /// Validate retention policy
    pub fn validate(&self, policy: &RetentionPolicy) -> Result<(), OperationsError> {
        // Validate telemetry retention (0-84 months / 0-7 years)
        if policy.telemetry_retention_months > 84 {
            return Err(OperationsError::InvalidRetention(
                "Telemetry retention cannot exceed 84 months (7 years)".to_string()
            ));
        }
        
        // Validate forensic retention (0-3650 days / 0-10 years)
        if policy.forensic_retention_days > 3650 {
            return Err(OperationsError::InvalidRetention(
                "Forensic retention cannot exceed 3650 days (10 years)".to_string()
            ));
        }
        
        // Validate disk usage threshold (50-100%)
        if policy.disk_max_usage_percent < 50 || policy.disk_max_usage_percent > 100 {
            return Err(OperationsError::InvalidRetention(
                "Disk usage threshold must be between 50% and 100%".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Write retention configuration to file
    fn write_config(&self, policy: &RetentionPolicy) -> Result<(), OperationsError> {
        let config_dir = Path::new(&self.config_path).parent()
            .ok_or_else(|| OperationsError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid config path"
            )))?;
        
        fs::create_dir_all(config_dir)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let config_content = format!(
            "# Path and File Name : {}\n\
            # Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU\n\
            # Details of functionality of this file: Data retention policy configuration\n\n\
            TELEMETRY_RETENTION_MONTHS={}\n\
            FORENSIC_RETENTION_DAYS={}\n\
            DISK_MAX_USAGE_PERCENT={}\n\n\
            # Note: AI Training Artifacts have a mandatory minimum retention of 2 years\n\
            # AI artifacts cannot be deleted by disk pressure - only via explicit operator approval\n",
            self.config_path,
            policy.telemetry_retention_months,
            policy.forensic_retention_days,
            policy.disk_max_usage_percent
        );
        
        fs::write(&self.config_path, config_content)
            .map_err(|e| OperationsError::IoError(e))?;
        
        Ok(())
    }
    
    /// Load existing retention configuration
    pub fn load(&self) -> Result<RetentionPolicy, OperationsError> {
        if !Path::new(&self.config_path).exists() {
            return Ok(RetentionPolicy::default());
        }
        
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let mut policy = RetentionPolicy::default();
        
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "TELEMETRY_RETENTION_MONTHS" => {
                        policy.telemetry_retention_months = value.trim().parse()
                            .map_err(|_| OperationsError::InvalidRetention(
                                format!("Invalid telemetry retention: {}", value)
                            ))?;
                    }
                    "FORENSIC_RETENTION_DAYS" => {
                        policy.forensic_retention_days = value.trim().parse()
                            .map_err(|_| OperationsError::InvalidRetention(
                                format!("Invalid forensic retention: {}", value)
                            ))?;
                    }
                    "DISK_MAX_USAGE_PERCENT" => {
                        policy.disk_max_usage_percent = value.trim().parse()
                            .map_err(|_| OperationsError::InvalidRetention(
                                format!("Invalid disk usage threshold: {}", value)
                            ))?;
                    }
                    _ => {}
                }
            }
        }
        
        self.validate(&policy)?;
        Ok(policy)
    }
}

