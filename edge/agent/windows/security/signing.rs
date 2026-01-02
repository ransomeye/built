// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/security/signing.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 event signing with replay-safe sequence numbers

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{error, debug, info};

#[path = "../agent/src/errors.rs"]
mod errors;
use errors::AgentError;

/// Event signer using Ed25519
pub struct EventSigner {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    sequence: Arc<AtomicU64>,
}

impl EventSigner {
    /// Create new event signer
    pub fn new() -> Result<Self, AgentError> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        info!("Event signer created with Ed25519 key");
        
        Ok(Self {
            signing_key,
            verifying_key,
            sequence: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Load signer from key file
    pub fn from_key_file(key_path: &std::path::Path) -> Result<Self, AgentError> {
        let key_bytes = std::fs::read(key_path)
            .map_err(|e| AgentError::SigningFailed(
                format!("Failed to read key file: {}", e)
            ))?;
        
        if key_bytes.len() != 32 {
            return Err(AgentError::SigningFailed(
                "Invalid key size (expected 32 bytes)".to_string()
            ));
        }
        
        let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
        let verifying_key = signing_key.verifying_key();
        
        info!("Event signer loaded from key file");
        
        Ok(Self {
            signing_key,
            verifying_key,
            sequence: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Sign event data
    /// 
    /// Includes replay-safe sequence number.
    pub fn sign(&self, data: &[u8]) -> Result<String, AgentError> {
        let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
        
        let mut message = Vec::with_capacity(8 + data.len());
        message.extend_from_slice(&seq.to_be_bytes());
        message.extend_from_slice(data);
        
        let signature: Signature = self.signing_key.sign(&message);
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
        
        debug!("Event signed: sequence={}, signature_len={}", seq, signature_b64.len());
        Ok(signature_b64)
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8], signature_b64: &str, sequence: u64) -> Result<bool, AgentError> {
        let signature_bytes = base64::engine::general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| AgentError::SigningFailed(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        if signature_bytes.len() != 64 {
            return Err(AgentError::SigningFailed(
                "Invalid signature size (expected 64 bytes)".to_string()
            ));
        }
        
        let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());
        
        let mut message = Vec::with_capacity(8 + data.len());
        message.extend_from_slice(&sequence.to_be_bytes());
        message.extend_from_slice(data);
        
        match self.verifying_key.verify(&message, &signature) {
            Ok(_) => {
                debug!("Signature verified: sequence={}", sequence);
                Ok(true)
            }
            Err(e) => {
                error!("Signature verification failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Get verifying key (public key)
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }
    
    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence.load(Ordering::Acquire)
    }
}

