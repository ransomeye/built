// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/rate_limit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rate limiting for enforcement actions - prevents mass execution

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::Utc;
use tracing::{warn, debug};
use crate::errors::EnforcementError;

#[derive(Debug, Clone)]
struct RateLimitWindow {
    count: usize,
    window_start: Instant,
}

pub struct RateLimiter {
    windows: Arc<RwLock<HashMap<String, RateLimitWindow>>>,
    max_actions_per_window: usize,
    window_duration_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_actions_per_window: usize, window_duration_seconds: u64) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            max_actions_per_window,
            window_duration_seconds,
        }
    }
    
    /// Check if action is allowed under rate limit
    pub fn check(&self, key: &str) -> Result<String, EnforcementError> {
        let mut windows_map = self.windows.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let now = Instant::now();
        let window = windows_map.entry(key.to_string())
            .or_insert_with(|| RateLimitWindow {
                count: 0,
                window_start: now,
            });
        
        // Reset window if expired
        if now.duration_since(window.window_start) > Duration::from_secs(self.window_duration_seconds) {
            window.count = 0;
            window.window_start = now;
        }
        
        // Check limit
        if window.count >= self.max_actions_per_window {
            let remaining = Duration::from_secs(self.window_duration_seconds)
                .saturating_sub(now.duration_since(window.window_start));
            return Err(EnforcementError::RateLimitExceeded(
                format!("Rate limit exceeded for key '{}'. {} actions in window. Retry after {:?}", 
                    key, window.count, remaining)
            ));
        }
        
        // Increment count
        window.count += 1;
        
        let status = format!("Action {} of {} allowed in window (resets in {:?})", 
            window.count, 
            self.max_actions_per_window,
            Duration::from_secs(self.window_duration_seconds)
                .saturating_sub(now.duration_since(window.window_start)));
        
        debug!("Rate limit check passed for key '{}': {}", key, status);
        Ok(status)
    }
    
    /// Reset rate limit for a key
    pub fn reset(&self, key: &str) -> Result<(), EnforcementError> {
        let mut windows_map = self.windows.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        windows_map.remove(key);
        debug!("Rate limit reset for key '{}'", key);
        Ok(())
    }
}

