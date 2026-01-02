// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/verifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographically verifies guardrails.yaml signature

use ring::signature::{UnparsedPublicKey, ED25519};
use sha2::{Sha256, Digest};
use hex;
use base64::{Engine as _, engine::general_purpose};
use crate::errors::{GuardrailError, GuardrailResult};
use crate::spec::GuardrailSpec;

pub struct GuardrailVerifier {
    public_key_bytes: Vec<u8>,
}

impl GuardrailVerifier {
    /// Create a verifier with the public key from the spec
    pub fn from_spec(spec: &GuardrailSpec) -> Self {
        // Decode public key from hex (signing script outputs hex)
        let public_key_bytes = hex::decode(&spec.public_key)
            .unwrap_or_else(|_| {
                // Try base64 if hex fails
                general_purpose::STANDARD.decode(&spec.public_key).unwrap_or_default()
            });
        
        Self {
            public_key_bytes,
        }
    }

    /// Verify the signature of the guardrails spec
    pub fn verify(&self, spec: &GuardrailSpec) -> GuardrailResult<()> {
        // Check if signature is empty (unsigned)
        if spec.signature.is_empty() || spec.public_key.is_empty() {
            return Err(GuardrailError::UnsignedSpec);
        }

        // Compute hash of spec (excluding signature and spec_hash fields)
        let spec_hash = self.compute_spec_hash(spec)?;

        // Verify that stored spec_hash matches computed hash
        if spec.spec_hash != spec_hash {
            return Err(GuardrailError::InvalidSignature(
                "Spec hash mismatch - file may have been tampered".to_string(),
            ));
        }

        // Decode signature (signing script outputs base64)
        let signature_bytes = general_purpose::STANDARD.decode(&spec.signature)
            .or_else(|_| hex::decode(&spec.signature))
            .map_err(|_| GuardrailError::InvalidSignature("Invalid signature encoding".to_string()))?;

        // Verify signature using Ed25519
        // The public key from openssl is in DER format, but we need raw 32 bytes
        // If it's DER format (longer), extract the raw key
        let raw_public_key = if self.public_key_bytes.len() == 32 {
            // Already raw Ed25519 public key
            self.public_key_bytes.clone()
        } else if self.public_key_bytes.len() > 32 {
            // Likely DER format - extract raw key (last 32 bytes or parse DER)
            // For simplicity, try last 32 bytes first
            if self.public_key_bytes.len() >= 32 {
                self.public_key_bytes[self.public_key_bytes.len() - 32..].to_vec()
            } else {
                return Err(GuardrailError::InvalidSignature(
                    format!("Invalid Ed25519 public key length: {} (expected 32 or DER)", self.public_key_bytes.len())
                ));
            }
        } else {
            return Err(GuardrailError::InvalidSignature(
                format!("Invalid Ed25519 public key length: {} (expected 32)", self.public_key_bytes.len())
            ));
        };

        let public_key = UnparsedPublicKey::new(&ED25519, &raw_public_key);
        public_key.verify(spec_hash.as_bytes(), &signature_bytes)
            .map_err(|e| GuardrailError::InvalidSignature(format!("Signature verification failed: {:?}", e)))?;

        Ok(())
    }

    /// Compute SHA-256 hash of the spec (excluding signature fields)
    fn compute_spec_hash(&self, spec: &GuardrailSpec) -> GuardrailResult<String> {
        // Serialize spec to JSON, but exclude signature and spec_hash for hashing
        let mut spec_for_hash = spec.clone();
        spec_for_hash.signature = String::new();
        spec_for_hash.spec_hash = String::new();

        let json = serde_json::to_string(&spec_for_hash)
            .map_err(|e| GuardrailError::Json(e))?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        
        Ok(hex::encode(hash))
    }

    /// Verify signature and return the spec if valid
    pub fn verify_and_load(loader: &crate::loader::GuardrailLoader) -> GuardrailResult<GuardrailSpec> {
        let spec = loader.load()?;
        let verifier = Self::from_spec(&spec);
        verifier.verify(&spec)?;
        Ok(spec)
    }
}

