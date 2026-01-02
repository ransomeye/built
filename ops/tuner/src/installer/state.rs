// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/state.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Install state - signed, immutable install state that must be verified before operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use hex;
use tracing::{debug, error, warn};

use crate::errors::OperationsError;
use crate::installer::crypto::CryptoIdentity;

/// Install state - signed state file that must be valid for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallState {
    pub install_id: String,
    pub installed_at: DateTime<Utc>,
    pub eula_accepted: bool,
    pub eula_accepted_at: Option<DateTime<Utc>>,
    pub retention_policy: crate::installer::retention::RetentionPolicy,
    pub identity: CryptoIdentity,
    pub engine_version: String,
    pub state_hash: String,
    pub signature: String,
}

/// Install state manager
pub struct InstallStateManager {
    state_path: String,
    crypto_manager: crate::installer::crypto::CryptoIdentityManager,
}

impl InstallStateManager {
    pub fn new(state_path: &str, keys_dir: impl AsRef<Path>) -> Self {
        Self {
            state_path: state_path.to_string(),
            crypto_manager: crate::installer::crypto::CryptoIdentityManager::new(keys_dir),
        }
    }
    
    /// Create and save install state
    pub fn create(
        &self,
        eula_accepted: bool,
        eula_accepted_at: Option<DateTime<Utc>>,
        retention_policy: crate::installer::retention::RetentionPolicy,
        identity: CryptoIdentity,
        engine_version: &str,
    ) -> Result<InstallState, OperationsError> {
        if !eula_accepted {
            return Err(OperationsError::EulaNotAccepted);
        }
        
        let install_id = uuid::Uuid::new_v4().to_string();
        let installed_at = Utc::now();
        
        // Create state (without hash and signature first)
        let mut state = InstallState {
            install_id: install_id.clone(),
            installed_at,
            eula_accepted,
            eula_accepted_at,
            retention_policy,
            identity: identity.clone(),
            engine_version: engine_version.to_string(),
            state_hash: String::new(),
            signature: String::new(),
        };
        
        // Compute state hash
        let state_json = serde_json::to_string(&state)
            .map_err(|e| OperationsError::SerializationError(e))?;
        let state_hash = Self::compute_hash(&state_json);
        state.state_hash = state_hash.clone();
        
        // Sign state
        let signature = self.crypto_manager.sign(&identity, state_hash.as_bytes())?;
        state.signature = signature;
        
        // Save state
        self.save(&state)?;
        
        debug!("Created install state: {} (hash: {})", install_id, state_hash);
        
        Ok(state)
    }
    
    /// Load and verify install state
    pub fn load(&self) -> Result<InstallState, OperationsError> {
        if !Path::new(&self.state_path).exists() {
            return Err(OperationsError::InvalidInstallState(
                "Install state file does not exist".to_string()
            ));
        }
        
        let state_json = fs::read_to_string(&self.state_path)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let state: InstallState = serde_json::from_str(&state_json)
            .map_err(|e| OperationsError::SerializationError(e))?;
        
        // Verify state
        self.verify(&state)?;
        
        Ok(state)
    }
    
    /// Verify install state integrity
    pub fn verify(&self, state: &InstallState) -> Result<(), OperationsError> {
        // Recompute hash (excluding signature)
        let mut state_for_hash = state.clone();
        state_for_hash.signature = String::new();
        state_for_hash.state_hash = String::new();
        
        let state_json = serde_json::to_string(&state_for_hash)
            .map_err(|e| OperationsError::SerializationError(e))?;
        let computed_hash = Self::compute_hash(&state_json);
        
        // Verify hash matches
        if computed_hash != state.state_hash {
            error!("Install state hash mismatch: expected {}, got {}", state.state_hash, computed_hash);
            return Err(OperationsError::InstallStateTampered(
                "State hash mismatch - state may have been tampered with".to_string()
            ));
        }
        
        // Verify signature
        let is_valid = self.crypto_manager.verify(
            &state.identity.public_key,
            state.state_hash.as_bytes(),
            &state.signature,
        )?;
        
        if !is_valid {
            error!("Install state signature verification failed");
            return Err(OperationsError::SignatureVerificationFailed(
                "State signature is invalid - state may have been tampered with".to_string()
            ));
        }
        
        // Verify EULA was accepted
        if !state.eula_accepted {
            return Err(OperationsError::EulaNotAccepted);
        }
        
        debug!("Install state verified successfully");
        Ok(())
    }
    
    /// Save install state to disk
    fn save(&self, state: &InstallState) -> Result<(), OperationsError> {
        let state_dir = Path::new(&self.state_path).parent()
            .ok_or_else(|| OperationsError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid state path"
            )))?;
        
        fs::create_dir_all(state_dir)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let state_json = serde_json::to_string_pretty(state)
            .map_err(|e| OperationsError::SerializationError(e))?;
        
        fs::write(&self.state_path, state_json)
            .map_err(|e| OperationsError::IoError(e))?;
        
        Ok(())
    }
    
    /// Compute SHA-256 hash
    fn compute_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }
    
    /// Check if install state exists and is valid
    pub fn is_valid(&self) -> bool {
        match self.load() {
            Ok(state) => {
                self.verify(&state).is_ok()
            }
            Err(_) => false
        }
    }
}

