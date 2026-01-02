// Path and File Name : /home/ransomeye/rebuild/ransomeye_reporting/src/hasher.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cryptographic hashing - creates SHA-256 hashes for evidence integrity verification and hash chaining

use sha2::{Sha256, Digest};
use serde_json::Value;
use hex;
use std::collections::VecDeque;

/// Cryptographic hasher for evidence integrity
/// Uses SHA-256 for all hashing operations
pub struct EvidenceHasher;

impl EvidenceHasher {
    pub fn new() -> Self {
        Self
    }
    
    /// Hash evidence bundle (SHA-256)
    /// Returns hex-encoded hash
    pub fn hash_evidence(&self, evidence: &Value) -> String {
        let json_bytes = serde_json::to_vec(evidence)
            .expect("Failed to serialize evidence");
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        let hash = hasher.finalize();
        
        hex::encode(hash)
    }
    
    /// Hash raw bytes (SHA-256)
    pub fn hash_bytes(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        hex::encode(hash)
    }
    
    /// Build hash chain from sequence of evidence bundles
    /// Each bundle's hash includes the previous bundle's hash
    pub fn build_hash_chain(&self, bundles: &[Value]) -> Vec<String> {
        let mut chain = Vec::new();
        let mut previous_hash = String::new();
        
        for bundle in bundles {
            let mut bundle_data = bundle.clone();
            
            // Inject previous hash into bundle for chaining
            if !previous_hash.is_empty() {
                bundle_data.as_object_mut()
                    .unwrap()
                    .insert("previous_bundle_hash".to_string(), Value::String(previous_hash.clone()));
            }
            
            let hash = self.hash_evidence(&bundle_data);
            chain.push(hash.clone());
            previous_hash = hash;
        }
        
        chain
    }
    
    /// Verify hash chain integrity
    /// Returns true if all links are valid
    pub fn verify_hash_chain(&self, bundles: &[Value], expected_chain: &[String]) -> bool {
        if bundles.len() != expected_chain.len() {
            return false;
        }
        
        let computed_chain = self.build_hash_chain(bundles);
        computed_chain == expected_chain
    }
    
    /// Compute Merkle root from evidence tree
    pub fn compute_merkle_root(&self, hashes: &[String]) -> String {
        if hashes.is_empty() {
            return String::new();
        }
        
        if hashes.len() == 1 {
            return hashes[0].clone();
        }
        
        let mut current_level = hashes.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    let combined = format!("{}{}", current_level[i], current_level[i + 1]);
                    let hash = self.hash_bytes(combined.as_bytes());
                    next_level.push(hash);
                } else {
                    // Odd number, hash with itself
                    let combined = format!("{}{}", current_level[i], current_level[i]);
                    let hash = self.hash_bytes(combined.as_bytes());
                    next_level.push(hash);
                }
            }
            
            current_level = next_level;
        }
        
        current_level[0].clone()
    }
}

impl Default for EvidenceHasher {
    fn default() -> Self {
        Self::new()
    }
}

