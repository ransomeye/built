// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/output/detection_result.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Detection result output - authoritative detection outputs for Policy Engine

use crate::explainability::{ExplainabilityArtifact, ConfidenceBreakdown};
use crate::kill_chain::stages::RansomwareStage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Detection result (authoritative output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Detection identifier
    pub detection_id: String,
    /// Entity identifier
    pub entity_id: String,
    /// Detection timestamp
    pub timestamp: DateTime<Utc>,
    /// Inferred kill-chain stage
    pub kill_chain_stage: RansomwareStage,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Explainability artifact
    pub explainability: ExplainabilityArtifact,
    /// Detection metadata
    pub metadata: DetectionMetadata,
}

/// Detection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionMetadata {
    /// Correlation engine version
    pub engine_version: String,
    /// Detection rule ID
    pub rule_id: Option<String>,
    /// Signal count
    pub signal_count: usize,
    /// Stage transition count
    pub stage_transition_count: usize,
}

impl DetectionResult {
    /// Create new detection result
    pub fn new(
        entity_id: String,
        kill_chain_stage: RansomwareStage,
        confidence: f64,
        explainability: ExplainabilityArtifact,
        metadata: DetectionMetadata,
    ) -> Self {
        Self {
            detection_id: uuid::Uuid::new_v4().to_string(),
            entity_id,
            timestamp: Utc::now(),
            kill_chain_stage,
            confidence,
            explainability,
            metadata,
        }
    }

    /// Check if detection meets minimum confidence threshold
    pub fn meets_threshold(&self, min_confidence: f64) -> bool {
        self.confidence >= min_confidence
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explainability::{ExplainabilityArtifact, ConfidenceBreakdown};

    #[test]
    fn test_detection_result_creation() {
        let explainability = ExplainabilityArtifact {
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signals_involved: vec![],
            stages_traversed: vec![],
            temporal_sequence: vec![],
            confidence_calculation: ConfidenceBreakdown {
                final_confidence: 0.8,
                base_confidence: 0.8,
                stage_multiplier: 1.0,
                temporal_decay_factor: 1.0,
                signal_contributions: vec![],
            },
            intelligence_context: None,
            human_readable_summary: "Test".to_string(),
        };

        let metadata = DetectionMetadata {
            engine_version: "1.0".to_string(),
            rule_id: None,
            signal_count: 1,
            stage_transition_count: 1,
        };

        let result = DetectionResult::new(
            "entity1".to_string(),
            RansomwareStage::EncryptionExecution,
            0.8,
            explainability,
            metadata,
        );

        assert_eq!(result.entity_id, "entity1");
        assert!(result.meets_threshold(0.7));
        assert!(!result.meets_threshold(0.9));
    }
}

