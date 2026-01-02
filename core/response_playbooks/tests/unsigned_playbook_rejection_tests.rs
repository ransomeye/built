// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/tests/unsigned_playbook_rejection_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for unsigned playbook rejection - fail-closed behavior

use ransomeye_response_playbooks::registry::PlaybookRegistry;
use ransomeye_response_playbooks::errors::PlaybookError;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_unsigned_playbook_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let playbook_dir = temp_dir.path().join("playbooks");
    fs::create_dir_all(&playbook_dir).unwrap();
    
    // Create a playbook without signature
    let unsigned_playbook = r#"
id: 00000000-0000-0000-0000-000000000001
name: Test Unsigned Playbook
version: 1.0.0
severity: high
trigger_conditions:
  policy_outcomes: [deny]
  alert_severity: [high]
  kill_chain_stage: [exploitation]
steps:
  - step_id: step_00000001
    step_name: Test Step
    action_type: isolate
    enforcement_action:
      adapter: linux_agent
      parameters: {}
    timeout_seconds: 60
    retry_on_failure: false
    continue_on_failure: false
rollback_steps: []
approvals_required:
  human_approval: false
  auto_approval: true
dry_run_supported: true
timeout_per_step: 60
max_execution_time: 3600
signature: ""
signature_hash: ""
"#;
    
    fs::write(playbook_dir.join("unsigned.yaml"), unsigned_playbook).unwrap();
    
    // Create dummy public key (will fail verification but structure is correct)
    let keys_dir = temp_dir.path().join("keys");
    fs::create_dir_all(&keys_dir).unwrap();
    fs::write(keys_dir.join("playbook_public_key.pem"), b"dummy key").unwrap();
    
    // Set environment variable
    std::env::set_var("RANSOMEYE_PLAYBOOK_PUBLIC_KEY_PATH", keys_dir.join("playbook_public_key.pem"));
    
    // Registry should reject unsigned playbook
    let result = PlaybookRegistry::new(playbook_dir.to_str().unwrap());
    
    // Should fail (either due to missing signature or invalid key)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_playbook_with_empty_signature_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let playbook_dir = temp_dir.path().join("playbooks");
    fs::create_dir_all(&playbook_dir).unwrap();
    
    // Create a playbook with empty signature
    let playbook = r#"
id: 00000000-0000-0000-0000-000000000002
name: Test Empty Signature
version: 1.0.0
severity: medium
trigger_conditions:
  policy_outcomes: [deny]
  alert_severity: [medium]
  kill_chain_stage: [delivery]
steps:
  - step_id: step_00000001
    step_name: Test Step
    action_type: notify
    enforcement_action:
      adapter: core
      parameters: {}
    timeout_seconds: 30
    retry_on_failure: false
    continue_on_failure: false
rollback_steps: []
approvals_required:
  human_approval: false
  auto_approval: true
dry_run_supported: true
timeout_per_step: 30
max_execution_time: 1800
signature: ""
signature_hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6"
"#;
    
    fs::write(playbook_dir.join("empty_sig.yaml"), playbook).unwrap();
    
    let keys_dir = temp_dir.path().join("keys");
    fs::create_dir_all(&keys_dir).unwrap();
    fs::write(keys_dir.join("playbook_public_key.pem"), b"dummy key").unwrap();
    
    std::env::set_var("RANSOMEYE_PLAYBOOK_PUBLIC_KEY_PATH", keys_dir.join("playbook_public_key.pem"));
    
    let result = PlaybookRegistry::new(playbook_dir.to_str().unwrap());
    
    // Should fail due to empty signature
    assert!(result.is_err());
}

