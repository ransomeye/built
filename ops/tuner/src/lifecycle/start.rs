// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/lifecycle/start.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Service starter - starts services with dependency ordering and state validation

use tracing::{info, error, debug};

use crate::errors::OperationsError;
use crate::lifecycle::status::ServiceStatusChecker;

/// Service starter - starts services in dependency order
pub struct ServiceStarter {
    status_checker: ServiceStatusChecker,
}

impl ServiceStarter {
    pub fn new(state_path: &str, keys_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            status_checker: ServiceStatusChecker::new(state_path, keys_dir),
        }
    }
    
    /// Start all services in dependency order
    pub fn start_all(&self) -> Result<(), OperationsError> {
        // Validate install state
        self.status_checker.validate_state()?;
        
        info!("Starting all RansomEye services");
        
        // Service dependency order
        let services = vec![
            "ransomeye-core",
            "ransomeye-ingestion",
            "ransomeye-correlation",
            "ransomeye-policy",
            "ransomeye-enforcement",
            "ransomeye-intelligence",
            "ransomeye-reporting",
        ];
        
        for service in services {
            self.start_service(service)?;
        }
        
        info!("All services started");
        Ok(())
    }
    
    /// Start a single service
    pub fn start_service(&self, service_name: &str) -> Result<(), OperationsError> {
        // Validate install state
        self.status_checker.validate_state()?;
        
        debug!("Starting service: {}", service_name);
        
        let output = std::process::Command::new("systemctl")
            .arg("start")
            .arg(service_name)
            .output()
            .map_err(|e| OperationsError::SystemdError(format!("Failed to start service: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start service {}: {}", service_name, error_msg);
            return Err(OperationsError::ServiceOperationFailed(
                format!("Failed to start {}: {}", service_name, error_msg)
            ));
        }
        
        debug!("Service {} started successfully", service_name);
        Ok(())
    }
}

