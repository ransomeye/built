// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/security.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic signature verification for playbooks (RSA-4096/Ed25519)

use std::fs;
use ring::signature::{self, UnparsedPublicKey};
use sha2::{Sha256, Digest};
use base64;
use crate::errors::PlaybookError;

pub struct PlaybookSignatureVerifier {
    public_key_bytes: Vec<u8>,
    algorithm: SignatureAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum SignatureAlgorithm {
    Rsa4096,
    Ed25519,
}

impl PlaybookSignatureVerifier {
    /// Create a new verifier from public key file path
    pub fn new(public_key_path: &str) -> Result<Self, PlaybookError> {
        let public_key_bytes = fs::read(public_key_path)
            .map_err(|e| PlaybookError::ConfigurationError(
                format!("Failed to read public key from {}: {}", public_key_path, e)
            ))?;
        
        // Detect algorithm from key format
        let algorithm = if public_key_bytes.len() > 512 {
            SignatureAlgorithm::Rsa4096
        } else {
            SignatureAlgorithm::Ed25519
        };
        
        Ok(Self {
            public_key_bytes,
            algorithm,
        })
    }
    
    /// Create a verifier from public key bytes
    pub fn from_bytes(public_key_bytes: Vec<u8>, algorithm: SignatureAlgorithm) -> Self {
        Self {
            public_key_bytes,
            algorithm,
        }
    }
    
    /// Verify playbook signature
    pub fn verify(&self, content: &[u8], signature: &str, signature_hash: &str) -> Result<bool, PlaybookError> {
        // First verify content hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let computed_hash = hasher.finalize();
        let computed_hash_hex = hex::encode(computed_hash);
        
        if computed_hash_hex != signature_hash {
            return Err(PlaybookError::InvalidSignature(
                format!("Content hash mismatch: expected {}, got {}", signature_hash, computed_hash_hex)
            ));
        }
        
        // Decode signature
        let signature_bytes = base64::decode(signature)
            .map_err(|e| PlaybookError::InvalidSignature(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        // Verify cryptographic signature
        let verification_result = match self.algorithm {
            SignatureAlgorithm::Rsa4096 => {
                let public_key = UnparsedPublicKey::new(
                    &signature::RSA_PSS_2048_8192_SHA256,
                    &self.public_key_bytes
                );
                public_key.verify(content, &signature_bytes).is_ok()
            }
            SignatureAlgorithm::Ed25519 => {
                let public_key = UnparsedPublicKey::new(
                    &signature::ED25519,
                    &self.public_key_bytes
                );
                public_key.verify(content, &signature_bytes).is_ok()
            }
        };
        
        if !verification_result {
            return Err(PlaybookError::InvalidSignature(
                "Cryptographic signature verification failed".to_string()
            ));
        }
        
        Ok(true)
    }
    
    /// Verify playbook from YAML content
    pub fn verify_playbook_yaml(&self, yaml_content: &str, playbook_signature: &str, playbook_signature_hash: &str) -> Result<bool, PlaybookError> {
        // Compute content hash (without signature fields)
        let content_for_hash = self.extract_content_for_hash(yaml_content)?;
        let content_bytes = content_for_hash.as_bytes();
        
        self.verify(content_bytes, playbook_signature, playbook_signature_hash)
    }
    
    /// Extract content for hashing (remove signature fields)
    fn extract_content_for_hash(&self, yaml_content: &str) -> Result<String, PlaybookError> {
        use serde_yaml::Value;
        
        let mut doc: Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| PlaybookError::SchemaValidationFailed(
                format!("Failed to parse YAML: {}", e)
            ))?;
        
        // Remove signature fields
        if let Value::Mapping(ref mut map) = doc {
            map.remove(&Value::String("signature".to_string()));
            map.remove(&Value::String("signature_hash".to_string()));
        }
        
        serde_yaml::to_string(&doc)
            .map_err(|e| PlaybookError::InternalError(
                format!("Failed to serialize YAML: {}", e)
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signature_verifier_creation() {
        // This test would require actual key files
        // For now, just test that the structure is correct
        let verifier = PlaybookSignatureVerifier::from_bytes(
            vec![0u8; 32],
            SignatureAlgorithm::Ed25519
        );
        assert_eq!(verifier.algorithm, SignatureAlgorithm::Ed25519);
    }
}

