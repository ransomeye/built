// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/lifecycle/status.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Service status - checks service status and validates install state

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::errors::OperationsError;
use crate::installer::state::InstallStateManager;

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

/// Service status checker
pub struct ServiceStatusChecker {
    state_manager: InstallStateManager,
}

impl ServiceStatusChecker {
    pub fn new(state_path: &str, keys_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            state_manager: InstallStateManager::new(state_path, keys_dir),
        }
    }
    
    /// Check service status via systemd
    pub fn check_service(&self, service_name: &str) -> Result<ServiceStatus, OperationsError> {
        // Verify install state is valid
        if !self.state_manager.is_valid() {
            return Err(OperationsError::InvalidInstallState(
                "Install state is invalid or missing".to_string()
            ));
        }
        
        // Check systemd service status
        let output = std::process::Command::new("systemctl")
            .arg("is-active")
            .arg(service_name)
            .output()
            .map_err(|e| OperationsError::SystemdError(format!("Failed to check service status: {}", e)))?;
        
        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        match status_str.as_str() {
            "active" => Ok(ServiceStatus::Running),
            "inactive" => Ok(ServiceStatus::Stopped),
            "failed" => Ok(ServiceStatus::Failed),
            _ => Ok(ServiceStatus::Unknown),
        }
    }
    
    /// Check all RansomEye services
    pub fn check_all_services(&self) -> Result<std::collections::HashMap<String, ServiceStatus>, OperationsError> {
        let services = vec![
            "ransomeye-core",
            "ransomeye-ingestion",
            "ransomeye-correlation",
            "ransomeye-policy",
            "ransomeye-enforcement",
            "ransomeye-intelligence",
            "ransomeye-reporting",
        ];
        
        let mut statuses = std::collections::HashMap::new();
        
        for service in services {
            match self.check_service(service) {
                Ok(status) => {
                    statuses.insert(service.to_string(), status);
                }
                Err(e) => {
                    debug!("Failed to check service {}: {}", service, e);
                    statuses.insert(service.to_string(), ServiceStatus::Unknown);
                }
            }
        }
        
        Ok(statuses)
    }
    
    /// Validate install state before operations
    pub fn validate_state(&self) -> Result<(), OperationsError> {
        if !self.state_manager.is_valid() {
            return Err(OperationsError::InvalidInstallState(
                "Install state is invalid or missing".to_string()
            ));
        }
        
        let state = self.state_manager.load()?;
        self.state_manager.verify(&state)?;
        
        Ok(())
    }
}

