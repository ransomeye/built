// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/config/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Configuration validation

use std::env;
use tracing::{error, info};

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let policies_path = env::var("RANSOMEYE_POLICY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/policy/policies".to_string());

        if !std::path::Path::new(&policies_path).exists() {
            return Err(format!("Policies directory not found: {}", policies_path).into());
        }

        let trust_store_path = env::var("RANSOMEYE_TRUST_STORE_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/policy/trust".to_string());

        if !std::path::Path::new(&trust_store_path).exists() {
            error!("Trust store directory not found: {}", trust_store_path);
            return Err(format!("Trust store directory not found: {}", trust_store_path).into());
        }

        info!("Configuration validated successfully");
        Ok(())
    }

    pub fn get_policies_path(&self) -> String {
        env::var("RANSOMEYE_POLICY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/policy/policies".to_string())
    }

    pub fn get_trust_store_path(&self) -> String {
        env::var("RANSOMEYE_TRUST_STORE_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/policy/trust".to_string())
    }

    pub fn get_revocation_list_path(&self) -> String {
        env::var("RANSOMEYE_REVOCATION_LIST_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/policy/revocation.list".to_string())
    }

    pub fn get_audit_log_path(&self) -> String {
        env::var("RANSOMEYE_AUDIT_LOG_PATH")
            .unwrap_or_else(|_| "/var/log/ransomeye/policy_audit.log".to_string())
    }
}

