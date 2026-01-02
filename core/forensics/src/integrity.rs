// Path and File Name : /home/ransomeye/rebuild/core/forensics/src/integrity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence integrity - content-addressed storage and tamper detection

use sha2::{Sha256, Digest};
use hex;
use serde::{Serialize, Deserialize};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use base64::{Engine as _, engine::general_purpose};
use rand::rngs::OsRng;
use rand::RngCore;

/// Evidence integrity checker
pub struct EvidenceIntegrity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl EvidenceIntegrity {
    /// Create new evidence integrity checker
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
    
    /// Compute content hash (SHA-256)
    pub fn compute_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Sign evidence
    pub fn sign(&self, data: &[u8]) -> String {
        let signature: Signature = self.signing_key.sign(data);
        general_purpose::STANDARD.encode(signature.to_bytes())
    }
    
    /// Verify evidence signature
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
    
    /// Verify evidence hash matches content
    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> Result<(), String> {
        let computed_hash = self.compute_hash(data);
        if computed_hash != expected_hash {
            return Err(format!("Hash mismatch: expected {}, got {}", expected_hash, computed_hash));
        }
        Ok(())
    }
}

impl Default for EvidenceIntegrity {
    fn default() -> Self {
        Self::new()
    }
}

/// Evidence item with integrity guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub content_hash: String,
    pub signature: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence_type: String,
    pub source: String,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
}

