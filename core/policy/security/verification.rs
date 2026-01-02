// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/verification.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy integrity verification using SHA-256

use sha2::{Sha256, Digest};
use hex;
use tracing::{error, debug};

pub struct PolicyVerifier;

impl PolicyVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn verify_hash(&self, content: &str, expected_hash: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let computed_hash = hex::encode(hasher.finalize());

        if computed_hash == expected_hash {
            debug!("Policy hash verification successful");
            true
        } else {
            error!("Policy hash verification failed (hash mismatch)");
            false
        }
    }

    pub fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

