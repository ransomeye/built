// Path and File Name : /home/ransomeye/rebuild/core/intel/tests/correlation_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for intel correlation - single weak signal does not escalate, multi-signal correlation escalates

use chrono::Utc;
use intel::correlation::{IntelCorrelator, CorrelatedSignal, SignalSource};
use intel::errors::IntelError;

#[test]
fn test_single_weak_signal_does_not_escalate() {
    let correlator = IntelCorrelator::new(3600, 2); // Require 2 signals minimum
    
    // Single weak signal (low confidence)
    let signals = vec![CorrelatedSignal {
        signal_id: "signal1".to_string(),
        source: SignalSource::Telemetry,
        ioc_value: "192.168.1.100".to_string(),
        ioc_type: "IP".to_string(),
        timestamp: Utc::now(),
        confidence: 0.3, // Low confidence
        metadata: serde_json::json!({}),
    }];
    
    let result = correlator.correlate_signals(signals);
    assert!(result.is_err());
    
    if let Err(IntelError::InsufficientConfidence(_)) = result {
        // Expected error - single weak signal should not escalate
    } else {
        panic!("Expected InsufficientConfidence error for single weak signal");
    }
}

#[test]
fn test_multi_signal_correlation_escalates() {
    let correlator = IntelCorrelator::new(3600, 2);
    
    // Multiple signals with good confidence
    let signals = vec![
        CorrelatedSignal {
            signal_id: "signal1".to_string(),
            source: SignalSource::Telemetry,
            ioc_value: "192.168.1.100".to_string(),
            ioc_type: "IP".to_string(),
            timestamp: Utc::now(),
            confidence: 0.8,
            metadata: serde_json::json!({}),
        },
        CorrelatedSignal {
            signal_id: "signal2".to_string(),
            source: SignalSource::Deception,
            ioc_value: "192.168.1.100".to_string(),
            ioc_type: "IP".to_string(),
            timestamp: Utc::now(),
            confidence: 0.9,
            metadata: serde_json::json!({}),
        },
    ];
    
    let result = correlator.correlate_signals(signals);
    assert!(result.is_ok());
    
    let correlation = result.unwrap();
    assert_eq!(correlation.signal_frequency, 2);
    assert!(correlation.confidence_score > 0.5);
}

#[test]
fn test_high_confidence_single_signal_escalates() {
    let correlator = IntelCorrelator::new(3600, 2);
    
    // Single high-confidence signal (>= 0.9)
    let signals = vec![CorrelatedSignal {
        signal_id: "signal1".to_string(),
        source: SignalSource::ThreatIntel,
        ioc_value: "192.168.1.100".to_string(),
        ioc_type: "IP".to_string(),
        timestamp: Utc::now(),
        confidence: 0.95, // High confidence
        metadata: serde_json::json!({}),
    }];
    
    let result = correlator.correlate_signals(signals);
    assert!(result.is_ok()); // High confidence single signal should escalate
    
    let correlation = result.unwrap();
    assert!(correlation.confidence_score >= 0.9);
}

#[test]
fn test_cross_source_agreement() {
    let correlator = IntelCorrelator::new(3600, 2);
    
    // Signals from different sources
    let signals = vec![
        CorrelatedSignal {
            signal_id: "signal1".to_string(),
            source: SignalSource::Telemetry,
            ioc_value: "192.168.1.100".to_string(),
            ioc_type: "IP".to_string(),
            timestamp: Utc::now(),
            confidence: 0.8,
            metadata: serde_json::json!({}),
        },
        CorrelatedSignal {
            signal_id: "signal2".to_string(),
            source: SignalSource::Deception,
            ioc_value: "192.168.1.100".to_string(),
            ioc_type: "IP".to_string(),
            timestamp: Utc::now(),
            confidence: 0.9,
            metadata: serde_json::json!({}),
        },
        CorrelatedSignal {
            signal_id: "signal3".to_string(),
            source: SignalSource::LateralMovement,
            ioc_value: "192.168.1.100".to_string(),
            ioc_type: "IP".to_string(),
            timestamp: Utc::now(),
            confidence: 0.85,
            metadata: serde_json::json!({}),
        },
    ];
    
    let result = correlator.correlate_signals(signals);
    assert!(result.is_ok());
    
    let correlation = result.unwrap();
    assert_eq!(correlation.signal_frequency, 3);
    assert!(correlation.source_agreement > 0.5); // Multiple sources agree
}

#[test]
fn test_no_signals_error() {
    let correlator = IntelCorrelator::new(3600, 2);
    
    let result = correlator.correlate_signals(Vec::new());
    assert!(result.is_err());
    
    if let Err(IntelError::NoSignals(_)) = result {
        // Expected error
    } else {
        panic!("Expected NoSignals error");
    }
}

