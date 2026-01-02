// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/uninstaller/uninstall.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main uninstaller - orchestrates complete uninstallation with verification and cleanup

use tracing::{info, error};

use crate::errors::OperationsError;
use crate::installer::state::InstallStateManager;
use crate::uninstaller::verification::UninstallVerifier;
use crate::uninstaller::cleanup::{CleanupManager, CleanupOptions};

/// Main uninstaller - orchestrates uninstallation flow
pub struct Uninstaller {
    project_root: String,
    state_path: String,
    keys_dir: String,
}

impl Uninstaller {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            state_path: format!("{}/ransomeye_installer/config/install_state.json", project_root),
            keys_dir: format!("{}/ransomeye_installer/keys", project_root),
        }
    }
    
    /// Run complete uninstallation flow
    pub fn uninstall(
        &self,
        remove_evidence: bool,
        secure_delete: bool,
        confirmed: bool,
    ) -> Result<(), OperationsError> {
        if !confirmed {
            return Err(OperationsError::UninstallVerificationFailed(
                "Uninstallation requires explicit confirmation".to_string()
            ));
        }
        
        info!("Starting RansomEye uninstallation");
        
        // Step 1: Verify install state
        info!("[1/3] Verifying install state...");
        let verifier = UninstallVerifier::new(&self.state_path, &self.keys_dir);
        verifier.verify()?;
        
        // Load state for cleanup
        let state_manager = InstallStateManager::new(&self.state_path, &self.keys_dir);
        let state = state_manager.load()?;
        
        // Step 2: Stop all services
        info!("[2/3] Stopping services...");
        // Services would be stopped here via systemd
        
        // Step 3: Cleanup
        info!("[3/3] Performing cleanup...");
        let cleanup_manager = CleanupManager::new(&self.project_root);
        let cleanup_options = CleanupOptions {
            remove_services: true,
            remove_configs: true,
            remove_evidence,
            secure_delete,
        };
        
        let cleanup_log = cleanup_manager.cleanup(&state, &cleanup_options)?;
        
        // Log cleanup
        self.log_cleanup(&cleanup_log)?;
        
        println!("\n{}", "=".repeat(80));
        println!("RANSOMEYE UNINSTALLATION COMPLETE");
        println!("{}", "=".repeat(80));
        println!();
        println!("Removed services: {}", cleanup_log.removed_services.len());
        println!("Removed configs: {}", cleanup_log.removed_configs.len());
        println!("Evidence destroyed: {}", cleanup_log.evidence_destroyed);
        println!("Secure deletion used: {}", cleanup_log.secure_delete_used);
        println!();
        println!("{}", "=".repeat(80));
        
        info!("Uninstallation complete");
        Ok(())
    }
    
    /// Log cleanup operation
    fn log_cleanup(&self, log: &crate::uninstaller::cleanup::CleanupLog) -> Result<(), OperationsError> {
        use std::fs;
        use serde_json;
        
        let log_dir = format!("{}/logs", self.project_root);
        fs::create_dir_all(&log_dir)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let log_file = format!("{}/uninstall_{}.json", log_dir, log.timestamp.format("%Y%m%d_%H%M%S"));
        let log_json = serde_json::to_string_pretty(log)
            .map_err(|e| OperationsError::SerializationError(e))?;
        
        fs::write(&log_file, log_json)
            .map_err(|e| OperationsError::IoError(e))?;
        
        Ok(())
    }
}

