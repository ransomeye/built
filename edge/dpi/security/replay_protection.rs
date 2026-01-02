// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/security/replay_protection.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay attack protection using nonce tracking and timestamp windows

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::{warn, debug};

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("Nonce already seen (replay attack): {0}")]
    DuplicateNonce(String),
    #[error("Timestamp out of window: {0}")]
    TimestampOutOfWindow(String),
    #[error("Invalid nonce format: {0}")]
    InvalidNonceFormat(String),
}

/// Replay protection manager
/// Tracks nonces and timestamps to prevent replay attacks
pub struct ReplayProtection {
    seen_nonces: Arc<Mutex<HashSet<String>>>,
    window_seconds: u64,
    max_nonces: usize,
}

impl ReplayProtection {
    pub fn new(window_seconds: u64, max_nonces: usize) -> Self {
        Self {
            seen_nonces: Arc::new(Mutex::new(HashSet::with_capacity(max_nonces))),
            window_seconds,
            max_nonces,
        }
    }
    
    /// Check if nonce has been seen before (replay attack detection)
    pub fn check_nonce(&self, nonce: &str, timestamp: &DateTime<Utc>) -> Result<bool, ReplayError> {
        // Validate nonce format (should be hex-encoded 32 bytes = 64 hex chars)
        if nonce.len() != 64 {
            return Err(ReplayError::InvalidNonceFormat(format!(
                "Invalid nonce length: {} (expected 64)", nonce.len()
            )));
        }
        
        // Validate hex format
        if !nonce.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ReplayError::InvalidNonceFormat("Nonce contains non-hex characters".to_string()));
        }
        
        // Check timestamp window
        let now = Utc::now();
        let event_time = timestamp.timestamp();
        let current_time = now.timestamp();
        
        let time_diff = (current_time - event_time).abs() as u64;
        if time_diff > self.window_seconds {
            warn!("Timestamp out of window: {} seconds (max: {})", time_diff, self.window_seconds);
            return Err(ReplayError::TimestampOutOfWindow(format!(
                "Time difference: {} seconds, max allowed: {}", time_diff, self.window_seconds
            )));
        }
        
        // Check if nonce already seen
        let mut nonces = self.seen_nonces.lock()
            .map_err(|e| ReplayError::DuplicateNonce(format!("Lock error: {}", e)))?;
        
        if nonces.contains(nonce) {
            warn!("Replay attack detected: nonce {} already seen", nonce);
            return Err(ReplayError::DuplicateNonce(format!("Nonce {} already seen", nonce)));
        }
        
        // Add nonce to set
        // Clean up old nonces if set is too large
        if nonces.len() >= self.max_nonces {
            // Remove oldest entries (simplified - in production use time-based eviction)
            let to_remove: Vec<String> = nonces.iter().take(self.max_nonces / 2).cloned().collect();
            for old_nonce in to_remove {
                nonces.remove(&old_nonce);
            }
        }
        
        nonces.insert(nonce.to_string());
        debug!("Nonce {} added to replay protection set (size: {})", nonce, nonces.len());
        
        Ok(true)
    }
    
    /// Clear expired nonces (called periodically)
    pub fn cleanup_expired(&self) {
        // In a production system, we'd track timestamps with nonces
        // For now, we rely on size-based eviction
        let mut nonces = match self.seen_nonces.lock() {
            Ok(n) => n,
            Err(_) => return,
        };
        
        if nonces.len() > self.max_nonces * 3 / 4 {
            // Remove oldest entries
            let to_remove: Vec<String> = nonces.iter().take(self.max_nonces / 4).cloned().collect();
            for old_nonce in to_remove {
                nonces.remove(&old_nonce);
            }
        }
    }
    
    /// Get current nonce set size (for monitoring)
    pub fn get_nonces_count(&self) -> usize {
        self.seen_nonces.lock()
            .map(|n| n.len())
            .unwrap_or(0)
    }
}
