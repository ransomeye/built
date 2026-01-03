// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/lifecycle/restart.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Service restarter - restarts services with state validation

#![cfg(feature = "future-tuner")]

use tracing::info;

use crate::errors::OperationsError;
use crate::lifecycle::start::ServiceStarter;
use crate::lifecycle::stop::ServiceStopper;

/// Service restarter - restarts services
pub struct ServiceRestarter {
    starter: ServiceStarter,
    stopper: ServiceStopper,
}

impl ServiceRestarter {
    pub fn new(state_path: &str, keys_dir: impl AsRef<std::path::Path>) -> Self {
        let keys_dir_ref = keys_dir.as_ref();
        Self {
            starter: ServiceStarter::new(state_path, keys_dir_ref),
            stopper: ServiceStopper::new(state_path, keys_dir_ref),
        }
    }
    
    /// Restart all services
    pub fn restart_all(&self) -> Result<(), OperationsError> {
        info!("Restarting all RansomEye services");
        
        self.stopper.stop_all()?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        self.starter.start_all()?;
        
        info!("All services restarted");
        Ok(())
    }
    
    /// Restart a single service
    pub fn restart_service(&self, service_name: &str) -> Result<(), OperationsError> {
        info!("Restarting service: {}", service_name);
        
        self.stopper.stop_service(service_name)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.starter.start_service(service_name)?;
        
        info!("Service {} restarted", service_name);
        Ok(())
    }
}

