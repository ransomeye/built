// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/kill_chain/inference.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Kill-chain stage inference engine - deterministic stage progression logic

use crate::kill_chain::rules::{KillChainRuleEngine, Signal};
use crate::kill_chain::stages::{RansomwareStage, StageMetadata};
use crate::kill_chain::transitions::{TransitionResult, TransitionRules};
use chrono::{DateTime, Utc};

/// Kill-chain inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Inferred stage
    pub stage: RansomwareStage,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Signals that contributed to inference
    pub contributing_signals: Vec<Signal>,
    /// Transition validation result
    pub transition_valid: bool,
    /// Rationale for inference
    pub rationale: String,
}

/// Kill-chain inference engine
pub struct KillChainInferencer {
    rule_engine: KillChainRuleEngine,
    transition_rules: TransitionRules,
}

impl KillChainInferencer {
    /// Create new inferencer
    pub fn new() -> Self {
        Self {
            rule_engine: KillChainRuleEngine::new(),
            transition_rules: TransitionRules::default(),
        }
    }

    /// Infer kill-chain stage from signals and current state
    pub fn infer(
        &self,
        current_stage: Option<RansomwareStage>,
        signals: &[Signal],
    ) -> Option<InferenceResult> {
        if signals.is_empty() {
            return None;
        }

        // Use rule engine to find matching stage
        if let Some((stage, confidence)) = self.rule_engine.infer_stage(current_stage, signals) {
            // Validate transition
            let transition_result = self
                .transition_rules
                .validate_transition(current_stage, stage, true);

            let transition_valid = matches!(transition_result, TransitionResult::Allowed);

            if !transition_valid {
                return None;
            }

            // Get contributing signals
            let contributing_signals: Vec<_> = signals
                .iter()
                .filter(|s| {
                    // Filter signals relevant to inferred stage
                    self.is_signal_relevant_to_stage(s, &stage)
                })
                .cloned()
                .collect();

            // Generate rationale
            let rationale = self.generate_rationale(current_stage, &stage, &contributing_signals);

            Some(InferenceResult {
                stage,
                confidence,
                contributing_signals,
                transition_valid: true,
                rationale,
            })
        } else {
            None
        }
    }

    /// Check if signal is relevant to stage
    fn is_signal_relevant_to_stage(&self, signal: &Signal, stage: &RansomwareStage) -> bool {
        if let Some(metadata) = self.rule_engine.get_stage_metadata(*stage) {
            metadata
                .entry_conditions
                .required_signals
                .contains(&signal.signal_type)
        } else {
            false
        }
    }

    /// Generate rationale for inference
    fn generate_rationale(
        &self,
        current_stage: Option<RansomwareStage>,
        inferred_stage: &RansomwareStage,
        signals: &[Signal],
    ) -> String {
        let stage_name = inferred_stage.name();
        let signal_count = signals.len();
        let signal_types: Vec<_> = signals
            .iter()
            .map(|s| s.signal_type.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        format!(
            "Inferred stage {} based on {} signals of types: {}. Previous stage: {:?}",
            stage_name,
            signal_count,
            signal_types.join(", "),
            current_stage.map(|s| s.name())
        )
    }

    /// Get stage metadata
    pub fn get_stage_metadata(&self, stage: RansomwareStage) -> Option<StageMetadata> {
        self.rule_engine.get_stage_metadata(stage).cloned()
    }
}

impl Default for KillChainInferencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_inference_with_signals() {
        let inferencer = KillChainInferencer::new();
        let signals = vec![Signal {
            signal_type: "network_connection".to_string(),
            timestamp: Utc::now(),
            entity_id: "test".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
        }];

        let result = inferencer.infer(None, &signals);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.stage, RansomwareStage::InitialAccess);
        assert!(result.confidence >= 0.6);
        assert!(!result.contributing_signals.is_empty());
    }

    #[test]
    fn test_inference_empty_signals() {
        let inferencer = KillChainInferencer::new();
        let result = inferencer.infer(None, &[]);
        assert!(result.is_none());
    }
}

