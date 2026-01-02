// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/security/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic signature verification using ring RSA-4096

use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use tracing::{error, debug};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct SignatureVerifier {
    public_key_bytes: Arc<Vec<u8>>,
}

impl SignatureVerifier {
    pub fn new(public_key_der: &[u8]) -> Result<Self, String> {
        Ok(Self {
            public_key_bytes: Arc::new(public_key_der.to_vec()),
        })
    }
    
    /// Verify signature using ring RSA-4096
    pub fn verify(&self, data: &[u8], signature_b64: &str) -> Result<bool, String> {
        let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        
        let public_key = UnparsedPublicKey::new(
            &RSA_PSS_2048_8192_SHA256,
            &self.public_key_bytes,
        );
        
        match public_key.verify(data, &signature_bytes) {
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
    
    /// Verify signature of JSON data
    pub fn verify_json(&self, json_bytes: &[u8], signature_b64: &str) -> Result<bool, String> {
        // Compute hash of JSON
        let mut hasher = Sha256::new();
        hasher.update(json_bytes);
        let hash = hasher.finalize();
        
        // Verify signature against hash
        self.verify(&hash, signature_b64)
    }
    
    /// Verify directive signature
    pub fn verify_directive(&self, directive_json: &str, signature: &str) -> Result<bool, String> {
        let json_bytes = directive_json.as_bytes();
        self.verify_json(json_bytes, signature)
    }
}

