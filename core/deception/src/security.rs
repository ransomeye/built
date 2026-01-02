// Path and File Name : /home/ransomeye/rebuild/core/deception/src/security.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic signature verification for deception assets and signals

use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fs;
use std::path::Path;

use crate::errors::DeceptionError;
use crate::asset::DeceptionAsset;
use crate::signals::DeceptionSignal;

pub struct SignatureVerifier {
    public_key: VerifyingKey,
}

impl SignatureVerifier {
    /// Create new verifier from public key file
    pub fn new(public_key_path: &str) -> Result<Self, DeceptionError> {
        let key_bytes = fs::read(public_key_path)
            .map_err(|e| DeceptionError::ConfigurationError(
                format!("Failed to read public key from {}: {}", public_key_path, e)
            ))?;
        
        let public_key = VerifyingKey::from_bytes(
            key_bytes.as_slice().try_into()
                .map_err(|_| DeceptionError::ConfigurationError(
                    "Invalid public key length (expected 32 bytes)".to_string()
                ))?
        )
        .map_err(|e| DeceptionError::ConfigurationError(
            format!("Failed to parse public key: {}", e)
        ))?;
        
        Ok(Self { public_key })
    }
    
    /// Verify asset signature
    pub fn verify_asset(&self, asset: &DeceptionAsset) -> Result<(), DeceptionError> {
        // Compute hash of asset (excluding signature fields)
        let hash = Self::compute_asset_hash(asset)?;
        
        // Verify hash matches signature_hash
        if hash != asset.signature_hash {
            return Err(DeceptionError::InvalidSignature(
                "Asset signature_hash mismatch".to_string()
            ));
        }
        
        // Decode signature
        let signature_bytes = STANDARD.decode(&asset.signature)
            .map_err(|e| DeceptionError::InvalidSignature(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        let signature = Signature::from_bytes(
            signature_bytes.as_slice().try_into()
                .map_err(|_| DeceptionError::InvalidSignature(
                    "Invalid signature length (expected 64 bytes)".to_string()
                ))?
        );
        
        // Verify signature
        self.public_key.verify(hash.as_bytes(), &signature)
            .map_err(|e| DeceptionError::InvalidSignature(
                format!("Signature verification failed: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// Verify signal signature
    pub fn verify_signal(&self, signal: &DeceptionSignal) -> Result<(), DeceptionError> {
        // Compute hash of signal (excluding signature field)
        let hash = Self::compute_signal_hash(signal)?;
        
        // Verify hash matches signal hash
        if hash != signal.hash {
            return Err(DeceptionError::SignalSignatureInvalid(
                "Signal hash mismatch".to_string()
            ));
        }
        
        // Decode signature
        let signature_bytes = STANDARD.decode(&signal.signature)
            .map_err(|e| DeceptionError::SignalSignatureInvalid(
                format!("Failed to decode signature: {}", e)
            ))?;
        
        let signature = Signature::from_bytes(
            signature_bytes.as_slice().try_into()
                .map_err(|_| DeceptionError::SignalSignatureInvalid(
                    "Invalid signature length (expected 64 bytes)".to_string()
                ))?
        );
        
        // Verify signature
        self.public_key.verify(hash.as_bytes(), &signature)
            .map_err(|e| DeceptionError::SignalSignatureInvalid(
                format!("Signal signature verification failed: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// Compute hash of asset (excluding signature fields)
    fn compute_asset_hash(asset: &DeceptionAsset) -> Result<String, DeceptionError> {
        // Create a copy without signature fields for hashing
        let mut hasher = Sha256::new();
        
        // Hash all fields except signature and signature_hash
        hasher.update(asset.asset_id.as_bytes());
        hasher.update(serde_json::to_string(&asset.asset_type)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(serde_json::to_string(&asset.deployment_scope)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(serde_json::to_string(&asset.visibility_level)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(serde_json::to_string(&asset.trigger_conditions)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(serde_json::to_string(&asset.telemetry_fields)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(serde_json::to_string(&asset.teardown_procedure)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(asset.max_lifetime.to_string().as_bytes());
        
        if let Some(ref metadata) = asset.metadata {
            hasher.update(serde_json::to_string(metadata)
                .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        }
        
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }
    
    /// Compute hash of signal (excluding signature field)
    fn compute_signal_hash(signal: &DeceptionSignal) -> Result<String, DeceptionError> {
        let mut hasher = Sha256::new();
        
        hasher.update(signal.signal_id.as_bytes());
        hasher.update(signal.asset_id.as_bytes());
        hasher.update(serde_json::to_string(&signal.interaction_type)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        hasher.update(signal.timestamp.to_rfc3339().as_bytes());
        hasher.update(signal.confidence_score.to_string().as_bytes());
        hasher.update(serde_json::to_string(&signal.metadata)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }
}

