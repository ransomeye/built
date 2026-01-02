// Path and File Name : /home/ransomeye/rebuild/core/policy/tests/version_rollback_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for policy version rollback prevention

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use policy::engine::policy::PolicyLoader;

#[test]
fn test_lower_policy_version_load_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();
    
    let version_state_path = temp_dir.path().join("versions.json");
    std::env::set_var("RANSOMEYE_POLICY_VERSION_STATE_PATH", version_state_path.to_str().unwrap());
    
    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();
    
    // Create policy v1.0.0
    let policy_v1 = r#"
id: test_policy
version: "1.0.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v1"
signature_hash: "test_hash_v1"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v1).unwrap();
    
    // Load v1.0.0 (should succeed)
    let mut loader1 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    
    // Now try to load v0.9.0 (should be rejected)
    let policy_v09 = r#"
id: test_policy
version: "0.9.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v09"
signature_hash: "test_hash_v09"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v09).unwrap();
    
    // Loading should fail due to version rollback
    let loader2 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    
    assert!(loader2.is_err());
    if let Err(e) = loader2 {
        assert!(format!("{:?}", e).contains("rollback") || format!("{:?}", e).contains("version"));
    }
}

#[test]
fn test_first_run_bootstrap_allows_any_version() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();
    
    let version_state_path = temp_dir.path().join("versions.json");
    std::env::set_var("RANSOMEYE_POLICY_VERSION_STATE_PATH", version_state_path.to_str().unwrap());
    
    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();
    
    // Create policy v1.0.0 - first run (clean state)
    let policy_v1 = r#"
id: test_policy
version: "1.0.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v1"
signature_hash: "test_hash_v1"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v1).unwrap();
    
    // First run: Clean state → should succeed (bootstrap)
    let loader1 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    
    assert!(loader1.is_ok(), "First-run bootstrap should allow version 1.0.0");
    
    // Verify version state was persisted
    assert!(version_state_path.exists(), "Version state file should be created");
    let content = fs::read_to_string(&version_state_path).unwrap();
    assert!(content.contains("test_policy"), "Version state should contain policy ID");
    assert!(content.contains("1.0.0"), "Version state should contain version 1.0.0");
}

#[test]
fn test_same_version_replay_fails() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();
    
    let version_state_path = temp_dir.path().join("versions.json");
    std::env::set_var("RANSOMEYE_POLICY_VERSION_STATE_PATH", version_state_path.to_str().unwrap());
    
    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();
    
    // Create policy v1.0.0
    let policy_v1 = r#"
id: test_policy
version: "1.0.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v1"
signature_hash: "test_hash_v1"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v1).unwrap();
    
    // First run: Bootstrap (should succeed)
    let loader1 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    assert!(loader1.is_ok(), "First run should succeed");
    
    // Second run: Same version → should fail (rollback protection)
    let loader2 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    
    assert!(loader2.is_err(), "Same version replay should fail");
    if let Err(e) = loader2 {
        assert!(format!("{:?}", e).to_lowercase().contains("rollback") || 
                format!("{:?}", e).to_lowercase().contains("not greater"), 
                "Error should mention rollback or version comparison");
    }
}

#[test]
fn test_lower_version_fails() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();
    
    let version_state_path = temp_dir.path().join("versions.json");
    std::env::set_var("RANSOMEYE_POLICY_VERSION_STATE_PATH", version_state_path.to_str().unwrap());
    
    let trust_dir = temp_dir.path().join("trust");
    fs::create_dir_all(&trust_dir).unwrap();
    
    // Create policy v1.0.0 - first run
    let policy_v1 = r#"
id: test_policy
version: "1.0.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v1"
signature_hash: "test_hash_v1"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v1).unwrap();
    
    // First run: Bootstrap (should succeed)
    let loader1 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    assert!(loader1.is_ok(), "First run should succeed");
    
    // Now try to load v0.9.0 (should fail - lower version)
    let policy_v09 = r#"
id: test_policy
version: "0.9.0"
name: Test Policy
description: Test
enabled: true
priority: 1
match_conditions: []
decision:
  action: deny
  allowed_actions: [deny]
  reasoning: Test
required_approvals: []
signature: "test_sig_v09"
signature_hash: "test_hash_v09"
"#;
    
    fs::write(policies_dir.join("policy.yaml"), policy_v09).unwrap();
    
    let loader2 = PolicyLoader::new(
        policies_dir.to_str().unwrap(),
        Some(trust_dir.to_str().unwrap())
    );
    
    assert!(loader2.is_err(), "Lower version should fail");
    if let Err(e) = loader2 {
        assert!(format!("{:?}", e).to_lowercase().contains("rollback") || 
                format!("{:?}", e).to_lowercase().contains("not greater"), 
                "Error should mention rollback or version comparison");
    }
}

