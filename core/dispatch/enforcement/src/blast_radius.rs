// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/blast_radius.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Blast radius limiting - prevents mass enforcement across assets

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{warn, debug};
use crate::errors::EnforcementError;

#[derive(Debug, Clone)]
struct BlastRadiusWindow {
    affected_hosts: Vec<String>,
    window_start: Instant,
}

pub struct BlastRadiusLimiter {
    windows: Arc<RwLock<HashMap<String, BlastRadiusWindow>>>,
    max_hosts_per_window: usize,
    window_duration_seconds: u64,
}

impl BlastRadiusLimiter {
    pub fn new(max_hosts_per_window: usize, window_duration_seconds: u64) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            max_hosts_per_window,
            window_duration_seconds,
        }
    }
    
    /// Check blast radius limit for targets
    pub fn check(&self, key: &str, targets: &[String]) -> Result<String, EnforcementError> {
        let mut windows_map = self.windows.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let now = Instant::now();
        let window = windows_map.entry(key.to_string())
            .or_insert_with(|| BlastRadiusWindow {
                affected_hosts: Vec::new(),
                window_start: now,
            });
        
        // Reset window if expired
        if now.duration_since(window.window_start) > Duration::from_secs(self.window_duration_seconds) {
            window.affected_hosts.clear();
            window.window_start = now;
        }
        
        // Count new hosts
        let mut new_hosts = 0;
        for target in targets {
            if !window.affected_hosts.contains(target) {
                new_hosts += 1;
            }
        }
        
        // Check if adding new hosts would exceed limit
        let total_after = window.affected_hosts.len() + new_hosts;
        if total_after > self.max_hosts_per_window {
            return Err(EnforcementError::BlastRadiusExceeded(
                format!("Blast radius limit exceeded. Would affect {} hosts, limit is {}. Current window has {} hosts.", 
                    total_after, self.max_hosts_per_window, window.affected_hosts.len())
            ));
        }
        
        // Add new hosts
        for target in targets {
            if !window.affected_hosts.contains(target) {
                window.affected_hosts.push(target.clone());
            }
        }
        
        let status = format!("Blast radius: {} hosts affected in window (limit: {}, resets in {:?})", 
            window.affected_hosts.len(),
            self.max_hosts_per_window,
            Duration::from_secs(self.window_duration_seconds)
                .saturating_sub(now.duration_since(window.window_start)));
        
        debug!("Blast radius check passed for key '{}': {}", key, status);
        Ok(status)
    }
    
    /// Reset blast radius window for a key
    pub fn reset(&self, key: &str) -> Result<(), EnforcementError> {
        let mut windows_map = self.windows.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        windows_map.remove(key);
        debug!("Blast radius reset for key '{}'", key);
        Ok(())
    }
}

