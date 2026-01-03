// Path and File Name : /home/ransomeye/rebuild/core/deception/src/playbook_integration.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Integration with Phase 6 playbooks - explicit mapping from deception signals to playbook IDs

#![cfg(feature = "future-deception")]

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn, debug};

use crate::signals::DeceptionSignal;
use crate::errors::DeceptionError;

/// Playbook integration for deception signals
/// 
/// Rules:
/// - Explicit mapping only (no implicit execution)
/// - Signal → Playbook ID mapping must be configured
/// - Missing mapping → NO ACTION (fail-closed)
/// - Returns playbook IDs for Phase 6 execution
pub struct PlaybookIntegration {
    /// Explicit mapping: signal interaction_type → playbook_id
    signal_to_playbook: Arc<RwLock<HashMap<String, String>>>,
}

impl PlaybookIntegration {
    /// Create new playbook integration
    pub fn new() -> Self {
        Self {
            signal_to_playbook: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Load playbook mappings from environment or configuration
    /// 
    /// Format: DECEPTION_PLAYBOOK_MAPPINGS="credential_lure_touched:containment_playbook_id,decoy_ssh_accessed:isolation_playbook_id"
    pub fn load_mappings(&self) -> Result<(), DeceptionError> {
        let mappings_str = std::env::var("DECEPTION_PLAYBOOK_MAPPINGS")
            .unwrap_or_else(|_| String::new());
        
        if mappings_str.is_empty() {
            warn!("No deception playbook mappings configured (DECEPTION_PLAYBOOK_MAPPINGS)");
            return Ok(());
        }
        
        let mut mappings = self.signal_to_playbook.write();
        mappings.clear();
        
        // Parse mappings: "key1:value1,key2:value2"
        for mapping in mappings_str.split(',') {
            let parts: Vec<&str> = mapping.split(':').collect();
            if parts.len() == 2 {
                let interaction_type = parts[0].trim().to_string();
                let playbook_id = parts[1].trim().to_string();
                info!("Mapped deception signal '{}' to playbook '{}'", interaction_type, playbook_id);
                mappings.insert(interaction_type, playbook_id);
            } else {
                warn!("Invalid playbook mapping format: {}", mapping);
            }
        }
        
        Ok(())
    }
    
    /// Get playbook ID for signal (explicit mapping only)
    /// 
    /// Returns None if no mapping exists (fail-closed: no implicit execution)
    pub fn get_playbook_for_signal(&self, signal: &DeceptionSignal) -> Option<String> {
        let mappings = self.signal_to_playbook.read();
        mappings.get(&signal.interaction_type).cloned()
    }
    
    /// Add explicit mapping
    pub fn add_mapping(&self, interaction_type: String, playbook_id: String) {
        let mut mappings = self.signal_to_playbook.write();
        mappings.insert(interaction_type, playbook_id);
    }
    
    /// Remove mapping
    pub fn remove_mapping(&self, interaction_type: &str) {
        let mut mappings = self.signal_to_playbook.write();
        mappings.remove(interaction_type);
    }
    
    /// Get all mappings
    pub fn get_all_mappings(&self) -> HashMap<String, String> {
        self.signal_to_playbook.read().clone()
    }
}

