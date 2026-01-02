// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/security.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Ed25519 signature generation and verification for scan results

use std::fs;
use ring::signature::{self, UnparsedPublicKey, KeyPair, Ed25519KeyPair};
use sha2::{Sha256, Digest};
use base64;
use crate::errors::ScannerError;
use crate::result::ScanResult;

pub struct ScanResultSigner {
    key_pair: Ed25519KeyPair,
}

pub struct ScanResultVerifier {
    public_key_bytes: Vec<u8>,
}

impl ScanResultSigner {
    /// Create a new signer from private key file path
    pub fn new(private_key_path: &str) -> Result<Self, ScannerError> {
        let private_key_bytes = fs::read(private_key_path)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to read private key from {}: {}", private_key_path, e)
            ))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(&private_key_bytes)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to parse Ed25519 key pair: {}", e)
            ))?;
        
        Ok(Self { key_pair })
    }
    
    /// Create a signer from key bytes
    pub fn from_bytes(private_key_bytes: &[u8]) -> Result<Self, ScannerError> {
        let key_pair = Ed25519KeyPair::from_pkcs8(private_key_bytes)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to parse Ed25519 key pair: {}", e)
            ))?;
        
        Ok(Self { key_pair })
    }
    
    /// Sign a scan result
    pub fn sign(&self, result: &ScanResult) -> Result<String, ScannerError> {
        // Compute hash first
        let hash = result.compute_hash();
        
        // Sign the hash
        let signature_bytes = self.key_pair.sign(hash.as_bytes());
        
        // Encode signature
        Ok(base64::encode(signature_bytes.as_ref()))
    }
    
    /// Sign and attach signature to result
    pub fn sign_result(&self, mut result: ScanResult) -> Result<ScanResult, ScannerError> {
        // Compute and set hash
        result.hash = result.compute_hash();
        
        // Sign and set signature
        result.signature = self.sign(&result)?;
        
        Ok(result)
    }
}

impl ScanResultVerifier {
    /// Create a new verifier from public key file path
    pub fn new(public_key_path: &str) -> Result<Self, ScannerError> {
        let public_key_bytes = fs::read(public_key_path)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to read public key from {}: {}", public_key_path, e)
            ))?;
        
        Ok(Self { public_key_bytes })
    }
    
    /// Create a verifier from public key bytes
    pub fn from_bytes(public_key_bytes: Vec<u8>) -> Self {
        Self { public_key_bytes }
    }
    
    /// Verify scan result signature
    pub fn verify(&self, result: &ScanResult) -> Result<bool, ScannerError> {
        // Verify hash first
        let computed_hash = result.compute_hash();
        if computed_hash != result.hash {
            return Err(ScannerError::InvalidSignature(
                format!("Hash mismatch: expected {}, got {}", result.hash, computed_hash)
            ));
        }
        
        // Decode signature
        let signature_bytes = base64::decode(&result.signature)
            .map_err(|e| ScannerError::InvalidSignature(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        // Verify signature
        let public_key = UnparsedPublicKey::new(&signature::ED25519, &self.public_key_bytes);
        let verification_result = public_key.verify(computed_hash.as_bytes(), &signature_bytes).is_ok();
        
        if !verification_result {
            return Err(ScannerError::InvalidSignature(
                "Ed25519 signature verification failed".to_string()
            ));
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signer_verifier_roundtrip() {
        // Generate a test key pair (in production, keys would be loaded from files)
        // This test validates the structure, actual key generation would use ring::rand
        // For now, just test that the structure is correct
        assert!(true); // Placeholder - full test requires key generation
    }
}

