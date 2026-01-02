// Path and File Name : /home/ransomeye/rebuild/core/deception/src/signals.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: High-confidence deception signals with cryptographic signatures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use ed25519_dalek::{SigningKey, Signer};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fs;

use crate::errors::DeceptionError;
use crate::asset::DeceptionAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionSignal {
    pub signal_id: String,
    pub asset_id: String,
    pub interaction_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence_score: f64,
    pub hash: String,
    pub signature: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct SignalGenerator {
    signing_key: SigningKey,
}

impl SignalGenerator {
    /// Create new signal generator from private key file
    pub fn new(private_key_path: &str) -> Result<Self, DeceptionError> {
        let key_bytes = fs::read(private_key_path)
            .map_err(|e| DeceptionError::ConfigurationError(
                format!("Failed to read private key from {}: {}", private_key_path, e)
            ))?;
        
        let signing_key = SigningKey::from_bytes(
            key_bytes.as_slice().try_into()
                .map_err(|_| DeceptionError::ConfigurationError(
                    "Invalid private key length (expected 32 bytes)".to_string()
                ))?
        );
        
        Ok(Self { signing_key })
    }
    
    /// Generate high-confidence signal from asset interaction
    /// FAIL-CLOSED: Only generates signals with confidence >= 0.9
    pub fn generate_signal(
        &self,
        asset: &DeceptionAsset,
        interaction_type: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<DeceptionSignal, DeceptionError> {
        // Validate interaction type matches trigger conditions
        if !asset.trigger_conditions.interaction_types.contains(&interaction_type) {
            return Err(DeceptionError::SignalGenerationFailed(
                format!("Interaction type '{}' not in trigger_conditions", interaction_type)
            ));
        }
        
        // Generate signal with high confidence (>= 0.9)
        let confidence_score = asset.trigger_conditions.min_confidence.max(0.9);
        
        if confidence_score < 0.9 {
            return Err(DeceptionError::SignalGenerationFailed(
                format!("Confidence score {} is below minimum threshold 0.9", confidence_score)
            ));
        }
        
        let signal_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Create signal (without signature first)
        let mut signal = DeceptionSignal {
            signal_id: signal_id.clone(),
            asset_id: asset.asset_id.clone(),
            interaction_type: interaction_type.clone(),
            timestamp,
            confidence_score,
            hash: String::new(), // Will be computed
            signature: String::new(), // Will be computed
            metadata,
        };
        
        // Compute hash
        let hash = Self::compute_signal_hash(&signal)?;
        signal.hash = hash.clone();
        
        // Sign signal
        let signature_bytes = self.signing_key.sign(hash.as_bytes());
        signal.signature = STANDARD.encode(signature_bytes.to_bytes());
        
        Ok(signal)
    }
    
    /// Compute hash of signal (excluding signature field)
    fn compute_signal_hash(signal: &DeceptionSignal) -> Result<String, DeceptionError> {
        let mut hasher = Sha256::new();
        
        hasher.update(signal.signal_id.as_bytes());
        hasher.update(signal.asset_id.as_bytes());
        hasher.update(signal.interaction_type.as_bytes());
        hasher.update(signal.timestamp.to_rfc3339().as_bytes());
        hasher.update(signal.confidence_score.to_string().as_bytes());
        hasher.update(serde_json::to_string(&signal.metadata)
            .map_err(|e| DeceptionError::Json(e))?.as_bytes());
        
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }
}

impl DeceptionSignal {
    /// Validate signal has required fields and signature
    pub fn validate(&self) -> Result<(), DeceptionError> {
        // Validate confidence >= 0.9
        if self.confidence_score < 0.9 {
            return Err(DeceptionError::SignalSignatureInvalid(
                format!("Signal confidence {} is below minimum threshold 0.9", self.confidence_score)
            ));
        }
        
        // Validate hash is not empty
        if self.hash.is_empty() {
            return Err(DeceptionError::SignalSignatureInvalid(
                "Signal hash is empty".to_string()
            ));
        }
        
        // Validate signature is not empty
        if self.signature.is_empty() {
            return Err(DeceptionError::SignalSignatureInvalid(
                "Signal signature is empty".to_string()
            ));
        }
        
        Ok(())
    }
}

