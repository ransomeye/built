// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/explainability.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Explainability engine - generates human and machine-readable detection rationale

use crate::kill_chain::stages::RansomwareStage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Explainability artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainabilityArtifact {
    /// Entity ID
    pub entity_id: String,
    /// Detection timestamp
    pub timestamp: DateTime<Utc>,
    /// Signals involved in detection
    pub signals_involved: Vec<SignalExplanation>,
    /// Kill-chain stages traversed
    pub stages_traversed: Vec<StageExplanation>,
    /// Temporal sequence of events
    pub temporal_sequence: Vec<TemporalEventExplanation>,
    /// Confidence calculation breakdown
    pub confidence_calculation: ConfidenceBreakdown,
    /// Intelligence context used (if any)
    pub intelligence_context: Option<IntelligenceContext>,
    /// Human-readable summary
    pub human_readable_summary: String,
}

/// Signal explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalExplanation {
    pub signal_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub contribution_to_detection: f64,
    pub description: String,
}

/// Stage explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageExplanation {
    pub stage: String,
    pub entered_at: DateTime<Utc>,
    pub confidence_at_entry: f64,
    pub evidence_count: usize,
}

/// Temporal event explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEventExplanation {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
}

/// Confidence calculation breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    pub final_confidence: f64,
    pub base_confidence: f64,
    pub stage_multiplier: f64,
    pub temporal_decay_factor: f64,
    pub signal_contributions: Vec<SignalContributionExplanation>,
}

/// Signal contribution to confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalContributionExplanation {
    pub signal_type: String,
    pub base_weight: f64,
    pub adjusted_weight: f64,
    pub contribution: f64,
}

/// Intelligence context (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceContext {
    pub threat_intel_matches: Vec<String>,
    pub ai_confidence_hints: Vec<String>,
    pub baseline_anomalies: Vec<String>,
}

/// Explainability generator
pub struct ExplainabilityGenerator;

impl ExplainabilityGenerator {
    /// Generate explainability artifact
    pub fn generate(
        entity_id: &str,
        current_stage: Option<RansomwareStage>,
        signals: &[SignalExplanation],
        stages: &[StageExplanation],
        temporal_events: &[TemporalEventExplanation],
        confidence_breakdown: ConfidenceBreakdown,
        intelligence_context: Option<IntelligenceContext>,
    ) -> ExplainabilityArtifact {
        let human_summary = Self::generate_human_readable_summary(
            entity_id,
            current_stage,
            signals,
            stages,
            &confidence_breakdown,
        );

        ExplainabilityArtifact {
            entity_id: entity_id.to_string(),
            timestamp: Utc::now(),
            signals_involved: signals.to_vec(),
            stages_traversed: stages.to_vec(),
            temporal_sequence: temporal_events.to_vec(),
            confidence_calculation: confidence_breakdown,
            intelligence_context,
            human_readable_summary: human_summary,
        }
    }

    /// Generate human-readable summary
    fn generate_human_readable_summary(
        entity_id: &str,
        current_stage: Option<RansomwareStage>,
        signals: &[SignalExplanation],
        stages: &[StageExplanation],
        confidence: &ConfidenceBreakdown,
    ) -> String {
        let stage_name = current_stage
            .map(|s| s.name())
            .unwrap_or("Unknown");
        
        let signal_count = signals.len();
        let signal_types: Vec<_> = signals
            .iter()
            .map(|s| s.signal_type.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        format!(
            "Entity {} detected at kill-chain stage {} with confidence {:.2}%. \
            Detection based on {} signals of types: {}. \
            Traversed {} stages. \
            Confidence calculated from base {:.2}% with stage multiplier {:.2} and temporal decay {:.2}.",
            entity_id,
            stage_name,
            confidence.final_confidence * 100.0,
            signal_count,
            signal_types.join(", "),
            stages.len(),
            confidence.base_confidence * 100.0,
            confidence.stage_multiplier,
            confidence.temporal_decay_factor
        )
    }

    /// Export to JSON
    pub fn to_json(artifact: &ExplainabilityArtifact) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(artifact)
    }

    /// Export to structured format for machine consumption
    pub fn to_structured(artifact: &ExplainabilityArtifact) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();
        result.insert("entity_id".to_string(), serde_json::json!(artifact.entity_id));
        result.insert("timestamp".to_string(), serde_json::json!(artifact.timestamp.to_rfc3339()));
        result.insert("confidence".to_string(), serde_json::json!(artifact.confidence_calculation.final_confidence));
        result.insert("stage".to_string(), serde_json::json!(artifact.stages_traversed.last().map(|s| s.stage.clone())));
        result.insert("signal_count".to_string(), serde_json::json!(artifact.signals_involved.len()));
        result.insert("stage_count".to_string(), serde_json::json!(artifact.stages_traversed.len()));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainability_generation() {
        let signals = vec![SignalExplanation {
            signal_type: "encryption_activity".to_string(),
            timestamp: Utc::now(),
            confidence: 0.8,
            contribution_to_detection: 0.5,
            description: "File encryption detected".to_string(),
        }];
        
        let stages = vec![StageExplanation {
            stage: "EncryptionExecution".to_string(),
            entered_at: Utc::now(),
            confidence_at_entry: 0.8,
            evidence_count: 1,
        }];
        
        let confidence = ConfidenceBreakdown {
            final_confidence: 0.8,
            base_confidence: 0.8,
            stage_multiplier: 1.0,
            temporal_decay_factor: 1.0,
            signal_contributions: vec![],
        };
        
        let artifact = ExplainabilityGenerator::generate(
            "entity1",
            Some(RansomwareStage::EncryptionExecution),
            &signals,
            &stages,
            &[],
            confidence,
            None,
        );
        
        assert!(!artifact.human_readable_summary.is_empty());
        assert_eq!(artifact.entity_id, "entity1");
    }
}

