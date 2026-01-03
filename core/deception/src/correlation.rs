// Path and File Name : /home/ransomeye/rebuild/core/deception/src/correlation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Integration with Phase 5 correlation engine - exposes deception signals as strong indicators

#![cfg(feature = "future-deception")]

use std::sync::Arc;
use tracing::{info, debug};
use chrono::Utc;

use crate::signals::DeceptionSignal;
use crate::errors::DeceptionError;

/// Correlation integration for deception signals
/// 
/// Deception signals are treated as STRONG indicators (confidence >= 0.9)
/// They can:
/// - Elevate confidence in correlation
/// - Short-circuit detection timelines
/// - NOT auto-enforce (explicit playbook mapping required)
pub struct CorrelationIntegration;

impl CorrelationIntegration {
    /// Convert deception signal to correlation event
    /// 
    /// Deception signals are high-confidence by design (>= 0.9)
    /// They are treated as strong indicators, not probabilistic noise
    pub fn signal_to_correlation_event(signal: &DeceptionSignal) -> Result<CorrelationEvent, DeceptionError> {
        // Validate signal has required confidence
        if signal.confidence_score < 0.9 {
            return Err(DeceptionError::SignalGenerationFailed(
                format!("Signal confidence {} is below minimum threshold 0.9", signal.confidence_score)
            ));
        }
        
        // Create correlation event from deception signal
        let event = CorrelationEvent {
            event_id: signal.signal_id.clone(),
            entity_id: format!("deception:{}", signal.asset_id),
            signal_type: format!("deception:{}", signal.interaction_type),
            timestamp: signal.timestamp,
            confidence: signal.confidence_score,
            source: SignalSource::Deception,
            metadata: signal.metadata.clone(),
        };
        
        debug!("Converted deception signal {} to correlation event", signal.signal_id);
        Ok(event)
    }
    
    /// Check if signal should elevate correlation confidence
    /// 
    /// Deception signals always elevate confidence (they are high-confidence by design)
    pub fn should_elevate_confidence(signal: &DeceptionSignal) -> bool {
        // Deception signals are always high-confidence (>= 0.9)
        // They should always elevate correlation confidence
        signal.confidence_score >= 0.9
    }
}

/// Correlation event structure (compatible with Phase 5)
#[derive(Debug, Clone)]
pub struct CorrelationEvent {
    pub event_id: String,
    pub entity_id: String,
    pub signal_type: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub confidence: f64,
    pub source: SignalSource,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalSource {
    Deception,
    Telemetry,
    ThreatIntel,
    NetworkScanner,
}

