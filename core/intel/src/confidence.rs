// Path and File Name : /home/ransomeye/rebuild/core/intel/src/confidence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Confidence scoring - computes confidence using source reputation, signal frequency, cross-source agreement, temporal proximity

use chrono::Duration;
use tracing::debug;

use crate::errors::IntelError;

/// Confidence scorer - computes confidence scores for correlated intel
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Create new confidence scorer
    pub fn new() -> Self {
        Self
    }
    
    /// Compute confidence score
    /// 
    /// Factors:
    /// - Source reputation (weight: 0.3)
    /// - Signal frequency (weight: 0.2)
    /// - Cross-source agreement (weight: 0.3)
    /// - Temporal proximity (weight: 0.2)
    pub fn compute_confidence(
        &self,
        signal_confidences: Vec<f64>,
        source_agreement: f64,
        signal_frequency: f64,
        temporal_proximity: Duration,
    ) -> Result<f64, IntelError> {
        if signal_confidences.is_empty() {
            return Err(IntelError::ScoringFailed("No signal confidences provided".to_string()));
        }
        
        // Average signal confidence (representing source reputation)
        let avg_signal_confidence = signal_confidences.iter().sum::<f64>() / signal_confidences.len() as f64;
        
        // Normalize signal frequency (0-1 scale, assuming max 10 signals)
        let normalized_frequency = (signal_frequency / 10.0).min(1.0);
        
        // Normalize temporal proximity (closer = higher score)
        // Score decreases as time span increases
        let proximity_seconds = temporal_proximity.num_seconds().max(0);
        let normalized_proximity = if proximity_seconds == 0 {
            1.0
        } else {
            // Exponential decay: score = e^(-seconds/3600)
            // 1 hour = 0.368, 6 hours = 0.002
            (-(proximity_seconds as f64) / 3600.0).exp()
        };
        
        // Weighted combination
        let confidence = 
            (avg_signal_confidence * 0.3) +
            (normalized_frequency * 0.2) +
            (source_agreement * 0.3) +
            (normalized_proximity * 0.2);
        
        // Clamp to [0.0, 1.0]
        let confidence = confidence.max(0.0).min(1.0);
        
        debug!("Computed confidence: {:.2} (signal: {:.2}, frequency: {:.2}, agreement: {:.2}, proximity: {:.2})",
               confidence, avg_signal_confidence, normalized_frequency, source_agreement, normalized_proximity);
        
        Ok(confidence)
    }
    
    /// Classify confidence level
    pub fn classify_confidence(&self, confidence: f64) -> ConfidenceLevel {
        if confidence >= 0.8 {
            ConfidenceLevel::High
        } else if confidence >= 0.5 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }
}

/// Confidence level classification
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceLevel {
    High,    // >= 0.8
    Medium,  // >= 0.5
    Low,     // < 0.5
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

