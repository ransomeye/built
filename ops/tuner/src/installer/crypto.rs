// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/crypto.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic identity - generates and manages Ed25519 key pairs for signing install state

#![cfg(feature = "future-tuner")]

use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::rand::SystemRandom;
use std::fs;
use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::errors::OperationsError;

/// Cryptographic identity - Ed25519 key pair for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoIdentity {
    pub identity_id: String,
    pub public_key: String,  // Base64 encoded
    pub key_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Cryptographic identity manager
pub struct CryptoIdentityManager {
    keys_dir: PathBuf,
}

impl CryptoIdentityManager {
    pub fn new(keys_dir: impl AsRef<Path>) -> Self {
        Self {
            keys_dir: keys_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Generate new cryptographic identity
    pub fn generate(&self) -> Result<CryptoIdentity, OperationsError> {
        debug!("Generating new cryptographic identity");
        
        // Create keys directory
        fs::create_dir_all(&self.keys_dir)
            .map_err(|e| OperationsError::IoError(e))?;
        
        // Generate Ed25519 key pair
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| OperationsError::IdentityGenerationFailed(format!("Failed to generate key pair: {:?}", e)))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|e| OperationsError::IdentityGenerationFailed(format!("Failed to create key pair: {:?}", e)))?;
        
        // Get public key
        let public_key_bytes = key_pair.public_key().as_ref();
        let public_key_b64 = general_purpose::STANDARD.encode(public_key_bytes);
        
        // Save private key
        let identity_id = uuid::Uuid::new_v4().to_string();
        let key_filename = format!("identity_{}.pem", identity_id);
        let key_path = self.keys_dir.join(&key_filename);
        
        fs::write(&key_path, pkcs8_bytes.as_ref())
            .map_err(|e| OperationsError::IoError(e))?;
        
        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path)
                .map_err(|e| OperationsError::IoError(e))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms)
                .map_err(|e| OperationsError::IoError(e))?;
        }
        
        let identity = CryptoIdentity {
            identity_id: identity_id.clone(),
            public_key: public_key_b64,
            key_path: key_path.clone(),
            created_at: chrono::Utc::now(),
        };
        
        debug!("Generated cryptographic identity: {} (key: {:?})", identity_id, key_path);
        
        Ok(identity)
    }
    
    /// Load existing identity
    pub fn load(&self, key_path: impl AsRef<Path>) -> Result<CryptoIdentity, OperationsError> {
        let key_path = key_path.as_ref();
        
        if !key_path.exists() {
            return Err(OperationsError::IdentityGenerationFailed(
                format!("Key file not found: {:?}", key_path)
            ));
        }
        
        let key_data = fs::read(key_path)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(&key_data)
            .map_err(|e| OperationsError::IdentityGenerationFailed(format!("Failed to load key: {:?}", e)))?;
        
        let public_key_bytes = key_pair.public_key().as_ref();
        let public_key_b64 = general_purpose::STANDARD.encode(public_key_bytes);
        
        // Extract identity ID from filename
        let identity_id = key_path.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("identity_"))
            .unwrap_or("unknown")
            .to_string();
        
        // Get file creation time
        let metadata = fs::metadata(key_path)
            .map_err(|e| OperationsError::IoError(e))?;
        let created_at = metadata.modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
            })
            .unwrap_or_else(|| chrono::Utc::now());
        
        Ok(CryptoIdentity {
            identity_id,
            public_key: public_key_b64,
            key_path: key_path.to_path_buf(),
            created_at,
        })
    }
    
    /// Sign data with identity
    pub fn sign(&self, identity: &CryptoIdentity, data: &[u8]) -> Result<String, OperationsError> {
        let key_data = fs::read(&identity.key_path)
            .map_err(|e| OperationsError::IoError(e))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(&key_data)
            .map_err(|e| OperationsError::IdentityGenerationFailed(format!("Failed to load key: {:?}", e)))?;
        
        let signature = key_pair.sign(data);
        let signature_b64 = general_purpose::STANDARD.encode(signature.as_ref());
        
        Ok(signature_b64)
    }
    
    /// Verify signature
    pub fn verify(&self, public_key_b64: &str, data: &[u8], signature_b64: &str) -> Result<bool, OperationsError> {
        use ring::signature::{UnparsedPublicKey, ED25519};
        
        let public_key_bytes = general_purpose::STANDARD.decode(public_key_b64)
            .map_err(|e| OperationsError::SignatureVerificationFailed(format!("Invalid public key encoding: {}", e)))?;
        
        let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| OperationsError::SignatureVerificationFailed(format!("Invalid signature encoding: {}", e)))?;
        
        let unparsed_key = UnparsedPublicKey::new(&ED25519, &public_key_bytes);
        
        match unparsed_key.verify(data, &signature_bytes) {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Signature verification failed: {:?}", e);
                Ok(false)
            }
        }
    }
}

