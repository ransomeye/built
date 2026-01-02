// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/invariants.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Hard invariant enforcement - fail-closed on violation with audit logging

use crate::errors::CorrelationError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Invariant violation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvariantViolation {
    /// Stage skip without evidence
    StageSkipWithoutEvidence {
        from_stage: String,
        to_stage: String,
        entity_id: String,
    },
    /// Confidence increase without new signal
    ConfidenceIncreaseWithoutSignal {
        entity_id: String,
        old_confidence: f64,
        new_confidence: f64,
    },
    /// Detection without minimum signal set
    DetectionWithoutMinimumSignals {
        entity_id: String,
        required_signals: HashSet<String>,
        actual_signals: HashSet<String>,
    },
    /// State explosion without eviction
    StateExplosionWithoutEviction {
        entity_id: String,
        state_size: usize,
        max_allowed: usize,
    },
}

/// Invariant violation audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantViolationLog {
    pub timestamp: chrono::DateTime<Utc>,
    pub violation: InvariantViolation,
    pub entity_id: String,
    pub action_taken: String,
}

/// Invariant enforcer
pub struct InvariantEnforcer {
    /// Maximum state size per entity
    max_state_size: usize,
    /// Minimum signal set required for detection
    min_signal_set: HashSet<String>,
    /// Violation log
    violation_log: Vec<InvariantViolationLog>,
}

impl InvariantEnforcer {
    /// Create new invariant enforcer
    pub fn new(max_state_size: usize, min_signal_set: HashSet<String>) -> Self {
        Self {
            max_state_size,
            min_signal_set,
            violation_log: Vec::new(),
        }
    }

    /// Enforce: No stage skip without evidence
    pub fn enforce_no_stage_skip_without_evidence(
        &mut self,
        from_stage: Option<&str>,
        to_stage: &str,
        entity_id: &str,
        has_evidence: bool,
    ) -> Result<(), CorrelationError> {
        if !has_evidence {
            let violation = InvariantViolation::StageSkipWithoutEvidence {
                from_stage: from_stage.unwrap_or("None").to_string(),
                to_stage: to_stage.to_string(),
                entity_id: entity_id.to_string(),
            };

            self.log_violation(violation, entity_id, "ABORT_CORRELATION");
            return Err(CorrelationError::InvariantViolation(
                "Stage skip without evidence".to_string(),
            ));
        }

        Ok(())
    }

    /// Enforce: No confidence increase without new signal
    pub fn enforce_no_confidence_increase_without_signal(
        &mut self,
        entity_id: &str,
        old_confidence: f64,
        new_confidence: f64,
        has_new_signal: bool,
    ) -> Result<(), CorrelationError> {
        if new_confidence > old_confidence && !has_new_signal {
            let violation = InvariantViolation::ConfidenceIncreaseWithoutSignal {
                entity_id: entity_id.to_string(),
                old_confidence,
                new_confidence,
            };

            self.log_violation(violation, entity_id, "REJECT_CONFIDENCE_UPDATE");
            return Err(CorrelationError::InvariantViolation(
                "Confidence increase without new signal".to_string(),
            ));
        }

        Ok(())
    }

    /// Enforce: No detection without minimum signal set
    pub fn enforce_no_detection_without_minimum_signals(
        &mut self,
        entity_id: &str,
        actual_signals: &HashSet<String>,
    ) -> Result<(), CorrelationError> {
        let missing: Vec<_> = self
            .min_signal_set
            .difference(actual_signals)
            .cloned()
            .collect();

        if !missing.is_empty() {
            let violation = InvariantViolation::DetectionWithoutMinimumSignals {
                entity_id: entity_id.to_string(),
                required_signals: self.min_signal_set.clone(),
                actual_signals: actual_signals.clone(),
            };

            self.log_violation(violation, entity_id, "REJECT_DETECTION");
            return Err(CorrelationError::InvariantViolation(format!(
                "Detection without minimum signal set. Missing: {:?}",
                missing
            )));
        }

        Ok(())
    }

    /// Enforce: No state explosion without eviction
    pub fn enforce_no_state_explosion_without_eviction(
        &mut self,
        entity_id: &str,
        state_size: usize,
        eviction_triggered: bool,
    ) -> Result<(), CorrelationError> {
        if state_size > self.max_state_size && !eviction_triggered {
            let violation = InvariantViolation::StateExplosionWithoutEviction {
                entity_id: entity_id.to_string(),
                state_size,
                max_allowed: self.max_state_size,
            };

            self.log_violation(violation, entity_id, "ABORT_CORRELATION");
            return Err(CorrelationError::InvariantViolation(format!(
                "State explosion without eviction. Size: {}, Max: {}",
                state_size, self.max_state_size
            )));
        }

        Ok(())
    }

    /// Log violation
    fn log_violation(
        &mut self,
        violation: InvariantViolation,
        entity_id: &str,
        action: &str,
    ) {
        let log_entry = InvariantViolationLog {
            timestamp: Utc::now(),
            violation: violation.clone(),
            entity_id: entity_id.to_string(),
            action_taken: action.to_string(),
        };

        self.violation_log.push(log_entry);
    }

    /// Get violation log
    pub fn get_violation_log(&self) -> &[InvariantViolationLog] {
        &self.violation_log
    }

    /// Clear violation log (use with caution)
    pub fn clear_violation_log(&mut self) {
        self.violation_log.clear();
    }
}

impl Default for InvariantEnforcer {
    fn default() -> Self {
        Self::new(
            1000, // Default max state size
            HashSet::new(), // Empty min signal set by default
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_skip_invariant() {
        let mut enforcer = InvariantEnforcer::default();
        
        // Should fail if no evidence
        let result = enforcer.enforce_no_stage_skip_without_evidence(
            Some("InitialAccess"),
            "EncryptionExecution",
            "entity1",
            false,
        );
        assert!(result.is_err());
        
        // Should pass with evidence
        let result = enforcer.enforce_no_stage_skip_without_evidence(
            Some("InitialAccess"),
            "Execution",
            "entity1",
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_confidence_increase_invariant() {
        let mut enforcer = InvariantEnforcer::default();
        
        // Should fail if confidence increases without new signal
        let result = enforcer.enforce_no_confidence_increase_without_signal(
            "entity1",
            0.5,
            0.7,
            false,
        );
        assert!(result.is_err());
        
        // Should pass with new signal
        let result = enforcer.enforce_no_confidence_increase_without_signal(
            "entity1",
            0.5,
            0.7,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_explosion_invariant() {
        let mut enforcer = InvariantEnforcer::new(100, HashSet::new());
        
        // Should fail if state exceeds limit without eviction
        let result = enforcer.enforce_no_state_explosion_without_eviction(
            "entity1",
            150,
            false,
        );
        assert!(result.is_err());
        
        // Should pass if eviction triggered
        let result = enforcer.enforce_no_state_explosion_without_eviction(
            "entity1",
            150,
            true,
        );
        assert!(result.is_ok());
    }
}

