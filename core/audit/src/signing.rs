// Path and File Name : /home/ransomeye/rebuild/core/audit/src/signing.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit signing - Ed25519 signature generation and verification

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use hex;
use base64::{Engine as _, engine::general_purpose};

/// Audit signer using Ed25519
pub struct AuditSigner {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl AuditSigner {
    /// Create new audit signer (generates new keypair)
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let mut key_bytes = [0u8; 32];
        csprng.fill_bytes(&mut key_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// Create from existing keypair
    pub fn from_keypair(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// Sign audit record data
    pub fn sign(&self, data: &[u8]) -> String {
        let signature: Signature = self.signing_key.sign(data);
        general_purpose::STANDARD.encode(signature.to_bytes())
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &str) -> Result<(), String> {
        let signature_bytes = general_purpose::STANDARD.decode(signature)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        
        if signature_bytes.len() != 64 {
            return Err(format!("Invalid signature length: expected 64, got {}", signature_bytes.len()));
        }
        
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&signature_bytes[..64]);
        let signature = Signature::from_bytes(&sig_array);
        
        self.verifying_key.verify(data, &signature)
            .map_err(|e| format!("Signature verification failed: {}", e))?;
        
        Ok(())
    }
    
    /// Get verifying key (public key) as hex
    pub fn get_verifying_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }
}

impl Default for AuditSigner {
    fn default() -> Self {
        Self::new()
    }
}

