// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/security/trust_chain.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Trust chain validation for Phase 6 → Phase 7 trust

use std::path::Path;
use std::fs;
use tracing::{debug, error, warn};
use crate::security::signature::SignatureVerifier;

pub struct TrustChain {
    policy_public_key: Option<SignatureVerifier>,
    agent_public_keys: std::collections::HashMap<String, SignatureVerifier>,
}

impl TrustChain {
    pub fn new() -> Self {
        Self {
            policy_public_key: None,
            agent_public_keys: std::collections::HashMap::new(),
        }
    }
    
    /// Load policy public key (for verifying Phase 6 directives)
    pub fn load_policy_key(&mut self, key_path: &str) -> Result<(), String> {
        let key_bytes = fs::read(key_path)
            .map_err(|e| format!("Failed to read policy key from {}: {}", key_path, e))?;
        
        let verifier = SignatureVerifier::new(&key_bytes)
            .map_err(|e| format!("Failed to create signature verifier: {}", e))?;
        
        self.policy_public_key = Some(verifier);
        debug!("Policy public key loaded from {}", key_path);
        Ok(())
    }
    
    /// Load agent public key (for verifying acknowledgments)
    pub fn load_agent_key(&mut self, agent_id: &str, key_path: &str) -> Result<(), String> {
        let key_bytes = fs::read(key_path)
            .map_err(|e| format!("Failed to read agent key from {}: {}", key_path, e))?;
        
        let verifier = SignatureVerifier::new(&key_bytes)
            .map_err(|e| format!("Failed to create signature verifier: {}", e))?;
        
        self.agent_public_keys.insert(agent_id.to_string(), verifier);
        debug!("Agent {} public key loaded from {}", agent_id, key_path);
        Ok(())
    }
    
    /// Verify directive signature (from Phase 6)
    pub fn verify_directive(&self, directive_json: &str, signature: &str) -> Result<bool, String> {
        let verifier = self.policy_public_key.as_ref()
            .ok_or_else(|| "Policy public key not loaded".to_string())?;
        
        verifier.verify_directive(directive_json, signature)
    }
    
    /// Verify acknowledgment signature (from agent)
    pub fn verify_acknowledgment(&self, agent_id: &str, ack_json: &str, signature: &str) -> Result<bool, String> {
        let verifier = self.agent_public_keys.get(agent_id)
            .ok_or_else(|| format!("Agent {} public key not loaded", agent_id))?;
        
        verifier.verify_directive(ack_json, signature)
    }
    
    /// Check if policy key is loaded
    pub fn has_policy_key(&self) -> bool {
        self.policy_public_key.is_some()
    }
    
    /// Check if agent key is loaded
    pub fn has_agent_key(&self, agent_id: &str) -> bool {
        self.agent_public_keys.contains_key(agent_id)
    }
}

