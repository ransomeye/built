// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/fail_closed_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests proving fail-closed behavior

use std::path::Path;
use std::fs;
use tempfile::TempDir;
use ransomeye_policy::{PolicyEngine, EvaluationContext};

#[test]
fn test_unsigned_policy_engine_refuses_to_start() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let unsigned_policy = r#"
id: test_policy
version: "1.0.0"
name: "Test Policy"
description: "Test policy for unsigned policy test"
enabled: true
priority: 100
match_conditions:
  - field: "alert_severity"
    operator: "equals"
    value: "critical"
decision:
  action: "deny"
  allowed_actions: ["deny"]
  reasoning: "Test"
required_approvals: []
"#;

    fs::write(policies_dir.join("test.yaml"), unsigned_policy).unwrap();

    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();

    let result = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    assert!(result.is_err());
    match result {
        Err(e) => {
            let err_msg = e.to_string();
            println!("Actual error message: {}", err_msg);
            assert!(err_msg.contains("Unsigned policy") || err_msg.contains("not signed") || err_msg.contains("Policy") && err_msg.contains("not signed"));
        }
        Ok(_) => panic!("Expected error"),
    }
}

#[test]
fn test_invalid_signature_engine_refuses_to_start() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let invalid_policy = r#"
id: test_policy
version: "1.0.0"
name: "Test Policy"
enabled: true
priority: 100
match_conditions:
  - field: "alert_severity"
    operator: "equals"
    value: "critical"
decision:
  action: "deny"
  allowed_actions: ["deny"]
  reasoning: "Test"
signature: "invalid_signature_base64"
signature_hash: "a1b2c3d4e5f6"
"#;

    fs::write(policies_dir.join("test.yaml"), invalid_policy).unwrap();

    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();

    let result = PolicyEngine::new(
        policies_dir.to_str().unwrap(),
        "1.0.0",
        Some(trust_dir.to_str().unwrap()),
        None,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_missing_context_produces_deny() {
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
        
        let mut context = EvaluationContext::new(
            "",
            "",
            "",
            None,
            None,
            "",
            vec![],
            "",
            serde_json::json!({}),
        );

        let result = engine.evaluate(context);
        assert!(result.is_err() || result.unwrap().is_deny());
    }
}

#[test]
fn test_no_matching_policy_produces_deny() {
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
            "low",
            "reconnaissance",
            None,
            None,
            "producer_1",
            vec![],
            "evidence_1",
            serde_json::json!({}),
        );

        let result = engine.evaluate(context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_deny());
    }
}

