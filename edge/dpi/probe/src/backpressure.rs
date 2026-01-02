// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/backpressure.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Backpressure handling - DROP + SIGNAL, never block

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, warn, info};

use super::errors::ProbeError;

/// Backpressure manager
/// 
/// Under pressure: DROP packets + SIGNAL (never block).
/// Lock-free statistics.
pub struct BackpressureManager {
    max_queue_size: usize,
    current_queue_size: Arc<AtomicU64>,
    packets_dropped: Arc<AtomicU64>,
    backpressure_active: Arc<AtomicBool>,
    drop_threshold: usize,
}

impl BackpressureManager {
    /// Create new backpressure manager
    pub fn new(max_queue_size: usize) -> Self {
        let drop_threshold = (max_queue_size as f64 * 0.8) as usize; // Start dropping at 80%
        
        Self {
            max_queue_size,
            current_queue_size: Arc::new(AtomicU64::new(0)),
            packets_dropped: Arc::new(AtomicU64::new(0)),
            backpressure_active: Arc::new(AtomicBool::new(false)),
            drop_threshold,
        }
    }
    
    /// Check if packet should be dropped
    /// 
    /// Returns true if packet should be dropped (backpressure active).
    /// Never blocks - always returns immediately.
    pub fn should_drop(&self, current_size: usize) -> bool {
        if current_size >= self.drop_threshold {
            if !self.backpressure_active.load(Ordering::Acquire) {
                self.backpressure_active.store(true, Ordering::Release);
                warn!("Backpressure activated: queue size {} >= threshold {}", 
                    current_size, self.drop_threshold);
            }
            
            // Increment drop counter
            self.packets_dropped.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        
        // Check if we can deactivate backpressure
        if current_size < (self.drop_threshold as f64 * 0.5) as usize {
            if self.backpressure_active.load(Ordering::Acquire) {
                self.backpressure_active.store(false, Ordering::Release);
                info!("Backpressure deactivated: queue size {} < threshold {}", 
                    current_size, self.drop_threshold / 2);
            }
        }
        
        false
    }
    
    /// Signal backpressure (non-blocking)
    pub fn signal(&self) {
        if self.backpressure_active.load(Ordering::Acquire) {
            // Signal to monitoring/alerting system
            // In production, would emit metrics or send signal
            error!("Backpressure active: {} packets dropped, queue size: {}", 
                self.packets_dropped.load(Ordering::Relaxed),
                self.current_queue_size.load(Ordering::Relaxed));
        }
    }
    
    /// Get statistics (lock-free)
    pub fn stats(&self) -> BackpressureStats {
        BackpressureStats {
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            backpressure_active: self.backpressure_active.load(Ordering::Relaxed),
            current_queue_size: self.current_queue_size.load(Ordering::Relaxed) as usize,
            drop_threshold: self.drop_threshold,
        }
    }
    
    /// Update queue size (for monitoring)
    pub fn update_queue_size(&self, size: usize) {
        self.current_queue_size.store(size as u64, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct BackpressureStats {
    pub packets_dropped: u64,
    pub backpressure_active: bool,
    pub current_queue_size: usize,
    pub drop_threshold: usize,
}

