// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/evidence_store.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Immutable evidence store - provides append-only storage with hash chaining and cryptographic signatures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use parking_lot::RwLock;
use tracing::{debug, error, warn};
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use ring::rand::SystemRandom;
use base64::{Engine as _, engine::general_purpose};

use crate::errors::ReportingError;
use crate::hasher::EvidenceHasher;
use crate::collector::CollectedEvidence;

/// Evidence bundle - sealed and immutable once created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub bundle_id: String,
    pub created_at: DateTime<Utc>,
    pub sealed_at: Option<DateTime<Utc>>,
    pub engine_version: String,
    pub policy_version: String,
    pub evidence_items: Vec<CollectedEvidence>,
    pub bundle_hash: String,
    pub previous_bundle_hash: Option<String>,
    pub signature: Option<String>,
    pub is_sealed: bool,
}

/// Immutable evidence store with hash chaining
pub struct EvidenceStore {
    store_path: PathBuf,
    hasher: EvidenceHasher,
    key_pair: Option<Ed25519KeyPair>,
    bundles: RwLock<Vec<EvidenceBundle>>,
    last_bundle_hash: RwLock<Option<String>>,
}

impl EvidenceStore {
    /// Create new evidence store
    /// If signing_key_path is provided, bundles will be cryptographically signed
    pub fn new(store_path: impl AsRef<Path>, signing_key_path: Option<&Path>) -> Result<Self, ReportingError> {
        let store_path = store_path.as_ref().to_path_buf();
        
        // Create store directory if it doesn't exist
        fs::create_dir_all(&store_path)
            .map_err(|e| ReportingError::IoError(e))?;
        
        // Load or generate signing key
        let key_pair = if let Some(key_path) = signing_key_path {
            if key_path.exists() {
                let key_data = fs::read(key_path)
                    .map_err(|e| ReportingError::IoError(e))?;
                Some(Ed25519KeyPair::from_pkcs8(&key_data)
                    .map_err(|e| ReportingError::SignatureVerificationFailed(format!("Failed to load key: {:?}", e)))?)
            } else {
                // Generate new key pair
                let rng = SystemRandom::new();
                let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
                    .map_err(|e| ReportingError::SignatureVerificationFailed(format!("Failed to generate key: {:?}", e)))?;
                
                // Save private key
                fs::write(key_path, pkcs8_bytes.as_ref())
                    .map_err(|e| ReportingError::IoError(e))?;
                
                Some(Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                    .map_err(|e| ReportingError::SignatureVerificationFailed(format!("Failed to create key pair: {:?}", e)))?)
            }
        } else {
            None
        };
        
        let store = Self {
            store_path,
            hasher: EvidenceHasher::new(),
            key_pair,
            bundles: RwLock::new(Vec::new()),
            last_bundle_hash: RwLock::new(None),
        };
        
        // Load existing bundles
        store.load_bundles()?;
        
        Ok(store)
    }
    
    /// Load existing bundles from disk
    fn load_bundles(&self) -> Result<(), ReportingError> {
        let bundles_dir = self.store_path.join("bundles");
        if !bundles_dir.exists() {
            return Ok(());
        }
        
        let mut bundles = Vec::new();
        let mut bundle_files: Vec<PathBuf> = fs::read_dir(&bundles_dir)
            .map_err(|e| ReportingError::IoError(e))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
            .collect();
        
        bundle_files.sort();
        
        for bundle_file in bundle_files {
            let bundle_data = fs::read_to_string(&bundle_file)
                .map_err(|e| ReportingError::IoError(e))?;
            let bundle: EvidenceBundle = serde_json::from_str(&bundle_data)
                .map_err(|e| ReportingError::SerializationError(e))?;
            
            // Verify bundle integrity
            if !self.verify_bundle_integrity(&bundle)? {
                error!("Bundle {} failed integrity check", bundle.bundle_id);
                return Err(ReportingError::EvidenceCorrupted(
                    format!("Bundle {} is corrupted", bundle.bundle_id)
                ));
            }
            
            bundles.push(bundle);
        }
        
        // Update last bundle hash
        if let Some(last_bundle) = bundles.last() {
            *self.last_bundle_hash.write() = Some(last_bundle.bundle_hash.clone());
        }
        
        *self.bundles.write() = bundles;
        
        debug!("Loaded {} evidence bundles", self.bundles.read().len());
        Ok(())
    }
    
    /// Create new evidence bundle
    /// Bundle is mutable until sealed
    pub fn create_bundle(
        &self,
        engine_version: &str,
        policy_version: &str,
    ) -> Result<String, ReportingError> {
        let bundle_id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        
        let bundle = EvidenceBundle {
            bundle_id: bundle_id.clone(),
            created_at,
            sealed_at: None,
            engine_version: engine_version.to_string(),
            policy_version: policy_version.to_string(),
            evidence_items: Vec::new(),
            bundle_hash: String::new(),
            previous_bundle_hash: self.last_bundle_hash.read().clone(),
            signature: None,
            is_sealed: false,
        };
        
        self.bundles.write().push(bundle);
        
        debug!("Created evidence bundle {}", bundle_id);
        Ok(bundle_id)
    }
    
    /// Add evidence to bundle
    /// Fails if bundle is already sealed
    pub fn add_evidence(
        &self,
        bundle_id: &str,
        evidence: CollectedEvidence,
    ) -> Result<(), ReportingError> {
        let mut bundles = self.bundles.write();
        
        let bundle = bundles.iter_mut()
            .find(|b| b.bundle_id == bundle_id)
            .ok_or_else(|| ReportingError::MissingEvidence(format!("Bundle {} not found", bundle_id)))?;
        
        if bundle.is_sealed {
            return Err(ReportingError::BundleSealed(format!("Bundle {} is already sealed", bundle_id)));
        }
        
        bundle.evidence_items.push(evidence);
        
        debug!("Added evidence to bundle {}", bundle_id);
        Ok(())
    }
    
    /// Seal evidence bundle (make it immutable)
    /// Computes hash, signs bundle, and saves to disk
    pub fn seal_bundle(&self, bundle_id: &str) -> Result<(), ReportingError> {
        let mut bundles = self.bundles.write();
        
        let bundle = bundles.iter_mut()
            .find(|b| b.bundle_id == bundle_id)
            .ok_or_else(|| ReportingError::MissingEvidence(format!("Bundle {} not found", bundle_id)))?;
        
        if bundle.is_sealed {
            return Err(ReportingError::BundleSealed(format!("Bundle {} is already sealed", bundle_id)));
        }
        
        // Compute bundle hash
        let bundle_hash = {
            let bundle_value = serde_json::to_value(&*bundle)
                .map_err(|e| ReportingError::SerializationError(e))?;
            self.hasher.hash_evidence(&bundle_value)
        };
        bundle.bundle_hash = bundle_hash;
        
        // Sign bundle if key pair is available
        if let Some(key_pair) = &self.key_pair {
            let message = bundle.bundle_hash.as_bytes();
            let signature = key_pair.sign(message);
            bundle.signature = Some(general_purpose::STANDARD.encode(signature.as_ref()));
        }
        
        bundle.sealed_at = Some(Utc::now());
        bundle.is_sealed = true;
        
        // Save bundle to disk
        self.save_bundle(bundle)?;
        
        // Update last bundle hash
        *self.last_bundle_hash.write() = Some(bundle.bundle_hash.clone());
        
        debug!("Sealed evidence bundle {} (hash: {})", bundle_id, bundle.bundle_hash);
        Ok(())
    }
    
    /// Save bundle to disk
    fn save_bundle(&self, bundle: &EvidenceBundle) -> Result<(), ReportingError> {
        let bundles_dir = self.store_path.join("bundles");
        fs::create_dir_all(&bundles_dir)
            .map_err(|e| ReportingError::IoError(e))?;
        
        let bundle_file = bundles_dir.join(format!("{}.json", bundle.bundle_id));
        let bundle_json = serde_json::to_string_pretty(bundle)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        fs::write(&bundle_file, bundle_json)
            .map_err(|e| ReportingError::IoError(e))?;
        
        Ok(())
    }
    
    /// Verify bundle integrity
    pub fn verify_bundle_integrity(&self, bundle: &EvidenceBundle) -> Result<bool, ReportingError> {
        // Recompute hash
        let bundle_value = serde_json::to_value(bundle)
            .map_err(|e| ReportingError::SerializationError(e))?;
        let computed_hash = self.hasher.hash_evidence(&bundle_value);
        
        if computed_hash != bundle.bundle_hash {
            return Ok(false);
        }
        
        // Verify signature if present
        if let (Some(signature_b64), Some(key_pair)) = (&bundle.signature, &self.key_pair) {
            let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
                .map_err(|e| ReportingError::SignatureVerificationFailed(format!("Invalid signature encoding: {}", e)))?;
            
            let public_key = key_pair.public_key();
            let unparsed_key = UnparsedPublicKey::new(&ED25519, public_key.as_ref());
            
            unparsed_key.verify(
                bundle.bundle_hash.as_bytes(),
                &signature_bytes,
            ).map_err(|e| ReportingError::SignatureVerificationFailed(format!("Signature verification failed: {:?}", e)))?;
        }
        
        Ok(true)
    }
    
    /// Get bundle by ID
    pub fn get_bundle(&self, bundle_id: &str) -> Result<EvidenceBundle, ReportingError> {
        let bundles = self.bundles.read();
        bundles.iter()
            .find(|b| b.bundle_id == bundle_id)
            .cloned()
            .ok_or_else(|| ReportingError::MissingEvidence(format!("Bundle {} not found", bundle_id)))
    }
    
    /// Get all bundles
    pub fn get_all_bundles(&self) -> Vec<EvidenceBundle> {
        self.bundles.read().clone()
    }
    
    /// Get bundles in time range
    pub fn get_bundles_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<EvidenceBundle> {
        self.bundles.read()
            .iter()
            .filter(|b| b.created_at >= start && b.created_at <= end)
            .cloned()
            .collect()
    }
}

