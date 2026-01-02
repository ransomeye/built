// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/precedence_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests for precedence rules

use ransomeye_policy::{PrecedenceRules, PolicyRule, AllowedAction, PolicyMatchCondition};

#[test]
fn test_precedence_higher_priority_wins() {
    let rules = PrecedenceRules::new();

    let policy1 = PolicyRule {
        id: "policy1".to_string(),
        version: "1.0.0".to_string(),
        priority: 50,
        match_conditions: vec![PolicyMatchCondition {
            field: "test".to_string(),
            operator: "equals".to_string(),
            value: serde_json::Value::String("test".to_string()),
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
        match_conditions: vec![],
        decision: AllowedAction::Deny,
        allowed_actions: vec![AllowedAction::Deny],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let ordering = rules.compare(&policy1, &policy2);
    assert!(matches!(ordering, std::cmp::Ordering::Greater));
}

#[test]
fn test_precedence_explicit_deny_wins() {
    let rules = PrecedenceRules::new();

    let policy1 = PolicyRule {
        id: "policy1".to_string(),
        version: "1.0.0".to_string(),
        priority: 100,
        match_conditions: vec![PolicyMatchCondition {
            field: "test".to_string(),
            operator: "equals".to_string(),
            value: serde_json::Value::String("test".to_string()),
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
            field: "test".to_string(),
            operator: "equals".to_string(),
            value: serde_json::Value::String("test".to_string()),
        }],
        decision: AllowedAction::Deny,
        allowed_actions: vec![AllowedAction::Deny],
        required_approvals: vec![],
        reasoning: "Test".to_string(),
    };

    let ordering = rules.compare(&policy1, &policy2);
    assert!(matches!(ordering, std::cmp::Ordering::Greater));
}

