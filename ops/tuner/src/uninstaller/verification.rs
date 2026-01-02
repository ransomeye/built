// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/uninstaller/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Uninstall verification - verifies install state before uninstallation

use crate::installer::state::InstallStateManager;
use crate::errors::OperationsError;
use tracing::{debug, error};

/// Uninstall verifier - verifies install state before uninstallation
pub struct UninstallVerifier {
    state_manager: InstallStateManager,
}

impl UninstallVerifier {
    pub fn new(state_path: &str, keys_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            state_manager: InstallStateManager::new(state_path, keys_dir),
        }
    }
    
    /// Verify install state is valid before uninstallation
    pub fn verify(&self) -> Result<(), OperationsError> {
        debug!("Verifying install state before uninstallation");
        
        let state = self.state_manager.load()?;
        self.state_manager.verify(&state)?;
        
        debug!("Install state verified successfully");
        Ok(())
    }
    
    /// Check if installation exists
    pub fn installation_exists(&self) -> bool {
        self.state_manager.is_valid()
    }
}

