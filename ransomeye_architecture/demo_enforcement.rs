// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/demo_enforcement.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Demonstration script showing Phase 2 runtime enforcement in action

use ransomeye_architecture_enforcement::BoundaryEnforcer;
use std::path::PathBuf;

fn main() {
    println!("=== Phase 2 Runtime Enforcement Demonstration ===\n");
    
    let audit_log = PathBuf::from("/tmp/ransomeye_audit_demo.log");
    let enforcer = BoundaryEnforcer::new(audit_log.clone())
        .expect("Failed to create boundary enforcer");
    
    println!("1. Testing ALLOWED flow: Data Plane → Core");
    match enforcer.enforce_boundary_crossing(
        "ransomeye_dpi_probe",
        "ransomeye_master_core",
        None,
        "telemetry",
    ) {
        Ok(()) => println!("   ✅ ALLOWED: Data Plane → Core (telemetry) - PASSED\n"),
        Err(e) => println!("   ❌ ERROR: {}", e),
    }
    
    println!("2. Testing ALLOWED flow: Control Plane → Intelligence Plane");
    match enforcer.enforce_boundary_crossing(
        "ransomeye_alert_engine",
        "ransomeye_ai_core",
        None,
        "analysis_request",
    ) {
        Ok(()) => println!("   ✅ ALLOWED: Control Plane → Intelligence Plane (read-only) - PASSED\n"),
        Err(e) => println!("   ❌ ERROR: {}", e),
    }
    
    println!("3. Testing FORBIDDEN flow: AI → Control Plane");
    println!("   This should ABORT the process...");
    match enforcer.enforce_boundary_crossing(
        "ransomeye_ai_core",
        "ransomeye_alert_engine",
        None,
        "api_call",
    ) {
        Ok(()) => println!("   ❌ ERROR: Should have been blocked!"),
        Err(e) => {
            println!("   ✅ BLOCKED: {}", e);
            println!("   (Process would abort in production)\n");
        }
    }
    
    println!("4. Testing FORBIDDEN flow: Data Plane → Policy Engine");
    println!("   This should ABORT the process...");
    match enforcer.enforce_boundary_crossing(
        "ransomeye_dpi_probe",
        "ransomeye_alert_engine",
        None,
        "policy_access",
    ) {
        Ok(()) => println!("   ❌ ERROR: Should have been blocked!"),
        Err(e) => {
            println!("   ✅ BLOCKED: {}", e);
            println!("   (Process would abort in production)\n");
        }
    }
    
    println!("5. Testing FAIL-CLOSED: Unknown component");
    println!("   This should ABORT the process...");
    match enforcer.enforce_boundary_crossing(
        "unknown_component",
        "ransomeye_alert_engine",
        None,
        "any_operation",
    ) {
        Ok(()) => println!("   ❌ ERROR: Should have aborted on unknown component!"),
        Err(e) => {
            println!("   ✅ FAIL-CLOSED: Unknown component detected");
            println!("   (Process would abort in production)\n");
        }
    }
    
    println!("=== Demonstration Complete ===");
    println!("Audit log written to: {:?}", audit_log);
}

