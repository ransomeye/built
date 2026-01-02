// Path and File Name : /home/ransomeye/rebuild/core/kernel/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Core kernel - fail-closed trust initialization

use std::path::Path;
use std::fs;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Trust material missing: {0}")]
    TrustMaterialMissing(String),
    #[error("Failed to initialize trust: {0}")]
    TrustInitFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Core kernel with fail-closed trust enforcement
pub struct Kernel {
    root_key_path: String,
    initialized: bool,
}

impl Kernel {
    /// Initialize kernel with fail-closed trust enforcement
    /// 
    /// FAIL-CLOSED: Returns error if trust material is missing
    pub fn new() -> Result<Self, KernelError> {
        // Require root public key from environment
        let root_key_path = std::env::var("RANSOMEYE_ROOT_KEY_PATH")
            .map_err(|_| KernelError::TrustMaterialMissing(
                "RANSOMEYE_ROOT_KEY_PATH environment variable not set".to_string()
            ))?;
        
        // Verify root key file exists
        if !Path::new(&root_key_path).exists() {
            return Err(KernelError::TrustMaterialMissing(
                format!("Root key file not found: {}", root_key_path)
            ));
        }
        
        // Verify root key file is not empty
        let key_size = fs::metadata(&root_key_path)
            .map_err(|e| KernelError::TrustInitFailed(
                format!("Failed to read root key metadata: {}", e)
            ))?
            .len();
        
        if key_size == 0 {
            return Err(KernelError::TrustMaterialMissing(
                format!("Root key file is empty: {}", root_key_path)
            ));
        }
        
        info!("Kernel initialized with root key: {} ({} bytes)", root_key_path, key_size);
        
        Ok(Self {
            root_key_path,
            initialized: true,
        })
    }
    
    /// Verify kernel is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get root key path
    pub fn root_key_path(&self) -> &str {
        &self.root_key_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kernel_fails_without_trust_material() {
        // Unset required environment variable
        std::env::remove_var("RANSOMEYE_ROOT_KEY_PATH");
        
        let result = Kernel::new();
        assert!(result.is_err());
        
        if let Err(KernelError::TrustMaterialMissing(_)) = result {
            // Expected error
        } else {
            panic!("Expected TrustMaterialMissing error");
        }
    }
}
