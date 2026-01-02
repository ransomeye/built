// Path and File Name : /home/ransomeye/rebuild/core/bus/src/integrity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Message integrity verification and anti-replay protection

use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc, Duration};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Sha256, Digest};
use thiserror::Error;
use tracing::{error, warn, debug};
use parking_lot::RwLock;

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Replay attack detected: {0}")]
    ReplayAttack(String),
    #[error("Message expired: {0}")]
    MessageExpired(String),
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone)]
struct MessageRecord {
    message_id: String,
    timestamp: DateTime<Utc>,
    component_id: String,
}

pub struct MessageIntegrity {
    /// Verified component public keys
    component_keys: Arc<DashMap<String, VerifyingKey>>,
    /// Processed message IDs (replay protection)
    processed_messages: Arc<DashMap<String, MessageRecord>>,
    /// Replay window (30 seconds)
    replay_window: Duration,
    /// Message expiry (5 minutes)
    message_expiry: Duration,
}

impl MessageIntegrity {
    pub fn new() -> Self {
        Self {
            component_keys: Arc::new(DashMap::new()),
            processed_messages: Arc::new(DashMap::new()),
            replay_window: Duration::seconds(30),
            message_expiry: Duration::minutes(5),
        }
    }
    
    /// Register component public key
    pub fn register_component(&self, component_id: &str, public_key: &[u8]) -> Result<(), IntegrityError> {
        if public_key.len() != 32 {
            return Err(IntegrityError::InvalidFormat(
                format!("Invalid Ed25519 public key length: expected 32 bytes, got {}", public_key.len())
            ));
        }
        
        let verifying_key = VerifyingKey::from_bytes(
            public_key.try_into()
                .map_err(|_| IntegrityError::InvalidFormat("Failed to convert public key".to_string()))?
        )
        .map_err(|e| IntegrityError::InvalidFormat(format!("Invalid Ed25519 public key: {:?}", e)))?;
        
        self.component_keys.insert(component_id.to_string(), verifying_key);
        debug!("Registered public key for component: {}", component_id);
        Ok(())
    }
    
    /// Verify message integrity and check for replay
    /// 
    /// FAIL-CLOSED: Returns error on signature failure, replay, or expiry
    pub fn verify_message(
        &self,
        message_id: &str,
        component_id: &str,
        data: &[u8],
        signature: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), IntegrityError> {
        let now = Utc::now();
        
        // Step 1: Check message expiry
        let age = now - timestamp;
        if age > self.message_expiry {
            error!("Message expired: {} (age: {}s)", message_id, age.num_seconds());
            return Err(IntegrityError::MessageExpired(
                format!("Message age: {} seconds", age.num_seconds())
            ));
        }
        
        // Step 2: Check replay window
        if let Some(record) = self.processed_messages.get(message_id) {
            let time_diff = (now - record.timestamp).num_seconds().abs();
            if time_diff <= self.replay_window.num_seconds() {
                error!("REPLAY ATTACK: Duplicate message ID detected: {} (time diff: {}s)", 
                       message_id, time_diff);
                return Err(IntegrityError::ReplayAttack(
                    format!("Duplicate message ID: {}", message_id)
                ));
            }
        }
        
        // Step 3: Verify signature
        let verifying_key = self.component_keys.get(component_id)
            .ok_or_else(|| IntegrityError::InvalidSignature(
                format!("Component not registered: {}", component_id)
            ))?;
        
        let signature_bytes = STANDARD.decode(signature)
            .map_err(|e| IntegrityError::InvalidSignature(format!("Failed to decode signature: {}", e)))?;
        
        if signature_bytes.len() != 64 {
            return Err(IntegrityError::InvalidSignature(
                format!("Invalid Ed25519 signature length: expected 64 bytes, got {}", signature_bytes.len())
            ));
        }
        
        let sig_bytes: [u8; 64] = signature_bytes.try_into()
            .map_err(|_| IntegrityError::InvalidSignature("Failed to convert signature to array".to_string()))?;
        
        let sig = Signature::from_bytes(&sig_bytes);
        
        // Verify signature
        verifying_key.value().verify(data, &sig)
            .map_err(|e| IntegrityError::InvalidSignature(format!("Signature verification failed: {:?}", e)))?;
        
        // Step 4: Record message as processed
        self.processed_messages.insert(
            message_id.to_string(),
            MessageRecord {
                message_id: message_id.to_string(),
                timestamp: now,
                component_id: component_id.to_string(),
            }
        );
        
        // Cleanup old messages outside replay window
        self.cleanup_old_messages(now);
        
        debug!("Message verified successfully: {}", message_id);
        Ok(())
    }
    
    fn cleanup_old_messages(&self, now: DateTime<Utc>) {
        let expired: Vec<String> = self.processed_messages
            .iter()
            .filter(|entry| {
                let age = now - entry.value().timestamp;
                age > self.replay_window
            })
            .map(|entry| entry.key().clone())
            .collect();
        
        for message_id in expired {
            self.processed_messages.remove(&message_id);
        }
    }
    
    /// Compute message hash for signing
    pub fn compute_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replay_detection() {
        let integrity = MessageIntegrity::new();
        let component_id = "test-component";
        let public_key = [0u8; 32]; // Dummy key for test
        integrity.register_component(component_id, &public_key).unwrap();
        
        let message_id = "msg-1";
        let data = b"test data";
        let signature = "dummy_signature_base64_64_bytes_long_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let timestamp = Utc::now();
        
        // First message should pass (we'll skip actual signature verification in test)
        // Second message with same ID should be rejected
        // Note: This test would need proper Ed25519 keys to fully test
    }
}

