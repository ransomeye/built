// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/verifier.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evidence verifier - validates evidence integrity, detects tampering, and verifies hash chains

use std::path::Path;
use std::fs;
use tracing::{debug, error, warn};

use crate::errors::ReportingError;
use crate::evidence_store::EvidenceStore;
use crate::hasher::EvidenceHasher;

/// Evidence verifier - validates evidence integrity and detects corruption
pub struct EvidenceVerifier {
    hasher: EvidenceHasher,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub bundle_count: usize,
    pub verified_bundles: usize,
    pub corrupted_bundles: usize,
}

impl EvidenceVerifier {
    pub fn new() -> Self {
        Self {
            hasher: EvidenceHasher::new(),
        }
    }
    
    /// Verify evidence store integrity
    pub fn verify_store(&self, store: &EvidenceStore) -> Result<VerificationResult, ReportingError> {
        let bundles = store.get_all_bundles();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut verified_count = 0;
        let mut corrupted_count = 0;
        
        debug!("Verifying {} evidence bundles", bundles.len());
        
        // Verify each bundle
        for bundle in &bundles {
            match store.verify_bundle_integrity(bundle) {
                Ok(true) => {
                    verified_count += 1;
                }
                Ok(false) => {
                    corrupted_count += 1;
                    errors.push(format!("Bundle {} failed integrity check", bundle.bundle_id));
                }
                Err(e) => {
                    corrupted_count += 1;
                    errors.push(format!("Bundle {} verification error: {}", bundle.bundle_id, e));
                }
            }
            
            // Check if bundle is sealed
            if !bundle.is_sealed {
                warnings.push(format!("Bundle {} is not sealed", bundle.bundle_id));
            }
            
            // Verify hash chain
            if let Some(prev_hash) = &bundle.previous_bundle_hash {
                // Find previous bundle
                let prev_bundle = bundles.iter()
                    .find(|b| b.bundle_hash == *prev_hash);
                
                if prev_bundle.is_none() {
                    errors.push(format!(
                        "Bundle {} references non-existent previous bundle hash {}",
                        bundle.bundle_id, prev_hash
                    ));
                }
            }
        }
        
        // Verify hash chain continuity
        let mut prev_hash: Option<String> = None;
        for bundle in &bundles {
            if let Some(ref expected_prev) = prev_hash {
                if bundle.previous_bundle_hash.as_ref() != Some(expected_prev) {
                    errors.push(format!(
                        "Bundle {} has broken hash chain (expected previous: {}, got: {:?})",
                        bundle.bundle_id,
                        expected_prev,
                        bundle.previous_bundle_hash
                    ));
                }
            }
            prev_hash = Some(bundle.bundle_hash.clone());
        }
        
        let is_valid = errors.is_empty() && corrupted_count == 0;
        
        if !is_valid {
            error!("Evidence store verification failed: {} errors, {} corrupted bundles", 
                   errors.len(), corrupted_count);
        } else {
            debug!("Evidence store verification passed: {} bundles verified", verified_count);
        }
        
        Ok(VerificationResult {
            is_valid,
            errors,
            warnings,
            bundle_count: bundles.len(),
            verified_bundles: verified_count,
            corrupted_bundles: corrupted_count,
        })
    }
    
    /// Verify single bundle file on disk
    pub fn verify_bundle_file(&self, bundle_path: impl AsRef<Path>) -> Result<bool, ReportingError> {
        let bundle_data = fs::read_to_string(bundle_path)
            .map_err(|e| ReportingError::IoError(e))?;
        
        let bundle: crate::evidence_store::EvidenceBundle = serde_json::from_str(&bundle_data)
            .map_err(|e| ReportingError::SerializationError(e))?;
        
        // Recompute hash
        let bundle_value = serde_json::to_value(&bundle)
            .map_err(|e| ReportingError::SerializationError(e))?;
        let computed_hash = self.hasher.hash_evidence(&bundle_value);
        
        if computed_hash != bundle.bundle_hash {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Detect tampering in evidence store
    /// Returns list of tampered bundle IDs
    pub fn detect_tampering(&self, store: &EvidenceStore) -> Result<Vec<String>, ReportingError> {
        let bundles = store.get_all_bundles();
        let mut tampered = Vec::new();
        
        for bundle in &bundles {
            match store.verify_bundle_integrity(bundle) {
                Ok(false) => {
                    tampered.push(bundle.bundle_id.clone());
                }
                Err(_) => {
                    tampered.push(bundle.bundle_id.clone());
                }
                _ => {}
            }
        }
        
        if !tampered.is_empty() {
            warn!("Detected tampering in {} bundles", tampered.len());
        }
        
        Ok(tampered)
    }
}

impl Default for EvidenceVerifier {
    fn default() -> Self {
        Self::new()
    }
}

