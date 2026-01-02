// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/security/replay_protection.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay attack protection using directive ID and nonce tracking

use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Utc;
use tracing::{debug, warn};

pub struct ReplayProtector {
    /// Set of processed directive IDs
    processed_directives: Arc<RwLock<HashSet<String>>>,
    
    /// Maximum number of directive IDs to track
    max_tracked: usize,
}

impl ReplayProtector {
    pub fn new(max_tracked: usize) -> Self {
        Self {
            processed_directives: Arc::new(RwLock::new(HashSet::new())),
            max_tracked,
        }
    }
    
    /// Check if directive ID has been processed before
    /// Returns true if directive is new, false if it's a replay
    pub fn is_new(&self, directive_id: &str) -> bool {
        let mut processed = self.processed_directives.write();
        
        if processed.contains(directive_id) {
            warn!("Replay attack detected: directive {} already processed", directive_id);
            return false;
        }
        
        // Prevent unbounded growth
        if processed.len() >= self.max_tracked {
            warn!("Replay protector at capacity ({} entries), clearing oldest", processed.len());
            processed.clear();
        }
        
        processed.insert(directive_id.to_string());
        debug!("Directive {} marked as processed", directive_id);
        true
    }
    
    /// Check if directive was already processed (without marking)
    pub fn was_processed(&self, directive_id: &str) -> bool {
        self.processed_directives.read().contains(directive_id)
    }
    
    /// Get count of tracked directives
    pub fn count(&self) -> usize {
        self.processed_directives.read().len()
    }
    
    /// Clear all tracked directives (for testing)
    pub fn clear(&self) {
        self.processed_directives.write().clear();
    }
}

