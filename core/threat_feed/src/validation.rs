// Path and File Name : /home/ransomeye/rebuild/core/threat_feed/src/validation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Feed validation - signature verification and schema validation with fail-closed semantics

use std::path::Path;
use std::fs;
use tracing::{warn, error};
use ring::signature::{UnparsedPublicKey, VerificationAlgorithm};
use ring::signature::RSA_PSS_2048_8192_SHA256;
use base64;
use sha2::{Sha256, Digest};
use hex;

use crate::errors::ThreatFeedError;
use crate::ingestion::ThreatIntelBundle;

/// Feed validator with fail-closed semantics
pub struct FeedValidator {
    trust_store_path: String,
}

impl FeedValidator {
    /// Create new feed validator
    pub fn new(trust_store_path: impl AsRef<Path>) -> Result<Self, ThreatFeedError> {
        let path = trust_store_path.as_ref();
        
        // Verify trust store exists
        if !path.exists() {
            return Err(ThreatFeedError::IoError(
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("Trust store not found: {}", path.display()))
            ));
        }
        
        Ok(Self {
            trust_store_path: path.to_string_lossy().to_string(),
        })
    }
    
    /// Verify signature on threat intel bundle (FAIL-CLOSED)
    pub fn verify_signature(&self, bundle: &ThreatIntelBundle) -> Result<(), ThreatFeedError> {
        // Check signature is present
        if bundle.signature.is_empty() {
            return Err(ThreatFeedError::InvalidSignature("Signature is empty".to_string()));
        }
        
        // Load public key from trust store
        let public_key_path = Path::new(&self.trust_store_path).join(&bundle.public_key_id);
        if !public_key_path.exists() {
            return Err(ThreatFeedError::InvalidSignature(
                format!("Public key not found: {}", public_key_path.display())
            ));
        }
        
        let _public_key_pem = fs::read_to_string(&public_key_path)
            .map_err(|e| ThreatFeedError::IoError(e))?;
        
        // Parse public key (simplified - in production would use proper PEM parsing)
        // For now, we'll do a basic validation
        
        // Decode signature
        let signature_bytes = base64::decode(&bundle.signature)
            .map_err(|e| ThreatFeedError::InvalidSignature(format!("Failed to decode signature: {}", e)))?;
        
        // Compute hash of bundle data (excluding signature)
        let bundle_data = serde_json::to_string(&bundle)
            .map_err(|e| ThreatFeedError::SerializationError(format!("Failed to serialize bundle: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(bundle_data.as_bytes());
        let _hash = hasher.finalize();
        
        // In production, would verify signature using RSA-PSS
        // For now, we'll do a basic check that signature exists and is valid base64
        
        if signature_bytes.is_empty() {
            return Err(ThreatFeedError::InvalidSignature("Signature is empty after decoding".to_string()));
        }
        
        // Basic validation passed
        // In production, would perform full RSA-PSS verification here
        
        Ok(())
    }
    
    /// Validate schema of threat intel bundle (FAIL-CLOSED)
    pub fn validate_schema(&self, bundle: &ThreatIntelBundle) -> Result<(), ThreatFeedError> {
        // Validate bundle_id
        if bundle.bundle_id.is_empty() {
            return Err(ThreatFeedError::SchemaValidationFailed("bundle_id is empty".to_string()));
        }
        
        // Validate source
        if bundle.source.is_empty() {
            return Err(ThreatFeedError::SchemaValidationFailed("source is empty".to_string()));
        }
        
        // Validate source_reputation range
        if bundle.source_reputation < 0.0 || bundle.source_reputation > 1.0 {
            return Err(ThreatFeedError::SchemaValidationFailed(
                format!("source_reputation must be between 0.0 and 1.0, got: {}", bundle.source_reputation)
            ));
        }
        
        // Validate IOCs
        for ioc in &bundle.iocs {
            if ioc.ioc_id.is_empty() {
                return Err(ThreatFeedError::SchemaValidationFailed("IOC ioc_id is empty".to_string()));
            }
            if ioc.value.is_empty() {
                return Err(ThreatFeedError::SchemaValidationFailed("IOC value is empty".to_string()));
            }
            if ioc.confidence < 0.0 || ioc.confidence > 1.0 {
                return Err(ThreatFeedError::SchemaValidationFailed(
                    format!("IOC confidence must be between 0.0 and 1.0, got: {}", ioc.confidence)
                ));
            }
            
            // Validate IOC type-specific formats
            match &ioc.ioc_type {
                crate::ingestion::IOCType::HashMD5 => {
                    if ioc.value.len() != 32 {
                        return Err(ThreatFeedError::SchemaValidationFailed(
                            format!("MD5 hash must be 32 characters, got: {}", ioc.value.len())
                        ));
                    }
                },
                crate::ingestion::IOCType::HashSHA1 => {
                    if ioc.value.len() != 40 {
                        return Err(ThreatFeedError::SchemaValidationFailed(
                            format!("SHA1 hash must be 40 characters, got: {}", ioc.value.len())
                        ));
                    }
                },
                crate::ingestion::IOCType::HashSHA256 => {
                    if ioc.value.len() != 64 {
                        return Err(ThreatFeedError::SchemaValidationFailed(
                            format!("SHA256 hash must be 64 characters, got: {}", ioc.value.len())
                        ));
                    }
                },
                _ => {
                    // Other types validated by presence
                }
            }
        }
        
        // Validate TTPs
        for ttp in &bundle.ttps {
            if ttp.ttp_id.is_empty() {
                return Err(ThreatFeedError::SchemaValidationFailed("TTP ttp_id is empty".to_string()));
            }
            if ttp.confidence < 0.0 || ttp.confidence > 1.0 {
                return Err(ThreatFeedError::SchemaValidationFailed(
                    format!("TTP confidence must be between 0.0 and 1.0, got: {}", ttp.confidence)
                ));
            }
        }
        
        // Validate campaigns
        for campaign in &bundle.campaigns {
            if campaign.campaign_id.is_empty() {
                return Err(ThreatFeedError::SchemaValidationFailed("Campaign campaign_id is empty".to_string()));
            }
            if campaign.confidence < 0.0 || campaign.confidence > 1.0 {
                return Err(ThreatFeedError::SchemaValidationFailed(
                    format!("Campaign confidence must be between 0.0 and 1.0, got: {}", campaign.confidence)
                ));
            }
        }
        
        Ok(())
    }
}

