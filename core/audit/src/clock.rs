// Path and File Name : /home/ransomeye/rebuild/core/audit/src/clock.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Clock guard - ensures monotonic timestamps and detects clock rollback

use chrono::{DateTime, Utc};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, warn};

/// Clock guard - ensures monotonic timestamps
pub struct ClockGuard {
    last_timestamp: Arc<RwLock<Option<DateTime<Utc>>>>,
    rollback_count: Arc<RwLock<u64>>,
}

impl ClockGuard {
    /// Create new clock guard
    pub fn new() -> Self {
        Self {
            last_timestamp: Arc::new(RwLock::new(None)),
            rollback_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Get current timestamp with rollback protection
    /// 
    /// FAIL-CLOSED: Returns error if clock rollback detected
    pub fn get_timestamp(&self) -> Result<DateTime<Utc>, String> {
        let now = Utc::now();
        
        let mut last = self.last_timestamp.write();
        
        if let Some(last_ts) = *last {
            if now < last_ts {
                // Clock rollback detected!
                let rollback_duration = last_ts - now;
                *self.rollback_count.write() += 1;
                
                error!("CLOCK ROLLBACK DETECTED: Current time {} is {} seconds before last timestamp {}", 
                       now, rollback_duration.num_seconds(), last_ts);
                
                // FAIL-CLOSED: Return error
                return Err(format!("Clock rollback detected: current time {} is before last timestamp {}", 
                                  now, last_ts));
            }
        }
        
        *last = Some(now);
        Ok(now)
    }
    
    /// Get rollback count
    pub fn get_rollback_count(&self) -> u64 {
        *self.rollback_count.read()
    }
    
    /// Check if clock is healthy
    pub fn is_healthy(&self) -> bool {
        self.get_rollback_count() == 0
    }
}

impl Default for ClockGuard {
    fn default() -> Self {
        Self::new()
    }
}

