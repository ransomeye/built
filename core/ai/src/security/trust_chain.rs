// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/security/trust_chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust chain enforcement for model signatures

use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, warn, info, debug};

use crate::security::signature::SignatureVerifier;

pub struct TrustChain {
    verifiers: Arc<RwLock<HashMap<String, SignatureVerifier>>>,
    root_verifier: Option<SignatureVerifier>,
}

impl TrustChain {
    pub fn new() -> Self {
        Self {
            verifiers: Arc::new(RwLock::new(HashMap::new())),
            root_verifier: None,
        }
    }
    
    /// Load root public key (trust anchor)
    pub fn load_root_key(&mut self, root_key_path: &Path) -> Result<(), String> {
        let root_verifier = SignatureVerifier::new(root_key_path)?;
        self.root_verifier = Some(root_verifier);
        info!("Root trust key loaded from {:?}", root_key_path);
        Ok(())
    }
    
    /// Add verifier for model
    pub fn add_verifier(&self, model_name: String, verifier: SignatureVerifier) {
        let mut verifiers = self.verifiers.write();
        verifiers.insert(model_name, verifier);
        debug!("Verifier added for model");
    }
    
    /// Verify model signature using trust chain
    pub fn verify_model(&self, model_name: &str, data: &[u8], signature: &str) -> Result<bool, String> {
        // Try model-specific verifier first
        {
            let verifiers = self.verifiers.read();
            if let Some(verifier) = verifiers.get(model_name) {
                return verifier.verify(data, signature);
            }
        }
        
        // Fall back to root verifier
        if let Some(ref root) = self.root_verifier {
            root.verify(data, signature)
        } else {
            Err("No verifier available for model".to_string())
        }
    }
    
    /// Check if trust chain is established
    pub fn is_established(&self) -> bool {
        self.root_verifier.is_some()
    }
}

