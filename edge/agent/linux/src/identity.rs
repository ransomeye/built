// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/identity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component identity management - unique per-instance keypair generation and loading

use std::fs;
use std::path::Path;
use ring::rand::{SystemRandom, SecureRandom};
use ring::signature::{RsaKeyPair, RSA_PSS_SHA256};
use sha2::{Sha256, Digest};
use base64;
use thiserror::Error;
use crate::config::Config;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Failed to generate keypair: {0}")]
    KeyGenerationFailed(String),
    #[error("Failed to save identity: {0}")]
    SaveFailed(String),
    #[error("Failed to load identity: {0}")]
    LoadFailed(String),
}

pub struct Identity {
    producer_id: String,
    keypair_bytes: Vec<u8>,
}

impl Identity {
    pub fn load_or_create(config: &Config) -> Result<Self, IdentityError> {
        let identity_dir = Path::new(&config.buffer_dir).parent()
            .unwrap_or(Path::new("/var/lib/ransomeye/linux_agent"));
        
        let key_path = identity_dir.join("identity.key");
        let id_path = identity_dir.join("identity.id");
        
        // Try to load existing identity
        if key_path.exists() && id_path.exists() {
            return Self::load(&key_path, &id_path);
        }
        
        // Create new identity
        Self::create(&key_path, &id_path)
    }
    
    fn create(key_path: &Path, id_path: &Path) -> Result<Self, IdentityError> {
        // Create directory if needed
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| IdentityError::SaveFailed(format!("Failed to create directory: {}", e)))?;
        }
        
        // Generate RSA-4096 keypair
        let rng = SystemRandom::new();
        let keypair_bytes = RsaKeyPair::generate_pkcs8(&rng, 4096)
            .map_err(|e| IdentityError::KeyGenerationFailed(format!("{}", e)))?;
        
        // Generate producer ID from keypair hash
        let mut hasher = Sha256::new();
        hasher.update(&keypair_bytes);
        let hash = hasher.finalize();
        let producer_id = format!("linux_agent_{}", hex::encode(&hash[..16]));
        
        // Save identity
        fs::write(key_path, &keypair_bytes)
            .map_err(|e| IdentityError::SaveFailed(format!("Failed to write key: {}", e)))?;
        fs::set_permissions(key_path, fs::Permissions::from_mode(0o600))
            .map_err(|e| IdentityError::SaveFailed(format!("Failed to set permissions: {}", e)))?;
        
        fs::write(id_path, &producer_id)
            .map_err(|e| IdentityError::SaveFailed(format!("Failed to write ID: {}", e)))?;
        
        Ok(Identity {
            producer_id,
            keypair_bytes: keypair_bytes.as_ref().to_vec(),
        })
    }
    
    fn load(key_path: &Path, id_path: &Path) -> Result<Self, IdentityError> {
        let keypair_bytes = fs::read(key_path)
            .map_err(|e| IdentityError::LoadFailed(format!("Failed to read key: {}", e)))?;
        
        let producer_id = fs::read_to_string(id_path)
            .map_err(|e| IdentityError::LoadFailed(format!("Failed to read ID: {}", e)))?;
        
        Ok(Identity {
            producer_id,
            keypair_bytes,
        })
    }
    
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }
    
    pub fn keypair(&self) -> RsaKeyPair {
        RsaKeyPair::from_pkcs8(&self.keypair_bytes)
            .expect("Keypair should be valid")
    }
}

