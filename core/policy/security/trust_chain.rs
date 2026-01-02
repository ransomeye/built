// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/security/trust_chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust chain verification for policy signatures

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use sha2::{Sha256, Digest};
use hex;
use tracing::{error, debug, warn};
use parking_lot::RwLock;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct TrustAnchor {
    pub key_id: String,
    pub public_key: Vec<u8>,
    pub fingerprint: String,
}

#[derive(Debug, Clone)]
pub struct TrustedKey {
    pub key_id: String,
    pub public_key: Vec<u8>,
    pub signed_by: Option<String>,
    pub fingerprint: String,
}

static TRUST_CHAIN: Lazy<Arc<RwLock<TrustChain>>> = Lazy::new(|| {
    Arc::new(RwLock::new(TrustChain::new()))
});

pub struct TrustChain {
    anchors: HashMap<String, TrustAnchor>,
    trusted_keys: HashMap<String, TrustedKey>,
}

impl TrustChain {
    fn new() -> Self {
        Self {
            anchors: HashMap::new(),
            trusted_keys: HashMap::new(),
        }
    }

    pub fn load_root_anchor(&mut self, key_id: &str, public_key: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let fingerprint = Self::compute_fingerprint(&public_key);
        
        let anchor = TrustAnchor {
            key_id: key_id.to_string(),
            public_key: public_key.clone(),
            fingerprint: fingerprint.clone(),
        };

        self.anchors.insert(key_id.to_string(), anchor);
        debug!("Loaded trust anchor: {} (fingerprint: {})", key_id, fingerprint);
        Ok(())
    }

    pub fn add_trusted_key(&mut self, key_id: &str, public_key: Vec<u8>, signed_by: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let fingerprint = Self::compute_fingerprint(&public_key);
        
        let trusted_key = TrustedKey {
            key_id: key_id.to_string(),
            public_key: public_key.clone(),
            signed_by: signed_by.clone(),
            fingerprint: fingerprint.clone(),
        };

        self.trusted_keys.insert(key_id.to_string(), trusted_key);
        debug!("Added trusted key: {} (fingerprint: {})", key_id, fingerprint);
        Ok(())
    }

    pub fn verify_trust_chain(&self, key_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if self.anchors.contains_key(key_id) {
            debug!("Key {} is a trust anchor", key_id);
            return Ok(true);
        }

        if let Some(trusted_key) = self.trusted_keys.get(key_id) {
            if let Some(ref signed_by) = trusted_key.signed_by {
                return self.verify_trust_chain(signed_by);
            } else {
                warn!("Trusted key {} has no signer", key_id);
                return Ok(false);
            }
        }

        error!("Key {} not found in trust chain", key_id);
        Ok(false)
    }

    pub fn get_public_key(&self, key_id: &str) -> Option<&[u8]> {
        if let Some(anchor) = self.anchors.get(key_id) {
            return Some(&anchor.public_key);
        }
        if let Some(trusted_key) = self.trusted_keys.get(key_id) {
            return Some(&trusted_key.public_key);
        }
        None
    }

    fn compute_fingerprint(public_key: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        hex::encode(hasher.finalize())
    }

    pub fn load_from_directory(&mut self, trust_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Path::new(trust_dir);
        if !dir.exists() {
            return Err(format!("Trust directory not found: {}", trust_dir).into());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let key_data = fs::read_to_string(&path)?;
                let key_bytes = base64::decode(key_data.trim())?;
                let key_id = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                if path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str()) == Some("anchors") {
                    self.load_root_anchor(&key_id, key_bytes)?;
                } else {
                    self.add_trusted_key(&key_id, key_bytes, None)?;
                }
            }
        }

        debug!("Loaded trust chain from {}", trust_dir);
        Ok(())
    }
}

pub fn initialize_trust_chain(trust_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut chain = TRUST_CHAIN.write();
    chain.load_from_directory(trust_dir)
}

pub fn verify_key_in_trust_chain(key_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let chain = TRUST_CHAIN.read();
    chain.verify_trust_chain(key_id)
}

pub fn get_public_key_from_chain(key_id: &str) -> Option<Vec<u8>> {
    let chain = TRUST_CHAIN.read();
    chain.get_public_key(key_id).map(|k| k.to_vec())
}

use std::sync::Arc;

