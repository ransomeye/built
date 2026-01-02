// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/backpressure.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Backpressure handler - manages bounded buffers and backpressure signals from Core

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{warn, debug};

pub struct BackpressureHandler {
    buffer_size: Arc<AtomicUsize>,
    max_buffer_size: usize,
    threshold: usize,
    backpressure_active: Arc<RwLock<bool>>,
    dropped_count: Arc<AtomicUsize>,
}

impl BackpressureHandler {
    pub fn new(max_buffer_size: usize, threshold: usize) -> Self {
        Self {
            buffer_size: Arc::new(AtomicUsize::new(0)),
            max_buffer_size,
            threshold,
            backpressure_active: Arc::new(RwLock::new(false)),
            dropped_count: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn can_accept(&self) -> bool {
        let current = self.buffer_size.load(Ordering::Relaxed);
        current < self.max_buffer_size
    }
    
    pub fn should_backpressure(&self) -> bool {
        let current = self.buffer_size.load(Ordering::Relaxed);
        current >= self.threshold
    }
    
    pub fn increment_buffer(&self, size: usize) -> bool {
        let current = self.buffer_size.fetch_add(size, Ordering::Relaxed);
        if current + size >= self.max_buffer_size {
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
            warn!("Buffer full, dropping event. Current: {}, Max: {}", current + size, self.max_buffer_size);
            false
        } else {
            true
        }
    }
    
    pub fn decrement_buffer(&self, size: usize) {
        self.buffer_size.fetch_sub(size, Ordering::Relaxed);
    }
    
    pub fn set_backpressure(&self, active: bool) {
        *self.backpressure_active.write() = active;
        if active {
            debug!("Backpressure signal received from Core");
        } else {
            debug!("Backpressure cleared");
        }
    }
    
    pub fn is_backpressure_active(&self) -> bool {
        *self.backpressure_active.read()
    }
    
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size.load(Ordering::Relaxed)
    }
    
    pub fn get_dropped_count(&self) -> usize {
        self.dropped_count.load(Ordering::Relaxed)
    }
    
    pub fn reset_stats(&self) {
        self.dropped_count.store(0, Ordering::Relaxed);
    }
}

