// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/output/rationale.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Detection rationale - human and machine-readable explanation

use serde::{Deserialize, Serialize};

/// Detection rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRationale {
    /// Human-readable summary
    pub summary: String,
    /// Key signals that triggered detection
    pub key_signals: Vec<String>,
    /// Kill-chain progression
    pub kill_chain_progression: Vec<String>,
    /// Confidence factors
    pub confidence_factors: Vec<String>,
}

impl DetectionRationale {
    /// Create new rationale
    pub fn new(
        summary: String,
        key_signals: Vec<String>,
        kill_chain_progression: Vec<String>,
        confidence_factors: Vec<String>,
    ) -> Self {
        Self {
            summary,
            key_signals,
            kill_chain_progression,
            confidence_factors,
        }
    }

    /// Generate from explainability artifact
    pub fn from_explainability(artifact: &crate::explainability::ExplainabilityArtifact) -> Self {
        let key_signals: Vec<String> = artifact
            .signals_involved
            .iter()
            .map(|s| format!("{} (conf: {:.2})", s.signal_type, s.confidence))
            .collect();

        let kill_chain_progression: Vec<String> = artifact
            .stages_traversed
            .iter()
            .map(|s| format!("{} -> {}", s.stage, s.confidence_at_entry))
            .collect();

        let confidence_factors = vec![
            format!("Base confidence: {:.2}", artifact.confidence_calculation.base_confidence),
            format!("Stage multiplier: {:.2}", artifact.confidence_calculation.stage_multiplier),
            format!("Temporal decay: {:.2}", artifact.confidence_calculation.temporal_decay_factor),
        ];

        Self {
            summary: artifact.human_readable_summary.clone(),
            key_signals,
            kill_chain_progression,
            confidence_factors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rationale_creation() {
        let rationale = DetectionRationale::new(
            "Test summary".to_string(),
            vec!["signal1".to_string()],
            vec!["InitialAccess".to_string()],
            vec![],
        );

        assert_eq!(rationale.summary, "Test summary");
        assert_eq!(rationale.key_signals.len(), 1);
    }
}

