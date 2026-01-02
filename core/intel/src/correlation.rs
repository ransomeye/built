// Path and File Name : /home/ransomeye/rebuild/core/intel/src/correlation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Intel correlation - correlates intel across telemetry, deception, lateral movement

use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::errors::IntelError;
use crate::confidence::ConfidenceScorer;

/// Signal source type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSource {
    Telemetry,
    Deception,
    LateralMovement,
    ThreatIntel,
}

/// Correlated signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedSignal {
    pub signal_id: String,
    pub source: SignalSource,
    pub ioc_value: String,
    pub ioc_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// Correlation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub correlation_id: String,
    pub ioc_value: String,
    pub ioc_type: String,
    pub signals: Vec<CorrelatedSignal>,
    pub confidence_score: f64,
    pub source_agreement: f64, // 0.0 to 1.0
    pub signal_frequency: u32,
    pub temporal_proximity: Duration,
    pub correlated_at: DateTime<Utc>,
}

/// Intel correlator - correlates signals across multiple sources
pub struct IntelCorrelator {
    scorer: ConfidenceScorer,
    signal_window: Duration,
    min_signals_for_correlation: u32,
}

impl IntelCorrelator {
    /// Create new intel correlator
    pub fn new(signal_window_secs: i64, min_signals: u32) -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            signal_window: Duration::seconds(signal_window_secs),
            min_signals_for_correlation: min_signals,
        }
    }
    
    /// Correlate signals across sources
    /// 
    /// FAIL-CLOSED: Single weak signal does not auto-escalate
    /// Requires multiple signals or high-confidence single signal
    pub fn correlate_signals(&self, signals: Vec<CorrelatedSignal>) -> Result<CorrelationResult, IntelError> {
        if signals.is_empty() {
            return Err(IntelError::NoSignals("No signals provided for correlation".to_string()));
        }
        
        // Group signals by IOC value and type
        let mut signal_groups: HashMap<(String, String), Vec<&CorrelatedSignal>> = HashMap::new();
        for signal in &signals {
            let key = (signal.ioc_value.clone(), signal.ioc_type.clone());
            signal_groups.entry(key)
                .or_insert_with(Vec::new)
                .push(signal);
        }
        
        // Find the group with most signals
        let (best_key, best_signals) = signal_groups.iter()
            .max_by_key(|(_, sigs)| sigs.len())
            .ok_or_else(|| IntelError::CorrelationFailed("No signal groups found".to_string()))?;
        
        // FAIL-CLOSED: Require minimum signals for correlation
        if best_signals.len() < self.min_signals_for_correlation as usize {
            // Check if we have a single high-confidence signal
            let max_confidence = best_signals.iter()
                .map(|s| s.confidence)
                .fold(0.0, f64::max);
            
            if max_confidence < 0.9 {
                return Err(IntelError::InsufficientConfidence(
                    format!("Only {} signal(s) with max confidence {:.2} (need {} signals or confidence >= 0.9)", 
                            best_signals.len(), max_confidence, self.min_signals_for_correlation)
                ));
            }
        }
        
        // Calculate correlation metrics
        let signal_frequency = best_signals.len() as u32;
        
        // Calculate source agreement (how many different sources agree)
        let unique_sources: std::collections::HashSet<_> = best_signals.iter()
            .map(|s| &s.source)
            .collect();
        let source_agreement = unique_sources.len() as f64 / 4.0; // 4 possible sources
        
        // Calculate temporal proximity
        let timestamps: Vec<DateTime<Utc>> = best_signals.iter()
            .map(|s| s.timestamp)
            .collect();
        let min_time = timestamps.iter().min().unwrap();
        let max_time = timestamps.iter().max().unwrap();
        let temporal_proximity = *max_time - *min_time;
        
        // Calculate confidence score using scorer
        let confidence_score = self.scorer.compute_confidence(
            best_signals.iter().map(|s| s.confidence).collect(),
            source_agreement,
            signal_frequency as f64,
            temporal_proximity,
        )?;
        
        let correlation_id = format!("corr_{}", uuid::Uuid::new_v4().to_string());
        
        let result = CorrelationResult {
            correlation_id,
            ioc_value: best_key.0.clone(),
            ioc_type: best_key.1.clone(),
            signals: best_signals.iter().map(|s| (*s).clone()).collect(),
            confidence_score,
            source_agreement,
            signal_frequency,
            temporal_proximity,
            correlated_at: Utc::now(),
        };
        
        info!("Correlated {} signals for IOC {} (confidence: {:.2})", 
              signal_frequency, best_key.0, confidence_score);
        
        Ok(result)
    }
    
    /// Check if signals are within temporal window
    pub fn signals_in_window(&self, signals: &[CorrelatedSignal]) -> bool {
        if signals.is_empty() {
            return false;
        }
        
        let timestamps: Vec<DateTime<Utc>> = signals.iter()
            .map(|s| s.timestamp)
            .collect();
        let min_time = timestamps.iter().min().unwrap();
        let max_time = timestamps.iter().max().unwrap();
        let span = *max_time - *min_time;
        
        span <= self.signal_window
    }
}

