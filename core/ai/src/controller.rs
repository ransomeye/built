// Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/src/controller.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: AI subsystem controller - manages AI state and fail-closed behavior

use std::sync::{Arc, RwLock};
use tracing::{error, warn, info, debug};
use crate::errors::AdvisoryError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIState {
    Enabled,
    Disabled,
    Degraded,
}

pub struct AIController {
    state: Arc<RwLock<AIState>>,
    disable_reason: Arc<RwLock<Option<String>>>,
}

impl AIController {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AIState::Enabled)),
            disable_reason: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Check if AI is enabled
    pub fn is_enabled(&self) -> Result<bool, AdvisoryError> {
        let state = self.state.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        Ok(*state == AIState::Enabled)
    }
    
    /// Disable AI subsystem
    pub fn disable(&self, reason: &str) -> Result<(), AdvisoryError> {
        let mut state = self.state.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        let mut disable_reason = self.disable_reason.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        *state = AIState::Disabled;
        *disable_reason = Some(reason.to_string());
        
        error!("AI subsystem DISABLED: {}", reason);
        Ok(())
    }
    
    /// Set AI to degraded state
    pub fn set_degraded(&self, reason: &str) -> Result<(), AdvisoryError> {
        let mut state = self.state.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        *state = AIState::Degraded;
        warn!("AI subsystem DEGRADED: {}", reason);
        Ok(())
    }
    
    /// Re-enable AI subsystem
    pub fn enable(&self) -> Result<(), AdvisoryError> {
        let mut state = self.state.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        let mut disable_reason = self.disable_reason.write()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        *state = AIState::Enabled;
        *disable_reason = None;
        
        info!("AI subsystem ENABLED");
        Ok(())
    }
    
    /// Get current state
    pub fn get_state(&self) -> Result<AIState, AdvisoryError> {
        let state = self.state.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        Ok(state.clone())
    }
    
    /// Get disable reason
    pub fn get_disable_reason(&self) -> Result<Option<String>, AdvisoryError> {
        let reason = self.disable_reason.read()
            .map_err(|e| AdvisoryError::InternalError(format!("Lock poisoned: {}", e)))?;
        Ok(reason.clone())
    }
    
    /// Require AI to be enabled
    pub fn require_enabled(&self) -> Result<(), AdvisoryError> {
        if !self.is_enabled()? {
            let reason = self.get_disable_reason()?
                .unwrap_or_else(|| "Unknown reason".to_string());
            return Err(AdvisoryError::AIDisabled(reason));
        }
        Ok(())
    }
}

