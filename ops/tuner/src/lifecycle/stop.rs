// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/lifecycle/stop.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Service stopper - stops services in reverse dependency order

use tracing::{info, debug};

use crate::errors::OperationsError;
use crate::lifecycle::status::ServiceStatusChecker;

/// Service stopper - stops services in reverse dependency order
pub struct ServiceStopper {
    status_checker: ServiceStatusChecker,
}

impl ServiceStopper {
    pub fn new(state_path: &str, keys_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            status_checker: ServiceStatusChecker::new(state_path, keys_dir),
        }
    }
    
    /// Stop all services in reverse dependency order
    pub fn stop_all(&self) -> Result<(), OperationsError> {
        info!("Stopping all RansomEye services");
        
        // Reverse dependency order
        let services = vec![
            "ransomeye-reporting",
            "ransomeye-intelligence",
            "ransomeye-enforcement",
            "ransomeye-policy",
            "ransomeye-correlation",
            "ransomeye-ingestion",
            "ransomeye-core",
        ];
        
        for service in services {
            self.stop_service(service)?;
        }
        
        info!("All services stopped");
        Ok(())
    }
    
    /// Stop a single service
    pub fn stop_service(&self, service_name: &str) -> Result<(), OperationsError> {
        debug!("Stopping service: {}", service_name);
        
        let output = std::process::Command::new("systemctl")
            .arg("stop")
            .arg(service_name)
            .output()
            .map_err(|e| OperationsError::SystemdError(format!("Failed to stop service: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Don't fail if service is already stopped
            if !error_msg.contains("not loaded") && !error_msg.contains("not found") {
                return Err(OperationsError::ServiceOperationFailed(
                    format!("Failed to stop {}: {}", service_name, error_msg)
                ));
            }
        }
        
        debug!("Service {} stopped", service_name);
        Ok(())
    }
}

