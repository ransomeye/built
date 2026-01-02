// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/signature_tamper_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime tests proving signature tampering is detected and engine refuses to start

use std::path::Path;
use std::fs;
use tempfile::TempDir;
use ransomeye_policy::engine::PolicyEngine;

#[test]
fn test_tampered_signature_engine_refuses_to_start() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let policy_content = r#"
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
signature: "valid_signature_base64_here"
signature_hash: "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890"
"#;

    fs::write(policies_dir.join("test.yaml"), policy_content).unwrap();

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
fn test_tampered_hash_engine_refuses_to_start() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let policy_content = r#"
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
signature: "valid_signature_base64_here"
signature_hash: "tampered_hash_value_that_does_not_match_content"
"#;

    fs::write(policies_dir.join("test.yaml"), policy_content).unwrap();

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
fn test_invalid_signature_format_engine_refuses_to_start() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    let policy_content = r#"
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
signature: "not_valid_base64!!!"
signature_hash: "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890"
"#;

    fs::write(policies_dir.join("test.yaml"), policy_content).unwrap();

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

