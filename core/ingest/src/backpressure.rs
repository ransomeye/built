// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/backpressure.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Explicit backpressure control - signals producers when system is overloaded

/*
 * Backpressure Controller
 * 
 * Manages explicit backpressure signals to producers.
 * Signals backpressure when buffers are full or system is overloaded.
 * Never drops events silently.
 */

use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use tracing::{warn, debug};

use crate::config::Config;

pub struct BackpressureController {
    config: Config,
    backpressure_states: Arc<DashMap<String, BackpressureState>>,
    global_backpressure: Arc<RwLock<bool>>,
}

struct BackpressureState {
    active: bool,
    signaled_at: Instant,
    reason: String,
}

impl BackpressureController {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            backpressure_states: Arc::new(DashMap::new()),
            global_backpressure: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn can_accept(&self, producer_id: &str) -> bool {
        // Check global backpressure
        if *self.global_backpressure.read() {
            return false;
        }
        
        // Check producer-specific backpressure
        if let Some(state) = self.backpressure_states.get(producer_id) {
            if state.active {
                // Check if backpressure should be cleared
                let elapsed = Instant::now().duration_since(state.signaled_at);
                if elapsed >= Duration::from_secs(self.config.backpressure_clear_seconds) {
                    // Clear backpressure
                    drop(state);
                    self.backpressure_states.remove(producer_id);
                    debug!("Backpressure cleared for producer: {}", producer_id);
                    return true;
                }
                return false;
            }
        }
        
        true
    }
    
    pub async fn signal_backpressure(&self, producer_id: &str) {
        let state = BackpressureState {
            active: true,
            signaled_at: Instant::now(),
            reason: "Buffer full or system overloaded".to_string(),
        };
        
        self.backpressure_states.insert(producer_id.to_string(), state);
        warn!("Backpressure signaled for producer: {}", producer_id);
    }
    
    pub async fn clear_backpressure(&self, producer_id: &str) {
        self.backpressure_states.remove(producer_id);
        debug!("Backpressure cleared for producer: {}", producer_id);
    }
    
    pub async fn set_global_backpressure(&self, active: bool) {
        *self.global_backpressure.write() = active;
        if active {
            warn!("Global backpressure activated");
        } else {
            debug!("Global backpressure cleared");
        }
    }
}

