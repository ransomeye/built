// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/scoring.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Deterministic confidence scoring for kill-chain progression

use crate::kill_chain::stages::RansomwareStage;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Signal contribution to confidence
#[derive(Debug, Clone)]
pub struct SignalContribution {
    pub signal_type: String,
    pub base_confidence: f64,
    pub temporal_decay: f64,
    pub timestamp: DateTime<Utc>,
}

/// Confidence scorer
pub struct ConfidenceScorer {
    /// Base confidence weights by signal type
    signal_weights: HashMap<String, f64>,
    /// Stage progression multipliers
    stage_multipliers: HashMap<RansomwareStage, f64>,
    /// Temporal decay rate per hour
    temporal_decay_per_hour: f64,
}

impl ConfidenceScorer {
    /// Create new scorer with default weights
    pub fn new() -> Self {
        let mut signal_weights = HashMap::new();
        signal_weights.insert("network_connection".to_string(), 0.3);
        signal_weights.insert("process_creation".to_string(), 0.4);
        signal_weights.insert("file_modification".to_string(), 0.5);
        signal_weights.insert("encryption_activity".to_string(), 0.8);
        signal_weights.insert("ransom_note".to_string(), 0.9);
        signal_weights.insert("credential_dump".to_string(), 0.6);
        signal_weights.insert("lateral_movement".to_string(), 0.5);

        let mut stage_multipliers = HashMap::new();
        for stage in RansomwareStage::all_stages() {
            let multiplier = match stage {
                RansomwareStage::InitialAccess => 0.3,
                RansomwareStage::Execution => 0.4,
                RansomwareStage::Persistence => 0.5,
                RansomwareStage::PrivilegeEscalation => 0.6,
                RansomwareStage::LateralMovement => 0.6,
                RansomwareStage::CredentialAccess => 0.7,
                RansomwareStage::Discovery => 0.7,
                RansomwareStage::EncryptionPreparation => 0.8,
                RansomwareStage::EncryptionExecution => 0.9,
                RansomwareStage::Impact => 1.0,
            };
            stage_multipliers.insert(stage, multiplier);
        }

        Self {
            signal_weights,
            stage_multipliers,
            temporal_decay_per_hour: 0.1,
        }
    }

    /// Calculate confidence from signals
    pub fn calculate_confidence(
        &self,
        signals: &[SignalContribution],
        current_stage: Option<RansomwareStage>,
        now: DateTime<Utc>,
    ) -> f64 {
        if signals.is_empty() {
            return 0.0;
        }

        // Calculate base confidence from signals
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for signal in signals {
            let base_weight = self
                .signal_weights
                .get(&signal.signal_type)
                .copied()
                .unwrap_or(0.2);

            // Apply temporal decay
            let hours_old = (now - signal.timestamp)
                .num_seconds()
                .max(0) as f64
                / 3600.0;
            let decay_factor = (1.0 - self.temporal_decay_per_hour).powf(hours_old);
            let adjusted_weight = base_weight * decay_factor * signal.base_confidence;

            weighted_sum += adjusted_weight;
            total_weight += base_weight;
        }

        let base_confidence = if total_weight > 0.0 {
            (weighted_sum / total_weight).min(1.0).max(0.0)
        } else {
            0.0
        };

        // Apply stage multiplier
        let stage_multiplier = current_stage
            .and_then(|s| self.stage_multipliers.get(&s).copied())
            .unwrap_or(1.0);

        (base_confidence * stage_multiplier).min(1.0).max(0.0)
    }

    /// Calculate confidence increase (must have new signal)
    pub fn calculate_confidence_increase(
        &self,
        old_confidence: f64,
        new_signals: &[SignalContribution],
        current_stage: Option<RansomwareStage>,
        now: DateTime<Utc>,
    ) -> f64 {
        if new_signals.is_empty() {
            return old_confidence; // No increase without new signals
        }

        let new_confidence = self.calculate_confidence(new_signals, current_stage, now);
        old_confidence.max(new_confidence)
    }
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calculation() {
        let scorer = ConfidenceScorer::new();
        let now = Utc::now();
        
        let signals = vec![SignalContribution {
            signal_type: "encryption_activity".to_string(),
            base_confidence: 0.8,
            temporal_decay: 0.0,
            timestamp: now,
        }];
        
        let confidence = scorer.calculate_confidence(&signals, None, now);
        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_no_confidence_increase_without_signal() {
        let scorer = ConfidenceScorer::new();
        let old_conf = 0.5;
        let new_conf = scorer.calculate_confidence_increase(old_conf, &[], None, Utc::now());
        assert_eq!(new_conf, old_conf);
    }
}

