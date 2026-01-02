// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/tests/ai_authority_violation_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Functional tests for AI authority violation detection - verifies AI cannot influence enforcement

use ransomeye_architecture_enforcement::BoundaryEnforcer;
use tempfile::TempDir;

#[test]
#[should_panic(expected = "abort")]
fn test_ai_enforcement_authority_blocked() {
    // Test that AI components cannot authorize enforcement
    // AI has zero enforcement authority
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let enforcer = BoundaryEnforcer::new(log_path).unwrap();
    
    // Attempt to authorize enforcement from AI component
    // This should abort
    let _ = enforcer.enforce_boundary_crossing(
        "ransomeye_ai_core",
        "ransomeye_response",
        None,
        "enforcement",
    );
}

#[test]
#[should_panic(expected = "abort")]
fn test_ai_control_plane_access_blocked() {
    // Test that AI components cannot send data to Control Plane
    // AI is non-authoritative and cannot influence Control Plane decisions
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let enforcer = BoundaryEnforcer::new(log_path).unwrap();
    
    // Attempt to access Control Plane from AI
    let _ = enforcer.enforce_boundary_crossing(
        "ransomeye_ai_assistant",
        "ransomeye_alert_engine",
        None,
        "api_access",
    );
}

#[test]
#[should_panic(expected = "abort")]
fn test_llm_enforcement_blocked() {
    // Test that LLM components cannot authorize enforcement
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let enforcer = BoundaryEnforcer::new(log_path).unwrap();
    
    // Attempt enforcement from LLM
    let _ = enforcer.enforce_boundary_crossing(
        "ransomeye_ai_assistant",
        "ransomeye_response",
        None,
        "enforcement",
    );
}

#[test]
#[should_panic(expected = "abort")]
fn test_threat_intel_enforcement_blocked() {
    // Test that Threat Intel Engine cannot authorize enforcement
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let enforcer = BoundaryEnforcer::new(log_path).unwrap();
    
    // Attempt enforcement from Threat Intel
    let _ = enforcer.enforce_boundary_crossing(
        "ransomeye_threat_intel_engine",
        "ransomeye_response",
        None,
        "enforcement",
    );
}

#[test]
fn test_ai_read_only_allowed() {
    // Test that AI can receive read-only data from Control Plane
    
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");
    let enforcer = BoundaryEnforcer::new(log_path).unwrap();
    
    // Control Plane → Intelligence Plane (read-only) should be allowed
    let result = enforcer.enforce_boundary_crossing(
        "ransomeye_alert_engine",
        "ransomeye_ai_core",
        None,
        "analysis_request",
    );
    
    assert!(result.is_ok(), "Control Plane → Intelligence Plane (read-only) should be allowed");
}
