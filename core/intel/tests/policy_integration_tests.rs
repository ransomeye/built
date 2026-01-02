// Path and File Name : /home/ransomeye/rebuild/core/intel/tests/policy_integration_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for policy integration - high-confidence triggers policy, medium requires corroboration, low logs only

use chrono::Utc;
use intel::correlation::{CorrelationResult, CorrelatedSignal, SignalSource};
use intel::policy_integration::PolicyIntegration;
use intel::scoring::EscalationDecision;
use intel::confidence::ConfidenceLevel;

#[test]
fn test_high_confidence_triggers_policy() {
    let integration = PolicyIntegration::new();
    
    // High confidence correlation (>= 0.8)
    let correlation = CorrelationResult {
        correlation_id: "test_corr".to_string(),
        ioc_value: "192.168.1.100".to_string(),
        ioc_type: "IP".to_string(),
        signals: vec![
            CorrelatedSignal {
                signal_id: "signal1".to_string(),
                source: SignalSource::ThreatIntel,
                ioc_value: "192.168.1.100".to_string(),
                ioc_type: "IP".to_string(),
                timestamp: Utc::now(),
                confidence: 0.9,
                metadata: serde_json::json!({}),
            },
        ],
        confidence_score: 0.85,
        source_agreement: 0.8,
        signal_frequency: 1,
        temporal_proximity: chrono::Duration::seconds(0),
        correlated_at: Utc::now(),
    };
    
    let can_trigger = integration.can_trigger_policy(&correlation).unwrap();
    assert!(can_trigger, "High confidence should trigger policy");
    
    let decision = integration.get_escalation_decision(&correlation).unwrap();
    assert!(decision.can_trigger_policy);
    assert!(!decision.requires_corroboration);
    assert!(!decision.log_only);
    assert_eq!(decision.confidence_level, ConfidenceLevel::High);
}

#[test]
fn test_medium_confidence_requires_corroboration() {
    let integration = PolicyIntegration::new();
    
    // Medium confidence correlation (>= 0.5, < 0.8)
    let correlation = CorrelationResult {
        correlation_id: "test_corr".to_string(),
        ioc_value: "192.168.1.100".to_string(),
        ioc_type: "IP".to_string(),
        signals: vec![
            CorrelatedSignal {
                signal_id: "signal1".to_string(),
                source: SignalSource::Telemetry,
                ioc_value: "192.168.1.100".to_string(),
                ioc_type: "IP".to_string(),
                timestamp: Utc::now(),
                confidence: 0.6,
                metadata: serde_json::json!({}),
            },
        ],
        confidence_score: 0.65,
        source_agreement: 0.5,
        signal_frequency: 1,
        temporal_proximity: chrono::Duration::seconds(3600),
        correlated_at: Utc::now(),
    };
    
    let can_trigger = integration.can_trigger_policy(&correlation).unwrap();
    assert!(!can_trigger, "Medium confidence should NOT trigger policy without corroboration");
    
    let decision = integration.get_escalation_decision(&correlation).unwrap();
    assert!(!decision.can_trigger_policy);
    assert!(decision.requires_corroboration);
    assert!(!decision.log_only);
    assert_eq!(decision.confidence_level, ConfidenceLevel::Medium);
}

#[test]
fn test_low_confidence_logs_only() {
    let integration = PolicyIntegration::new();
    
    // Low confidence correlation (< 0.5)
    let correlation = CorrelationResult {
        correlation_id: "test_corr".to_string(),
        ioc_value: "192.168.1.100".to_string(),
        ioc_type: "IP".to_string(),
        signals: vec![
            CorrelatedSignal {
                signal_id: "signal1".to_string(),
                source: SignalSource::Telemetry,
                ioc_value: "192.168.1.100".to_string(),
                ioc_type: "IP".to_string(),
                timestamp: Utc::now(),
                confidence: 0.3,
                metadata: serde_json::json!({}),
            },
        ],
        confidence_score: 0.35,
        source_agreement: 0.2,
        signal_frequency: 1,
        temporal_proximity: chrono::Duration::seconds(7200),
        correlated_at: Utc::now(),
    };
    
    let can_trigger = integration.can_trigger_policy(&correlation).unwrap();
    assert!(!can_trigger, "Low confidence should NOT trigger policy");
    
    let decision = integration.get_escalation_decision(&correlation).unwrap();
    assert!(!decision.can_trigger_policy);
    assert!(!decision.requires_corroboration);
    assert!(decision.log_only);
    assert_eq!(decision.confidence_level, ConfidenceLevel::Low);
}

