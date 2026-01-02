// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/security/signature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 signature verification utilities

use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use hex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid signature format: {0}")]
    InvalidFormat(String),
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    #[error("Hash mismatch: {0}")]
    HashMismatch(String),
}

/// Verify an Ed25519 signature against data and public key
/// 
/// FAIL-CLOSED: Returns error on verification failure, never soft-fails
pub fn verify_signature(
    data: &[u8],
    signature: &str,
    public_key: &[u8],
) -> Result<bool, SignatureError> {
    // Decode base64 signature (Ed25519 signatures are 64 bytes)
    let signature_bytes = general_purpose::STANDARD.decode(signature)
        .map_err(|e| SignatureError::InvalidFormat(format!("Failed to decode signature: {}", e)))?;
    
    if signature_bytes.len() != 64 {
        return Err(SignatureError::InvalidFormat(
            format!("Invalid Ed25519 signature length: expected 64 bytes, got {}", signature_bytes.len())
        ));
    }
    
    // Parse Ed25519 public key (32 bytes)
    let key_bytes = if public_key.len() == 32 {
        // Raw 32-byte key
        public_key.to_vec()
    } else if public_key.starts_with(b"-----BEGIN") {
        // PEM format - extract base64 content
        let key_str = std::str::from_utf8(public_key)
            .map_err(|e| SignatureError::InvalidFormat(format!("Invalid PEM encoding: {}", e)))?;
        let lines: Vec<&str> = key_str.lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        let base64_content: String = lines.join("");
        let decoded = general_purpose::STANDARD.decode(base64_content.trim())
            .map_err(|e| SignatureError::InvalidFormat(format!("Failed to decode PEM public key: {}", e)))?;
        
        if decoded.len() != 32 {
            return Err(SignatureError::InvalidFormat(
                format!("Invalid Ed25519 public key length: expected 32 bytes, got {}", decoded.len())
            ));
        }
        decoded
    } else {
        // Try base64 decode
        let decoded = general_purpose::STANDARD.decode(public_key)
            .map_err(|e| SignatureError::InvalidFormat(format!("Failed to decode public key: {}", e)))?;
        
        if decoded.len() != 32 {
            return Err(SignatureError::InvalidFormat(
                format!("Invalid Ed25519 public key length: expected 32 bytes, got {}", decoded.len())
            ));
        }
        decoded
    };
    
    // Create VerifyingKey from 32-byte public key
    let verifying_key = VerifyingKey::from_bytes(&key_bytes.try_into()
        .map_err(|_| SignatureError::InvalidFormat("Failed to convert public key to array".to_string()))?)
        .map_err(|e| SignatureError::InvalidFormat(format!("Invalid Ed25519 public key: {:?}", e)))?;
    
    // Create Signature from 64-byte signature
    let sig = Signature::from_bytes(&signature_bytes.try_into()
        .map_err(|_| SignatureError::InvalidFormat("Failed to convert signature to array".to_string()))?)
        .map_err(|e| SignatureError::InvalidFormat(format!("Invalid Ed25519 signature: {:?}", e)))?;
    
    // Verify signature using Ed25519
    match verifying_key.verify(data, &sig) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            Err(SignatureError::VerificationFailed(
                format!("Ed25519 signature verification failed: {:?}", e)
            ))
        }
    }
}

/// Compute data hash for verification
pub fn compute_data_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Validate signature format
pub fn validate_signature_format(signature: &str) -> bool {
    general_purpose::STANDARD.decode(signature)
        .map(|bytes| bytes.len() == 64)
        .unwrap_or(false)
}
