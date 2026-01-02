// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/backpressure.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Backpressure handling for event processing

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use parking_lot::Mutex;
use tracing::{error, warn, info, debug};

use super::errors::AgentError;

/// Backpressure handler
/// 
/// Monitors buffer size and applies backpressure when thresholds are exceeded.
pub struct BackpressureHandler {
    max_buffer_size: u64,
    threshold: f64,
    current_size: Arc<AtomicU64>,
    backpressure_active: Arc<AtomicBool>,
    dropped_events: Arc<AtomicU64>,
}

impl BackpressureHandler {
    /// Create new backpressure handler
    pub fn new(max_buffer_size: u64, threshold: f64) -> Self {
        Self {
            max_buffer_size,
            threshold,
            current_size: Arc::new(AtomicU64::new(0)),
            backpressure_active: Arc::new(AtomicBool::new(false)),
            dropped_events: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Check if backpressure should be applied
    pub fn should_apply_backpressure(&self) -> bool {
        let current = self.current_size.load(Ordering::Acquire);
        let threshold_size = (self.max_buffer_size as f64 * self.threshold) as u64;
        
        if current >= threshold_size {
            if !self.backpressure_active.load(Ordering::Acquire) {
                warn!("Backpressure activated: current={}, threshold={}", current, threshold_size);
                self.backpressure_active.store(true, Ordering::Release);
            }
            true
        } else {
            if self.backpressure_active.load(Ordering::Acquire) {
                info!("Backpressure deactivated: current={}, threshold={}", current, threshold_size);
                self.backpressure_active.store(false, Ordering::Release);
            }
            false
        }
    }
    
    /// Add to buffer size
    pub fn add_size(&self, size: u64) {
        self.current_size.fetch_add(size, Ordering::AcqRel);
    }
    
    /// Remove from buffer size
    pub fn remove_size(&self, size: u64) {
        self.current_size.fetch_sub(size, Ordering::AcqRel);
    }
    
    /// Drop event due to backpressure
    pub fn drop_event(&self) {
        self.dropped_events.fetch_add(1, Ordering::AcqRel);
        debug!("Event dropped due to backpressure");
    }
    
    /// Get current buffer size
    pub fn current_size(&self) -> u64 {
        self.current_size.load(Ordering::Acquire)
    }
    
    /// Get dropped events count
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::Acquire)
    }
    
    /// Check if backpressure is active
    pub fn is_active(&self) -> bool {
        self.backpressure_active.load(Ordering::Acquire)
    }
}

