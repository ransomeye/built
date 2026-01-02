// Path and File Name : /home/ransomeye/rebuild/core/ai/security/ed25519_signer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 signature signing and verification for models (mandatory per Phase 6)

use std::path::Path;
use std::fs;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use tracing::{error, debug, info};
use rand::rngs::OsRng;

pub struct Ed25519ModelSigner {
    signing_key: SigningKey,
}

pub struct Ed25519ModelVerifier {
    verifying_key: VerifyingKey,
}

impl Ed25519ModelSigner {
    /// Create new signer from key file
    pub fn from_key_file(key_path: &Path) -> Result<Self, String> {
        let key_bytes = fs::read(key_path)
            .map_err(|e| format!("Failed to read signing key from {:?}: {}", key_path, e))?;
        
        if key_bytes.len() != 32 {
            return Err(format!("Invalid Ed25519 key length: expected 32 bytes, got {}", key_bytes.len()));
        }
        
        let signing_key = SigningKey::from_bytes(
            key_bytes.try_into().map_err(|_| "Invalid key format")?
        );
        
        info!("Ed25519 model signer initialized from {:?}", key_path);
        Ok(Self { signing_key })
    }
    
    /// Generate new signing key pair
    pub fn generate() -> (Self, Vec<u8>) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key_bytes = signing_key.verifying_key().to_bytes().to_vec();
        
        (Self { signing_key }, verifying_key_bytes)
    }
    
    /// Sign model data
    pub fn sign_model(&self, model_data: &[u8]) -> Result<String, String> {
        // Compute hash of model data
        let mut hasher = Sha256::new();
        hasher.update(model_data);
        let hash = hasher.finalize();
        
        // Sign hash
        let signature: Signature = self.signing_key.sign(&hash);
        let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
        
        debug!("Model signed with Ed25519 (hash: {})", hex::encode(hash));
        Ok(signature_b64)
    }
    
    /// Sign manifest JSON
    pub fn sign_manifest(&self, manifest_json: &[u8]) -> Result<String, String> {
        // Compute hash of manifest
        let mut hasher = Sha256::new();
        hasher.update(manifest_json);
        let hash = hasher.finalize();
        
        // Sign hash
        let signature: Signature = self.signing_key.sign(&hash);
        let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
        
        debug!("Manifest signed with Ed25519");
        Ok(signature_b64)
    }
}

impl Ed25519ModelVerifier {
    /// Create new verifier from public key file
    pub fn from_public_key_file(key_path: &Path) -> Result<Self, String> {
        let key_bytes = fs::read(key_path)
            .map_err(|e| format!("Failed to read public key from {:?}: {}", key_path, e))?;
        
        // Handle base64-encoded keys
        let decoded = if key_bytes.len() > 32 {
            // Try base64 decode
            general_purpose::STANDARD.decode(&key_bytes)
                .map_err(|e| format!("Failed to decode public key: {}", e))?
        } else {
            key_bytes
        };
        
        if decoded.len() != 32 {
            return Err(format!("Invalid Ed25519 public key length: expected 32 bytes, got {}", decoded.len()));
        }
        
        let verifying_key = VerifyingKey::from_bytes(
            decoded.try_into().map_err(|_| "Invalid public key format")?
        ).map_err(|e| format!("Invalid Ed25519 public key: {:?}", e))?;
        
        info!("Ed25519 model verifier initialized from {:?}", key_path);
        Ok(Self { verifying_key })
    }
    
    /// Verify model signature
    pub fn verify_model(&self, model_data: &[u8], signature_b64: &str) -> Result<bool, String> {
        // Compute hash of model data
        let mut hasher = Sha256::new();
        hasher.update(model_data);
        let hash = hasher.finalize();
        
        // Decode signature
        let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        
        if signature_bytes.len() != 64 {
            return Err(format!("Invalid Ed25519 signature length: expected 64 bytes, got {}", signature_bytes.len()));
        }
        
        let signature = Signature::from_bytes(
            signature_bytes.try_into().map_err(|_| "Invalid signature format")?
        );
        
        // Verify signature
        match self.verifying_key.verify(&hash, &signature) {
            Ok(_) => {
                debug!("Ed25519 model signature verified successfully");
                Ok(true)
            }
            Err(e) => {
                error!("Ed25519 model signature verification failed: {:?}", e);
                Err(format!("Verification failed: {:?}", e))
            }
        }
    }
    
    /// Verify manifest signature
    pub fn verify_manifest(&self, manifest_json: &[u8], signature_b64: &str) -> Result<bool, String> {
        // Compute hash of manifest
        let mut hasher = Sha256::new();
        hasher.update(manifest_json);
        let hash = hasher.finalize();
        
        // Decode signature
        let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        
        if signature_bytes.len() != 64 {
            return Err(format!("Invalid Ed25519 signature length: expected 64 bytes, got {}", signature_bytes.len()));
        }
        
        let signature = Signature::from_bytes(
            signature_bytes.try_into().map_err(|_| "Invalid signature format")?
        );
        
        // Verify signature
        match self.verifying_key.verify(&hash, &signature) {
            Ok(_) => {
                debug!("Ed25519 manifest signature verified successfully");
                Ok(true)
            }
            Err(e) => {
                error!("Ed25519 manifest signature verification failed: {:?}", e);
                Err(format!("Verification failed: {:?}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ed25519_sign_verify() {
        let (signer, public_key_bytes) = Ed25519ModelSigner::generate();
        
        // Create verifier from public key
        let verifier = Ed25519ModelVerifier::from_public_key_bytes(&public_key_bytes).unwrap();
        
        // Sign and verify model data
        let model_data = b"test model data";
        let signature = signer.sign_model(model_data).unwrap();
        
        assert!(verifier.verify_model(model_data, &signature).unwrap());
    }
}

impl Ed25519ModelVerifier {
    /// Create verifier from public key bytes
    pub fn from_public_key_bytes(key_bytes: &[u8]) -> Result<Self, String> {
        if key_bytes.len() != 32 {
            return Err(format!("Invalid Ed25519 public key length: expected 32 bytes, got {}", key_bytes.len()));
        }
        
        let verifying_key = VerifyingKey::from_bytes(
            key_bytes.try_into().map_err(|_| "Invalid public key format")?
        ).map_err(|e| format!("Invalid Ed25519 public key: {:?}", e))?;
        
        Ok(Self { verifying_key })
    }
}

