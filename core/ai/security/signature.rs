// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/security/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: RSA-4096 signature verification for models and manifests

use std::path::Path;
use std::fs;
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use tracing::{error, debug};

pub struct SignatureVerifier {
    public_key_bytes: Vec<u8>,
}

impl SignatureVerifier {
    pub fn new(public_key_path: &Path) -> Result<Self, String> {
        let public_key_bytes = fs::read(public_key_path)
            .map_err(|e| format!("Failed to read public key from {:?}: {}", public_key_path, e))?;
        
        Ok(Self {
            public_key_bytes,
        })
    }
    
    /// Verify RSA-4096 signature
    pub fn verify(&self, data: &[u8], signature_b64: &str) -> Result<bool, String> {
        // Decode signature
        let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        
        // Compute hash of data
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        // Verify signature
        let public_key = UnparsedPublicKey::new(
            &RSA_PSS_2048_8192_SHA256,
            &self.public_key_bytes,
        );
        
        match public_key.verify(&hash, &signature_bytes) {
            Ok(_) => {
                debug!("Signature verification successful");
                Ok(true)
            }
            Err(e) => {
                error!("Signature verification failed: {:?}", e);
                Err(format!("Verification failed: {:?}", e))
            }
        }
    }
    
    /// Verify JSON signature
    pub fn verify_json(&self, json_bytes: &[u8], signature_b64: &str) -> Result<bool, String> {
        self.verify(json_bytes, signature_b64)
    }
}

