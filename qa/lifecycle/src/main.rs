// Path and File Name : /home/ransomeye/rebuild/qa/lifecycle/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Lifecycle validation binary - runs full lifecycle validation suite and generates report

use lifecycle::LifecycleValidator;
use tracing::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting RansomEye Lifecycle Validation");

    // Get project root from environment or use default
    let project_root = env::var("RANSOMEYE_PROJECT_ROOT")
        .unwrap_or_else(|_| "/home/ransomeye/rebuild".to_string());

    // Create validator
    let validator = LifecycleValidator::new(project_root);

    // Run full validation suite
    let report = validator.run_full_validation().await;

    // Generate report
    let readiness_validator = lifecycle::readiness::ReadinessValidator::new(&validator);
    let report_path = readiness_validator.generate_report(report.clone()).await
        .map_err(|e| format!("Failed to generate report: {}", e))?;

    // Print summary
    println!("\n=== LIFECYCLE VALIDATION SUMMARY ===");
    println!("Overall Status: {}", report.overall_status);
    println!("GO/NO-GO: {}", report.go_no_go);
    println!("\nStage Results:");
    println!("  Install: {}", if report.install_result.passed { "PASS" } else { "FAIL" });
    println!("  Start: {}", if report.start_result.passed { "PASS" } else { "FAIL" });
    println!("  Stop: {}", if report.stop_result.passed { "PASS" } else { "FAIL" });
    println!("  Restart: {}", if report.restart_result.passed { "PASS" } else { "FAIL" });
    println!("  Upgrade: {}", if report.upgrade_result.passed { "PASS" } else { "FAIL" });
    println!("  Rollback: {}", if report.rollback_result.passed { "PASS" } else { "FAIL" });
    println!("  Uninstall: {}", if report.uninstall_result.passed { "PASS" } else { "FAIL" });
    println!("  Audit Continuity: {}", if report.audit_continuity_result.passed { "PASS" } else { "FAIL" });
    println!("  Trust Continuity: {}", if report.trust_continuity_result.passed { "PASS" } else { "FAIL" });
    
    println!("\nFailure Injection Results:");
    for (component, result) in &report.failure_injection_results {
        println!("  {}: {}", component, if result.passed { "PASS" } else { "FAIL" });
    }

    println!("\nStress Test Results:");
    for result in &report.stress_test_results {
        println!("  {}: {}", result.stage, if result.passed { "PASS" } else { "FAIL" });
    }

    println!("\nReport written to: {}", report_path);

    // Exit with appropriate code
    if report.overall_status == "PASS" && report.go_no_go == "GO" {
        info!("Lifecycle validation PASSED - System is GO for release");
        Ok(())
    } else {
        error!("Lifecycle validation FAILED - System is NO-GO for release");
        std::process::exit(1);
    }
}

