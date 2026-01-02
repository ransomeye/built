// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/reentrancy.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Reentrancy and loop prevention - dispatcher MUST NOT trigger itself

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashSet;
use parking_lot::RwLock;
use tracing::{debug, error, warn};
use super::errors::DispatcherError;

pub struct ReentrancyGuard {
    /// Currently executing directive IDs
    executing: Arc<RwLock<HashSet<String>>>,
    
    /// Guard flag to prevent reentrancy
    in_execution: Arc<AtomicBool>,
}

impl ReentrancyGuard {
    pub fn new() -> Self {
        Self {
            executing: Arc::new(RwLock::new(HashSet::new())),
            in_execution: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Enter execution guard - prevents reentrancy
    pub fn enter(&self, directive_id: &str) -> Result<ReentrancyGuardToken, DispatcherError> {
        // Check if already in execution
        if self.in_execution.swap(true, Ordering::Acquire) {
            error!("Reentrancy detected: dispatcher already executing");
            return Err(DispatcherError::ReentrancyDetected);
        }
        
        // Check if this directive is already executing
        {
            let mut executing = self.executing.write();
            if executing.contains(directive_id) {
                error!("Loop detected: directive {} already executing", directive_id);
                self.in_execution.store(false, Ordering::Release);
                return Err(DispatcherError::LoopDetected);
            }
            executing.insert(directive_id.to_string());
        }
        
        debug!("Entered execution guard for directive {}", directive_id);
        Ok(ReentrancyGuardToken {
            directive_id: directive_id.to_string(),
            guard: Arc::clone(&self.executing),
            in_execution: Arc::clone(&self.in_execution),
        })
    }
}

#[derive(Debug)]
pub struct ReentrancyGuardToken {
    directive_id: String,
    guard: Arc<RwLock<HashSet<String>>>,
    in_execution: Arc<AtomicBool>,
}

impl Drop for ReentrancyGuardToken {
    fn drop(&mut self) {
        {
            let mut executing = self.guard.write();
            executing.remove(&self.directive_id);
        }
        self.in_execution.store(false, Ordering::Release);
        debug!("Exited execution guard for directive {}", self.directive_id);
    }
}
