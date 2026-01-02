// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/security/nonce.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Nonce freshness tracking for replay protection

use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, warn};

#[derive(Clone)]
pub struct NonceTracker {
    /// Set of seen nonces
    seen_nonces: Arc<RwLock<HashSet<String>>>,
    
    /// Nonce expiry time (cleanup old nonces)
    nonce_ttl_seconds: i64,
}

impl NonceTracker {
    pub fn new(nonce_ttl_seconds: i64) -> Self {
        Self {
            seen_nonces: Arc::new(RwLock::new(HashSet::new())),
            nonce_ttl_seconds,
        }
    }
    
    /// Check if nonce is fresh (not seen before)
    /// Returns true if nonce is fresh, false if it's a replay
    pub fn is_fresh(&self, nonce: &str) -> bool {
        let mut seen = self.seen_nonces.write();
        
        if seen.contains(nonce) {
            warn!("Replay detected: nonce {} already seen", nonce);
            return false;
        }
        
        seen.insert(nonce.to_string());
        debug!("Nonce {} marked as seen", nonce);
        true
    }
    
    /// Cleanup old nonces (called periodically)
    pub fn cleanup(&self) {
        // For now, we keep all nonces for the TTL period
        // In production, would track timestamps and remove expired ones
        let mut seen = self.seen_nonces.write();
        if seen.len() > 10000 {
            // Prevent unbounded growth - clear if too large
            warn!("Nonce tracker too large ({} entries), clearing", seen.len());
            seen.clear();
        }
    }
    
    /// Get count of tracked nonces
    pub fn count(&self) -> usize {
        self.seen_nonces.read().len()
    }
}

