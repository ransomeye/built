// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/conflict_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests for conflict detection and resolution

use ransomeye_policy::{ConflictDetector, ConflictResolver, PolicyRule, AllowedAction, PolicyMatchCondition, PolicyConflict, ConflictType, ConflictResolution};

#[test]
fn test_conflict_detection_same_priority() {
    let detector = ConflictDetector::new();

    let policy1 = PolicyRule {
        id: "policy1".to_string(),
        version: "1.0.0".to_string(),
        priority: 100,
        match_conditions: vec![PolicyMatchCondition {
            field: "alert_severity".to_string(),
            operator: "equals".to_string(),
            value: serde_json::Value::String("critical".to_string()),
        }],
        decision: AllowedAction::Allow,
        allowed_actions: vec![AllowedAction::Allow],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let policy2 = PolicyRule {
        id: "policy2".to_string(),
        version: "1.0.0".to_string(),
        priority: 100,
        match_conditions: vec![PolicyMatchCondition {
            field: "alert_severity".to_string(),
            operator: "equals".to_string(),
            value: serde_json::Value::String("critical".to_string()),
        }],
        decision: AllowedAction::Deny,
        allowed_actions: vec![AllowedAction::Deny],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let policies = vec![policy1, policy2];
    let conflicts = detector.detect_conflicts(&policies).unwrap();
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_resolution_explicit_deny() {
    let resolver = ConflictResolver::new();

    let policy1 = PolicyRule {
        id: "policy1".to_string(),
        version: "1.0.0".to_string(),
        priority: 100,
        match_conditions: vec![],
        decision: AllowedAction::Allow,
        allowed_actions: vec![AllowedAction::Allow],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let policy2 = PolicyRule {
        id: "policy2".to_string(),
        version: "1.0.0".to_string(),
        priority: 100,
        match_conditions: vec![],
        decision: AllowedAction::Deny,
        allowed_actions: vec![AllowedAction::Deny],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let conflict = PolicyConflict {
        policy_ids: vec!["policy1".to_string(), "policy2".to_string()],
        conflict_type: ConflictType::ContradictoryActions,
        resolution: None,
    };

    let policies = vec![policy1, policy2];
    let resolution = resolver.resolve(&conflict, &policies).unwrap();
    match resolution {
        ConflictResolution::UseExplicitDeny => {}
        _ => panic!("Expected UseExplicitDeny resolution"),
    }
}

