// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/security/signing.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 event signing with replay-safe sequence numbers

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier, SecretKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{error, debug, info};
use base64::Engine;

use crate::errors::ProbeError;

/// Event signer using Ed25519
pub struct EventSigner {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    sequence: Arc<AtomicU64>,
}

impl EventSigner {
    /// Create new event signer
    pub fn new() -> Result<Self, ProbeError> {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let secret_key = SecretKey::from(secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key);
        let verifying_key = signing_key.verifying_key();
        
        info!("Event signer created with Ed25519 key");
        
        Ok(Self {
            signing_key,
            verifying_key,
            sequence: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Load signer from key file
    pub fn from_key_file(key_path: &std::path::Path) -> Result<Self, ProbeError> {
        let key_bytes = std::fs::read(key_path)
            .map_err(|e| ProbeError::SigningFailed(
                format!("Failed to read key file: {}", e)
            ))?;
        
        if key_bytes.len() != 32 {
            return Err(ProbeError::SigningFailed(
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
    pub fn sign(&self, data: &[u8]) -> Result<String, ProbeError> {
        // Get next sequence number (replay-safe)
        let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
        
        // Create message: sequence || data
        let mut message = Vec::with_capacity(8 + data.len());
        message.extend_from_slice(&seq.to_be_bytes());
        message.extend_from_slice(data);
        
        // Sign message
        let signature: Signature = self.signing_key.sign(&message);
        
        // Encode signature as base64
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
        
        debug!("Event signed: sequence={}, signature_len={}", seq, signature_b64.len());
        Ok(signature_b64)
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8], signature_b64: &str, sequence: u64) -> Result<bool, ProbeError> {
        // Decode signature
        let signature_bytes = base64::engine::general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| ProbeError::SigningFailed(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        if signature_bytes.len() != 64 {
            return Err(ProbeError::SigningFailed(
                "Invalid signature size (expected 64 bytes)".to_string()
            ));
        }
        
        let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());
        
        // Reconstruct message: sequence || data
        let mut message = Vec::with_capacity(8 + data.len());
        message.extend_from_slice(&sequence.to_be_bytes());
        message.extend_from_slice(data);
        
        // Verify signature
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEvent {
    pub data: Vec<u8>,
    pub signature: String,
    pub sequence: u64,
}

