// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/real_policy_loading_test.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Test that real signed policies load and verify correctly

use ransomeye_policy::PolicyEngine;

#[test]
fn test_real_signed_policies_load() {
    let policies_path = "policies";
    let trust_store_path = "security/trust_store";
    
    let result = PolicyEngine::new(
        policies_path,
        "1.0.0",
        Some(trust_store_path),
        None,
        None,
    );
    
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Engine should initialize with real signed policies");
    let engine = result.unwrap();
    
    println!("✓ Engine initialized successfully");
    println!("✓ All policies loaded and signatures verified");
    assert!(engine.is_started());
}

