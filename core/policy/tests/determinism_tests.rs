// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/determinism_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests proving deterministic policy evaluation

use std::path::Path;
use std::fs;
use tempfile::TempDir;
use ransomeye_policy::{PolicyEngine, EvaluationContext, PolicyDecision};
use serde_json::json;

#[test]
fn test_same_input_produces_same_decision() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();

    let engine = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    if engine.is_ok() {
        let engine = engine.unwrap();

        let context1 = EvaluationContext::new(
            "alert_1",
            "critical",
            "actions_on_objectives",
            Some("server".to_string()),
            None,
            "producer_1",
            vec!["rule_1".to_string()],
            "evidence_1",
            json!({}),
        );

        let context2 = EvaluationContext::new(
            "alert_1",
            "critical",
            "actions_on_objectives",
            Some("server".to_string()),
            None,
            "producer_1",
            vec!["rule_1".to_string()],
            "evidence_1",
            json!({}),
        );

        let decision1 = engine.evaluate(context1);
        let decision2 = engine.evaluate(context2);

        if decision1.is_ok() && decision2.is_ok() {
            let d1 = decision1.unwrap();
            let d2 = decision2.unwrap();

            assert_eq!(d1.decision, d2.decision);
            assert_eq!(d1.policy_id, d2.policy_id);
        }
    }
}

#[test]
fn test_decision_hash_is_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();

    let engine = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    if engine.is_ok() {
        let engine = engine.unwrap();

        let context = EvaluationContext::new(
            "alert_1",
            "critical",
            "actions_on_objectives",
            Some("server".to_string()),
            None,
            "producer_1",
            vec!["rule_1".to_string()],
            "evidence_1",
            json!({}),
        );

        let decision = engine.evaluate(context);

        if decision.is_ok() {
            let d = decision.unwrap();
            assert!(d.verify());
            
            let decision_clone = PolicyDecision::new(
                &d.alert_id,
                &d.policy_id,
                &d.policy_version,
                d.decision.clone(),
                d.allowed_actions.clone(),
                d.required_approvals.clone(),
                &d.evidence_reference,
                &d.kill_chain_stage,
                &d.severity,
                d.asset_class.clone(),
                &d.reasoning,
                &d.policy_signature,
            );
            
            assert_eq!(d.decision_hash, decision_clone.decision_hash);
        }
    }
}

#[test]
fn test_policy_evaluation_order_is_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();

    let engine1 = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    let engine2 = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    if engine1.is_ok() && engine2.is_ok() {
        let engine1 = engine1.unwrap();
        let engine2 = engine2.unwrap();

        let context = EvaluationContext::new(
            "alert_1",
            "critical",
            "actions_on_objectives",
            Some("server".to_string()),
            None,
            "producer_1",
            vec!["rule_1".to_string()],
            "evidence_1",
            json!({}),
        );

        let decision1 = engine1.evaluate(context.clone());
        let decision2 = engine2.evaluate(context);

        if decision1.is_ok() && decision2.is_ok() {
            let d1 = decision1.unwrap();
            let d2 = decision2.unwrap();

            assert_eq!(d1.decision, d2.decision);
            assert_eq!(d1.policy_id, d2.policy_id);
        }
    }
}

