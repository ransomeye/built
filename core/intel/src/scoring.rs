// Path and File Name : /home/ransomeye/rebuild/core/intel/src/scoring.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Intel scoring - final scoring and escalation decisions

use crate::confidence::{ConfidenceScorer, ConfidenceLevel};
use crate::correlation::CorrelationResult;
use crate::errors::IntelError;

/// Intel scorer - determines escalation based on confidence
pub struct IntelScorer {
    scorer: ConfidenceScorer,
}

impl IntelScorer {
    /// Create new intel scorer
    pub fn new() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
        }
    }
    
    /// Score correlation result and determine escalation
    /// 
    /// Rules:
    /// - High confidence (>= 0.8): Can trigger policies
    /// - Medium confidence (>= 0.5): Requires corroboration
    /// - Low confidence (< 0.5): Logs only
    pub fn score(&self, correlation: &CorrelationResult) -> Result<EscalationDecision, IntelError> {
        let confidence_level = self.scorer.classify_confidence(correlation.confidence_score);
        
        let decision = match confidence_level {
            ConfidenceLevel::High => {
                EscalationDecision {
                    can_trigger_policy: true,
                    requires_corroboration: false,
                    log_only: false,
                    confidence_level,
                    reasoning: format!("High confidence ({:.2}) from {} signals across {} sources",
                                      correlation.confidence_score,
                                      correlation.signal_frequency,
                                      correlation.signals.len()),
                }
            },
            ConfidenceLevel::Medium => {
                EscalationDecision {
                    can_trigger_policy: false,
                    requires_corroboration: true,
                    log_only: false,
                    confidence_level,
                    reasoning: format!("Medium confidence ({:.2}) requires additional corroboration ({} signals)",
                                      correlation.confidence_score,
                                      correlation.signal_frequency),
                }
            },
            ConfidenceLevel::Low => {
                EscalationDecision {
                    can_trigger_policy: false,
                    requires_corroboration: false,
                    log_only: true,
                    confidence_level,
                    reasoning: format!("Low confidence ({:.2}) - logging only ({} signals)",
                                      correlation.confidence_score,
                                      correlation.signal_frequency),
                }
            },
        };
        
        Ok(decision)
    }
}

/// Escalation decision based on confidence
#[derive(Debug, Clone)]
pub struct EscalationDecision {
    pub can_trigger_policy: bool,
    pub requires_corroboration: bool,
    pub log_only: bool,
    pub confidence_level: ConfidenceLevel,
    pub reasoning: String,
}

impl Default for IntelScorer {
    fn default() -> Self {
        Self::new()
    }
}

