// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/engine.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Main correlation engine - orchestrates all components for deterministic detection

use crate::entity_state::EntityStateManager;
use crate::errors::CorrelationError;
use crate::explainability::{ExplainabilityArtifact, ExplainabilityGenerator, SignalExplanation, StageExplanation, TemporalEventExplanation, ConfidenceBreakdown};
use crate::graph::EntityGraph;
use crate::input::normalization::EventNormalizer;
use crate::input::validated_events::ValidatedEvent;
use crate::invariants::InvariantEnforcer;
use crate::kill_chain::inference::KillChainInferencer;
use crate::kill_chain::rules::Signal as KillChainSignal;
use crate::kill_chain::stages::RansomwareStage;
use crate::output::detection_result::{DetectionMetadata, DetectionResult};
use crate::scheduler::EntityScheduler;
use crate::scoring::{ConfidenceScorer, SignalContribution};
use crate::temporal::TemporalCorrelator;
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;

/// Correlation engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum entities
    pub max_entities: usize,
    /// Maximum signals per entity
    pub max_signals_per_entity: usize,
    /// Maximum transitions per entity
    pub max_transitions_per_entity: usize,
    /// Entity TTL (seconds)
    pub entity_ttl_seconds: u64,
    /// Temporal window size (seconds)
    pub temporal_window_seconds: u64,
    /// Maximum events per temporal window
    pub max_events_per_window: usize,
    /// Minimum confidence threshold
    pub min_confidence_threshold: f64,
    /// Minimum signal set for detection
    pub min_signal_set: HashSet<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_entities: 50000,
            max_signals_per_entity: 1000,
            max_transitions_per_entity: 50,
            entity_ttl_seconds: 3600,
            temporal_window_seconds: 300,
            max_events_per_window: 1000,
            min_confidence_threshold: 0.6,
            min_signal_set: HashSet::new(),
        }
    }
}

/// Main correlation engine
pub struct CorrelationEngine {
    /// Entity state manager
    state_manager: Arc<EntityStateManager>,
    /// Kill-chain inferencer
    inferencer: KillChainInferencer,
    /// Confidence scorer
    scorer: ConfidenceScorer,
    /// Temporal correlator
    temporal_correlator: Arc<parking_lot::RwLock<TemporalCorrelator>>,
    /// Entity graph
    graph: Arc<parking_lot::RwLock<EntityGraph>>,
    /// Scheduler
    scheduler: Arc<EntityScheduler>,
    /// Invariant enforcer
    invariant_enforcer: Arc<parking_lot::RwLock<InvariantEnforcer>>,
    /// Configuration
    config: EngineConfig,
}

impl CorrelationEngine {
    /// Create new correlation engine
    pub fn new(config: EngineConfig) -> Self {
        let state_manager = Arc::new(EntityStateManager::new(
            config.max_entities,
            config.max_signals_per_entity,
            config.max_transitions_per_entity,
            config.entity_ttl_seconds,
        ));

        let temporal_correlator = Arc::new(parking_lot::RwLock::new(
            TemporalCorrelator::new(
                config.temporal_window_seconds,
                config.max_events_per_window,
            ),
        ));

        let graph = Arc::new(parking_lot::RwLock::new(EntityGraph::new(
            config.max_entities,
        )));

        let scheduler = Arc::new(EntityScheduler::new());

        let invariant_enforcer = Arc::new(parking_lot::RwLock::new(
            InvariantEnforcer::new(
                config.max_signals_per_entity,
                config.min_signal_set.clone(),
            ),
        ));

        Self {
            state_manager,
            inferencer: KillChainInferencer::new(),
            scorer: ConfidenceScorer::new(),
            temporal_correlator,
            graph,
            scheduler,
            invariant_enforcer,
            config,
        }
    }

    /// Process validated event
    pub fn process_event(
        &self,
        event: ValidatedEvent,
    ) -> Result<Option<DetectionResult>, CorrelationError> {
        // Normalize event to signal
        let signal = EventNormalizer::normalize(&event);

        // Get or create entity state
        let _entity_state = self.state_manager.get_or_create_entity(&event.entity_id)?;

        // Add signal to entity (use event timestamp, not processing time)
        self.state_manager.add_signal(
            &event.entity_id,
            signal.signal_type.clone(),
            signal.confidence,
            signal.timestamp,
        )?;

        // Get current entity state
        let entity_state = self
            .state_manager
            .get_entity(&event.entity_id)
            .ok_or_else(|| CorrelationError::EntityNotFound(event.entity_id.clone()))?;

        // Build signal list for inference
        let signals: Vec<KillChainSignal> = entity_state
            .signal_history
            .iter()
            .map(|s| KillChainSignal {
                signal_type: s.signal_type.clone(),
                timestamp: s.timestamp,
                entity_id: event.entity_id.clone(),
                confidence: s.confidence,
                metadata: std::collections::HashMap::new(),
            })
            .collect();

        // Infer kill-chain stage
        let inference_result = self
            .inferencer
            .infer(entity_state.current_stage, &signals);

        if let Some(inference) = inference_result {
            // Require at least 2 signals total for InitialAccess to reduce false positives
            if inference.stage == crate::kill_chain::stages::RansomwareStage::InitialAccess 
                && signals.len() < 2 {
                // Not enough signals for reliable InitialAccess detection
                return Ok(None);
            }
            // Check invariants
            let mut enforcer = self.invariant_enforcer.write();

            // Enforce: no stage skip without evidence
            enforcer.enforce_no_stage_skip_without_evidence(
                entity_state.current_stage.as_ref().map(|s| s.name()),
                inference.stage.name(),
                &event.entity_id,
                !inference.contributing_signals.is_empty(),
            )?;

            // Use inference confidence (from rule engine) for detection threshold
            // This is the confidence that matched the rule pattern
            let inference_confidence = inference.confidence;
            
            // Calculate overall entity confidence using scorer (for state tracking)
            let signal_contributions: Vec<SignalContribution> = signals
                .iter()
                .map(|s| SignalContribution {
                    signal_type: s.signal_type.clone(),
                    base_confidence: s.confidence,
                    temporal_decay: 0.0,
                    timestamp: s.timestamp,
                })
                .collect();

            let new_confidence = self.scorer.calculate_confidence(
                &signal_contributions,
                Some(inference.stage),
                Utc::now(),
            );

            // Enforce: no confidence increase without new signal
            enforcer.enforce_no_confidence_increase_without_signal(
                &event.entity_id,
                entity_state.confidence,
                new_confidence.max(inference_confidence), // Use higher of the two
                true, // We have a new signal
            )?;

            // Enforce: no detection without minimum signal set
            let actual_signals: HashSet<String> = signals
                .iter()
                .map(|s| s.signal_type.clone())
                .collect();
            enforcer.enforce_no_detection_without_minimum_signals(&event.entity_id, &actual_signals)?;

            // Update entity state
            self.state_manager.add_transition(
                &event.entity_id,
                entity_state.current_stage,
                inference.stage,
                new_confidence,
            )?;

            // Check if detection threshold met (use inference confidence from rule engine)
            if inference_confidence >= self.config.min_confidence_threshold {
                // Generate explainability
                let explainability = self.generate_explainability(
                    &event.entity_id,
                    inference.stage,
                    &signals,
                    &entity_state,
                    new_confidence,
                )?;

                // Create detection result
                let metadata = DetectionMetadata {
                    engine_version: "1.0.0".to_string(),
                    rule_id: None,
                    signal_count: signals.len(),
                    stage_transition_count: entity_state.transition_history.len() + 1,
                };

                let detection = DetectionResult::new(
                    event.entity_id.clone(),
                    inference.stage,
                    inference_confidence, // Use inference confidence for detection
                    explainability,
                    metadata,
                );

                return Ok(Some(detection));
            }
        }

        Ok(None)
    }

    /// Generate explainability artifact
    fn generate_explainability(
        &self,
        entity_id: &str,
        current_stage: RansomwareStage,
        signals: &[KillChainSignal],
        entity_state: &crate::entity_state::EntityStateEntry,
        confidence: f64,
    ) -> Result<ExplainabilityArtifact, CorrelationError> {
        let signal_explanations: Vec<SignalExplanation> = signals
            .iter()
            .map(|s| SignalExplanation {
                signal_type: s.signal_type.clone(),
                timestamp: s.timestamp,
                confidence: s.confidence,
                contribution_to_detection: s.confidence,
                description: format!("Signal: {}", s.signal_type),
            })
            .collect();

        let stage_explanations: Vec<StageExplanation> = entity_state
            .transition_history
            .iter()
            .map(|t| StageExplanation {
                stage: t.to_stage.name().to_string(),
                entered_at: t.timestamp,
                confidence_at_entry: t.confidence,
                evidence_count: 1,
            })
            .collect();

        let temporal_explanations: Vec<TemporalEventExplanation> = signals
            .iter()
            .map(|s| TemporalEventExplanation {
                event_id: format!("signal_{}", s.signal_type),
                timestamp: s.timestamp,
                event_type: s.signal_type.clone(),
                description: format!("Signal event: {}", s.signal_type),
            })
            .collect();

        let confidence_breakdown = ConfidenceBreakdown {
            final_confidence: confidence,
            base_confidence: confidence,
            stage_multiplier: 1.0,
            temporal_decay_factor: 1.0,
            signal_contributions: vec![],
        };

        Ok(ExplainabilityGenerator::generate(
            entity_id,
            Some(current_stage),
            &signal_explanations,
            &stage_explanations,
            &temporal_explanations,
            confidence_breakdown,
            None,
        ))
    }

    /// Evict expired entities
    pub fn evict_expired(&self) -> usize {
        self.state_manager.evict_expired()
    }

    /// Get engine statistics
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            entity_count: self.state_manager.entity_count(),
            estimated_memory_bytes: self.state_manager.estimate_memory_usage(),
        }
    }
}

/// Engine statistics
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub entity_count: usize,
    pub estimated_memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::validated_events::ValidationMetadata;
    use std::collections::HashMap;

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = CorrelationEngine::new(config);
        let stats = engine.get_stats();
        assert_eq!(stats.entity_count, 0);
    }

    #[test]
    fn test_event_processing() {
        let config = EngineConfig::default();
        let engine = CorrelationEngine::new(config);

        let event = ValidatedEvent {
            event_id: "e1".to_string(),
            entity_id: "entity1".to_string(),
            timestamp: Utc::now(),
            signal_type: "network_connection".to_string(),
            payload: HashMap::new(),
            validation_metadata: ValidationMetadata {
                validated_at: Utc::now(),
                validator_version: "1.0".to_string(),
                checks_passed: vec![],
                validation_hash: None,
            },
        };

        let result = engine.process_event(event);
        assert!(result.is_ok());
    }
}
