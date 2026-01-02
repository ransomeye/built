// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/kill_chain/rules.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Kill-chain inference rules - deterministic rules for stage progression

use crate::kill_chain::stages::{RansomwareStage, StageMetadata};
use crate::kill_chain::transitions::TransitionRules;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Signal type identifier
pub type SignalType = String;

/// Signal evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: SignalType,
    pub timestamp: DateTime<Utc>,
    pub entity_id: String,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Kill-chain rule for stage inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillChainRule {
    /// Rule identifier
    pub id: String,
    /// Rule version
    pub version: String,
    /// Target stage
    pub target_stage: RansomwareStage,
    /// Required signal patterns
    pub required_signals: Vec<SignalPattern>,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Temporal constraints
    pub temporal_constraints: Option<TemporalConstraint>,
}

/// Signal pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPattern {
    /// Signal type to match
    pub signal_type: SignalType,
    /// Minimum count required
    pub min_count: usize,
    /// Maximum count (None = unlimited)
    pub max_count: Option<usize>,
}

/// Temporal constraint for signal correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConstraint {
    /// Maximum time window in seconds for all signals
    pub max_window_seconds: u64,
    /// Minimum time between signals (None = no minimum)
    pub min_interval_seconds: Option<u64>,
}

/// Kill-chain rule engine
pub struct KillChainRuleEngine {
    rules: Vec<KillChainRule>,
    stage_metadata: HashMap<RansomwareStage, StageMetadata>,
    transition_rules: TransitionRules,
}

impl KillChainRuleEngine {
    /// Create new rule engine with default rules
    pub fn new() -> Self {
        let mut stage_metadata = HashMap::new();
        for stage in RansomwareStage::all_stages() {
            stage_metadata.insert(stage, StageMetadata::default_for(stage));
        }

        Self {
            rules: Self::default_rules(),
            stage_metadata,
            transition_rules: TransitionRules::default(),
        }
    }

    /// Get default kill-chain rules
    fn default_rules() -> Vec<KillChainRule> {
        vec![
            KillChainRule {
                id: "initial_access_1".to_string(),
                version: "1.0".to_string(),
                target_stage: RansomwareStage::InitialAccess,
                required_signals: vec![
                    SignalPattern {
                        signal_type: "network_connection".to_string(),
                        min_count: 1,
                        max_count: None,
                    },
                ],
                min_confidence: 0.6,
                temporal_constraints: Some(TemporalConstraint {
                    max_window_seconds: 300,
                    min_interval_seconds: None,
                }),
            },
            KillChainRule {
                id: "execution_1".to_string(),
                version: "1.0".to_string(),
                target_stage: RansomwareStage::Execution,
                required_signals: vec![
                    SignalPattern {
                        signal_type: "process_creation".to_string(),
                        min_count: 1,
                        max_count: None,
                    },
                ],
                min_confidence: 0.7,
                temporal_constraints: Some(TemporalConstraint {
                    max_window_seconds: 60,
                    min_interval_seconds: None,
                }),
            },
            KillChainRule {
                id: "encryption_execution_1".to_string(),
                version: "1.0".to_string(),
                target_stage: RansomwareStage::EncryptionExecution,
                required_signals: vec![
                    SignalPattern {
                        signal_type: "file_modification".to_string(),
                        min_count: 10,
                        max_count: None,
                    },
                    SignalPattern {
                        signal_type: "encryption_activity".to_string(),
                        min_count: 1,
                        max_count: None,
                    },
                ],
                min_confidence: 0.8,
                temporal_constraints: Some(TemporalConstraint {
                    max_window_seconds: 60,
                    min_interval_seconds: Some(1),
                }),
            },
            KillChainRule {
                id: "impact_1".to_string(),
                version: "1.0".to_string(),
                target_stage: RansomwareStage::Impact,
                required_signals: vec![
                    SignalPattern {
                        signal_type: "ransom_note".to_string(),
                        min_count: 1,
                        max_count: None,
                    },
                ],
                min_confidence: 0.9,
                temporal_constraints: Some(TemporalConstraint {
                    max_window_seconds: 300,
                    min_interval_seconds: None,
                }),
            },
        ]
    }

    /// Evaluate signals against rules to infer stage
    pub fn infer_stage(
        &self,
        current_stage: Option<RansomwareStage>,
        signals: &[Signal],
    ) -> Option<(RansomwareStage, f64)> {
        let mut best_match: Option<(RansomwareStage, f64)> = None;

        for rule in &self.rules {
            // Check transition validity
            let transition_result = self
                .transition_rules
                .validate_transition(current_stage, rule.target_stage, true);
            
            if !matches!(transition_result, crate::kill_chain::transitions::TransitionResult::Allowed) {
                continue;
            }

            // Evaluate rule against signals
            if let Some(confidence) = self.evaluate_rule(rule, signals) {
                if confidence >= rule.min_confidence {
                    match best_match {
                        None => best_match = Some((rule.target_stage, confidence)),
                        Some((_, best_conf)) if confidence > best_conf => {
                            best_match = Some((rule.target_stage, confidence))
                        }
                        _ => {}
                    }
                }
            }
        }

        best_match
    }

    /// Evaluate a rule against signals
    fn evaluate_rule(&self, rule: &KillChainRule, signals: &[Signal]) -> Option<f64> {
        let mut matched_signals = 0;
        let mut total_confidence = 0.0;
        let mut signal_count = 0;
        let mut pattern_matching_signals: Vec<&Signal> = Vec::new();

        for pattern in &rule.required_signals {
            let matching: Vec<_> = signals
                .iter()
                .filter(|s| s.signal_type == pattern.signal_type)
                .collect();

            let count = matching.len();
            if count < pattern.min_count {
                return None; // Pattern not satisfied
            }

            if let Some(max) = pattern.max_count {
                if count > max {
                    return None; // Pattern exceeded
                }
            }

            matched_signals += 1;
            for signal in &matching {
                total_confidence += signal.confidence;
                signal_count += 1;
                pattern_matching_signals.push(*signal);
            }
        }

        // Check temporal constraints on pattern-matching signals only
        if let Some(temporal) = &rule.temporal_constraints {
            if !self.check_temporal_constraints(&pattern_matching_signals, temporal) {
                return None;
            }
        }

        if matched_signals == rule.required_signals.len() && signal_count > 0 {
            Some(total_confidence / signal_count as f64)
        } else {
            None
        }
    }

    /// Check temporal constraints
    fn check_temporal_constraints(
        &self,
        signals: &[&Signal],
        constraint: &TemporalConstraint,
    ) -> bool {
        if signals.is_empty() {
            return false;
        }

        let timestamps: Vec<_> = signals.iter().map(|s| s.timestamp).collect();
        let min_ts = timestamps.iter().min().unwrap();
        let max_ts = timestamps.iter().max().unwrap();

        let window_seconds = (*max_ts - *min_ts).num_seconds() as u64;
        if window_seconds > constraint.max_window_seconds {
            return false;
        }

        // Check minimum interval if specified
        // This ensures signals of the same type are spaced out (not all signals)
        // We check intervals between consecutive signals of the same type
        if let Some(min_interval) = constraint.min_interval_seconds {
            // Group signals by type and check intervals within each group
            use std::collections::HashMap;
            let mut signals_by_type: HashMap<String, Vec<&Signal>> = HashMap::new();
            for signal in signals {
                signals_by_type
                    .entry(signal.signal_type.clone())
                    .or_insert_with(Vec::new)
                    .push(signal);
            }
            
            // Check intervals for each signal type
            for (_, mut type_signals) in signals_by_type {
                if type_signals.len() > 1 {
                    type_signals.sort_by_key(|s| s.timestamp);
                    for i in 1..type_signals.len() {
                        let interval = (type_signals[i].timestamp - type_signals[i - 1].timestamp).num_seconds() as u64;
                        if interval < min_interval {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    /// Get stage metadata
    pub fn get_stage_metadata(&self, stage: RansomwareStage) -> Option<&StageMetadata> {
        self.stage_metadata.get(&stage)
    }
}

impl Default for KillChainRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_access_inference() {
        let engine = KillChainRuleEngine::new();
        let signals = vec![Signal {
            signal_type: "network_connection".to_string(),
            timestamp: Utc::now(),
            entity_id: "test".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
        }];

        let result = engine.infer_stage(None, &signals);
        assert!(result.is_some());
        let (stage, conf) = result.unwrap();
        assert_eq!(stage, RansomwareStage::InitialAccess);
        assert!(conf >= 0.6);
    }
}

