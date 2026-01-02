// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/safety.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Safety guards - allowlist/denylist, rate limits

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use super::directive_envelope::DirectiveEnvelope;
use super::errors::DispatcherError;

/// Explicit allowlist of allowed actions
const ALLOWED_ACTIONS: &[&str] = &[
    "block",
    "isolate",
    "quarantine",
    "monitor",
    "allow",
    "deny",
];

/// Explicit denylist of forbidden actions
const DENIED_ACTIONS: &[&str] = &[
    "delete",
    "format",
    "shutdown",
    "reboot",
];

pub struct SafetyGuards {
    /// Per-action rate limits
    action_rate_limits: Arc<RwLock<HashMap<String, Vec<u64>>>>, // action -> timestamps
    
    /// Per-entity rate limits
    entity_rate_limits: Arc<RwLock<HashMap<String, Vec<u64>>>>, // entity -> timestamps
    
    /// Global execution ceiling
    global_executions: Arc<RwLock<Vec<u64>>>, // timestamps
    
    /// Rate limit configuration
    max_actions_per_window: usize,
    window_seconds: u64,
    max_global_per_window: usize,
}

impl SafetyGuards {
    pub fn new(max_actions_per_window: usize, window_seconds: u64, max_global_per_window: usize) -> Self {
        Self {
            action_rate_limits: Arc::new(RwLock::new(HashMap::new())),
            entity_rate_limits: Arc::new(RwLock::new(HashMap::new())),
            global_executions: Arc::new(RwLock::new(Vec::new())),
            max_actions_per_window,
            window_seconds,
            max_global_per_window,
        }
    }
    
    /// Check if action is allowed (allowlist)
    pub fn is_action_allowed(&self, action: &str) -> bool {
        ALLOWED_ACTIONS.contains(&action)
    }
    
    /// Check if action is denied (denylist)
    pub fn is_action_denied(&self, action: &str) -> bool {
        DENIED_ACTIONS.contains(&action)
    }
    
    /// Check safety guards for directive
    pub fn check(&self, directive: &DirectiveEnvelope) -> Result<(), DispatcherError> {
        // Check allowlist
        if !self.is_action_allowed(&directive.action) {
            return Err(DispatcherError::InvalidDirective(
                format!("Action '{}' is not in allowlist", directive.action)
            ));
        }
        
        // Check denylist
        if self.is_action_denied(&directive.action) {
            return Err(DispatcherError::InvalidDirective(
                format!("Action '{}' is in denylist", directive.action)
            ));
        }
        
        // Check per-action rate limit
        self.check_action_rate_limit(&directive.action)?;
        
        // Check per-entity rate limit (using policy_id as entity)
        self.check_entity_rate_limit(&directive.policy_id)?;
        
        // Check global execution ceiling
        self.check_global_limit()?;
        
        debug!("Safety guards passed for directive {}", directive.directive_id);
        Ok(())
    }
    
    fn check_action_rate_limit(&self, action: &str) -> Result<(), DispatcherError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now.saturating_sub(self.window_seconds);
        
        let mut limits = self.action_rate_limits.write();
        let timestamps = limits.entry(action.to_string()).or_insert_with(Vec::new);
        
        // Remove old timestamps
        timestamps.retain(|&t| t > window_start);
        
        if timestamps.len() >= self.max_actions_per_window {
            return Err(DispatcherError::InvalidDirective(
                format!("Action '{}' rate limit exceeded: {} actions in {} seconds", 
                    action, timestamps.len(), self.window_seconds)
            ));
        }
        
        timestamps.push(now);
        Ok(())
    }
    
    fn check_entity_rate_limit(&self, entity: &str) -> Result<(), DispatcherError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now.saturating_sub(self.window_seconds);
        
        let mut limits = self.entity_rate_limits.write();
        let timestamps = limits.entry(entity.to_string()).or_insert_with(Vec::new);
        
        // Remove old timestamps
        timestamps.retain(|&t| t > window_start);
        
        if timestamps.len() >= self.max_actions_per_window {
            return Err(DispatcherError::InvalidDirective(
                format!("Entity '{}' rate limit exceeded: {} actions in {} seconds", 
                    entity, timestamps.len(), self.window_seconds)
            ));
        }
        
        timestamps.push(now);
        Ok(())
    }
    
    fn check_global_limit(&self) -> Result<(), DispatcherError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now.saturating_sub(self.window_seconds);
        
        let mut global = self.global_executions.write();
        
        // Remove old timestamps
        global.retain(|&t| t > window_start);
        
        if global.len() >= self.max_global_per_window {
            return Err(DispatcherError::InvalidDirective(
                format!("Global execution ceiling exceeded: {} executions in {} seconds", 
                    global.len(), self.window_seconds)
            ));
        }
        
        global.push(now);
        Ok(())
    }
}
