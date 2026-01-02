// Path and File Name : /home/ransomeye/rebuild/core/intel/src/policy_integration.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy integration - validates confidence before policy execution with fail-closed semantics

use tracing::{info, warn};

use crate::scoring::{IntelScorer, EscalationDecision};
use crate::correlation::CorrelationResult;
use crate::errors::IntelError;

/// Policy integration - validates intel confidence before policy execution
pub struct PolicyIntegration {
    scorer: IntelScorer,
}

impl PolicyIntegration {
    /// Create new policy integration
    pub fn new() -> Self {
        Self {
            scorer: IntelScorer::new(),
        }
    }
    
    /// Check if correlation result can trigger policy
    /// 
    /// FAIL-CLOSED: No policy execution without sufficient confidence
    /// - High confidence (>= 0.8): Can trigger policies
    /// - Medium confidence (>= 0.5): Requires corroboration (blocks policy)
    /// - Low confidence (< 0.5): Logs only (blocks policy)
    pub fn can_trigger_policy(&self, correlation: &CorrelationResult) -> Result<bool, IntelError> {
        let decision = self.scorer.score(correlation)?;
        
        if decision.log_only {
            warn!("Intel correlation {} has low confidence - logging only, no policy execution",
                  correlation.correlation_id);
            return Ok(false);
        }
        
        if decision.requires_corroboration {
            warn!("Intel correlation {} requires corroboration - blocking policy execution until additional signals",
                  correlation.correlation_id);
            return Ok(false);
        }
        
        if decision.can_trigger_policy {
            info!("Intel correlation {} has high confidence - can trigger policy execution",
                  correlation.correlation_id);
            return Ok(true);
        }
        
        // Default: block policy execution
        Ok(false)
    }
    
    /// Get escalation decision for correlation
    pub fn get_escalation_decision(&self, correlation: &CorrelationResult) -> Result<EscalationDecision, IntelError> {
        self.scorer.score(correlation)
    }
}

impl Default for PolicyIntegration {
    fn default() -> Self {
        Self::new()
    }
}

