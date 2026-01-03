// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Real cryptographic policy signature verification using RSA-4096

#![cfg(feature = "future-policy")]

use ring::signature::{self, UnparsedPublicKey};
use base64::{Engine as _, engine::general_purpose};
use std::path::Path;
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, debug, warn};
use once_cell::sync::Lazy;

static TRUST_STORE: Lazy<Arc<RwLock<TrustStore>>> = Lazy::new(|| {
    Arc::new(RwLock::new(TrustStore::new()))
});

pub struct TrustStore {
    public_keys: Vec<Vec<u8>>,
}

impl TrustStore {
    fn new() -> Self {
        Self {
            public_keys: Vec::new(),
        }
    }

    pub fn load_public_key(&mut self, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(key_path);
        
        // Try DER format first (preferred for ring)
        if path.extension().and_then(|s| s.to_str()) == Some("der") {
            let key_bytes = fs::read(key_path)
                .map_err(|e| format!("Failed to read DER public key from {}: {}", key_path, e))?;
            self.public_keys.push(key_bytes);
            debug!("Loaded DER public key from {}", key_path);
            return Ok(());
        }
        
        // Fall back to PEM format
        let key_data = fs::read_to_string(key_path)
            .map_err(|e| format!("Failed to read public key from {}: {}", key_path, e))?;
        
        // Parse PEM format: extract base64 content between headers
        let key_bytes = if key_data.contains("-----BEGIN") {
            // PEM format - extract base64 content
            let lines: Vec<&str> = key_data.lines()
                .filter(|line| !line.starts_with("-----"))
                .collect();
            let base64_content: String = lines.join("");
            general_purpose::STANDARD.decode(base64_content.trim())
                .map_err(|e| format!("Failed to decode PEM public key: {}", e))?
        } else {
            // Assume raw base64
            general_purpose::STANDARD.decode(key_data.trim())
                .map_err(|e| format!("Failed to decode public key: {}", e))?
        };
        
        self.public_keys.push(key_bytes);
        debug!("Loaded PEM public key from {}", key_path);
        Ok(())
    }

    pub fn get_public_keys(&self) -> &[Vec<u8>] {
        &self.public_keys
    }
}

pub struct PolicySignatureVerifier;

impl PolicySignatureVerifier {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }

    pub fn load_trust_store(trust_store_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let trust_dir = Path::new(trust_store_path);
        if !trust_dir.exists() {
            return Err(format!("Trust store directory not found: {}", trust_store_path).into());
        }

        let mut store = TRUST_STORE.write();
        for entry in fs::read_dir(trust_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Load DER format first (preferred), then PEM
            if path.extension().and_then(|s| s.to_str()) == Some("der") ||
               path.extension().and_then(|s| s.to_str()) == Some("pem") ||
               path.extension().and_then(|s| s.to_str()) == Some("pub") {
                if let Err(e) = store.load_public_key(path.to_str().unwrap()) {
                    warn!("Failed to load public key from {}: {}", path.display(), e);
                }
            }
        }

        if store.public_keys.is_empty() {
            warn!("No public keys found in trust store - signature verification will fail");
        } else {
            debug!("Loaded {} public keys from trust store", store.public_keys.len());
        }
        Ok(())
    }

    pub fn verify(&self, content: &str, signature: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Ring's RSA_PSS_2048_8192_SHA256 expects the message (not the hash)
        // It will compute SHA-256 internally and verify the signature
        // Content must be the EXACT bytes that were signed (after removing signature fields and serializing)
        let signature_bytes = general_purpose::STANDARD.decode(signature)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;

        let store = TRUST_STORE.read();
        let public_keys = store.get_public_keys();

        if public_keys.is_empty() {
            error!("No public keys loaded in trust store");
            return Ok(false);
        }

        // Get content as raw bytes for verification
        let content_bytes = content.as_bytes();

        for (idx, public_key_bytes) in public_keys.iter().enumerate() {
            debug!("Trying public key {} ({} bytes)", idx, public_key_bytes.len());
            // Use RSA_PSS_2048_8192_SHA256 for verification (matches signing algorithm RSA_PSS_SHA256)
            let public_key = UnparsedPublicKey::new(&signature::RSA_PSS_2048_8192_SHA256, public_key_bytes);
            
            // Verify using EXACT bytes that were signed (no transformation, no hashing)
            // ring will compute SHA-256 internally as part of RSA_PSS_2048_8192_SHA256
            match public_key.verify(content_bytes, &signature_bytes) {
                Ok(_) => {
                    debug!("Policy signature verified successfully with key {}", idx);
                    return Ok(true);
                }
                Err(e) => {
                    error!("Signature verification failed with key {}: {:?}", idx, e);
                    debug!("Content length: {} bytes, Signature length: {} bytes", content_bytes.len(), signature_bytes.len());
                    continue;
                }
            }
        }

        error!("Policy signature verification failed: no matching public key");
        Ok(false)
    }

    pub fn verify_with_key(&self, content: &str, signature: &str, public_key_bytes: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        let signature_bytes = general_purpose::STANDARD.decode(signature)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;

        // Use RSA_PSS_2048_8192_SHA256 for verification (matches signing algorithm RSA_PSS_SHA256)
        let public_key = UnparsedPublicKey::new(&signature::RSA_PSS_2048_8192_SHA256, public_key_bytes);
        
        // Verify using EXACT bytes that were signed (no transformation, no hashing)
        // ring will compute SHA-256 internally as part of RSA_PSS_2048_8192_SHA256
        let content_bytes = content.as_bytes();
        match public_key.verify(content_bytes, &signature_bytes) {
            Ok(_) => {
                debug!("Policy signature verified successfully");
                Ok(true)
            }
            Err(e) => {
                error!("Policy signature verification failed: {:?}", e);
                Ok(false)
            }
        }
    }
}

