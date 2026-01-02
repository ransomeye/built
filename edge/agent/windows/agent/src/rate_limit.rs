// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/rate_limit.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rate limiting for event emission

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use parking_lot::Mutex;
use tracing::{error, warn, info, debug};

use super::errors::AgentError;

/// Rate limiter
/// 
/// Enforces rate limits on event emission to prevent overwhelming the system.
pub struct RateLimiter {
    max_events_per_second: u64,
    window_start: Arc<Mutex<u64>>,
    events_in_window: Arc<AtomicU64>,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(max_events_per_second: u64) -> Self {
        Self {
            max_events_per_second,
            window_start: Arc::new(Mutex::new(0)),
            events_in_window: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Check if event can be emitted (rate limit check)
    pub fn check_rate_limit(&self) -> Result<bool, AgentError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::RateLimitExceeded(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let mut window_start = self.window_start.lock();
        
        // Reset window if 1 second has passed
        if now > *window_start + 1 {
            *window_start = now;
            self.events_in_window.store(0, Ordering::Release);
        }
        
        let current_count = self.events_in_window.load(Ordering::Acquire);
        
        if current_count >= self.max_events_per_second {
            debug!("Rate limit exceeded: {}/{} events", current_count, self.max_events_per_second);
            return Ok(false);
        }
        
        self.events_in_window.fetch_add(1, Ordering::AcqRel);
        Ok(true)
    }
    
    /// Get current events in window
    pub fn current_count(&self) -> u64 {
        self.events_in_window.load(Ordering::Acquire)
    }
    
    /// Get max events per second
    pub fn max_events_per_second(&self) -> u64 {
        self.max_events_per_second
    }
}

