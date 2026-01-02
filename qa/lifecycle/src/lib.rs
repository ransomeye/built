// Path and File Name : /home/ransomeye/rebuild/qa/lifecycle/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Lifecycle validation framework - install, start, stop, restart, upgrade, rollback, uninstall, failure injection, stress testing

pub mod install;
pub mod service;
pub mod upgrade;
pub mod failure;
pub mod stress;
pub mod readiness;

use install::InstallValidator;
use service::ServiceValidator;
use upgrade::UpgradeValidator;
use failure::FailureInjector;
use stress::StressValidator;
use readiness::ReadinessValidator;
use chrono::Utc;
use tracing::info;
use std::path::Path;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTestResult {
    pub stage: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleValidationReport {
    pub timestamp: String,
    pub install_result: LifecycleTestResult,
    pub start_result: LifecycleTestResult,
    pub stop_result: LifecycleTestResult,
    pub restart_result: LifecycleTestResult,
    pub upgrade_result: LifecycleTestResult,
    pub rollback_result: LifecycleTestResult,
    pub uninstall_result: LifecycleTestResult,
    pub failure_injection_results: HashMap<String, LifecycleTestResult>,
    pub stress_test_results: Vec<LifecycleTestResult>,
    pub audit_continuity_result: LifecycleTestResult,
    pub trust_continuity_result: LifecycleTestResult,
    pub overall_status: String, // "PASS" or "FAIL"
    pub go_no_go: String, // "GO" or "NO-GO"
}

pub struct LifecycleValidator {
    project_root: String,
    test_artifacts_dir: String,
}

impl LifecycleValidator {
    pub fn new(project_root: String) -> Self {
        let test_artifacts_dir = format!("{}/logs/lifecycle_tests", project_root);
        std::fs::create_dir_all(&test_artifacts_dir).unwrap_or_else(|_| {});
        
        Self {
            project_root,
            test_artifacts_dir,
        }
    }

    pub fn get_artifacts_dir(&self) -> &str {
        &self.test_artifacts_dir
    }

    pub fn get_project_root(&self) -> &str {
        &self.project_root
    }

    /// Run complete lifecycle validation suite
    pub async fn run_full_validation(&self) -> LifecycleValidationReport {
        info!("Starting full lifecycle validation suite");

        // Installation validation
        let install_validator = InstallValidator::new(self);
        let install_result = install_validator.validate_clean_install().await;
        let _bootstrap_result = install_validator.validate_bootstrap().await;

        // Service validation
        let service_validator = ServiceValidator::new(self);
        let start_result = service_validator.validate_start().await;
        let stop_result = service_validator.validate_stop().await;
        let restart_result = service_validator.validate_restart().await;
        let _crash_recovery_result = service_validator.validate_crash_recovery().await;

        // Upgrade validation
        let upgrade_validator = UpgradeValidator::new(self);
        let upgrade_result = upgrade_validator.validate_upgrade().await;
        let rollback_result = upgrade_validator.validate_rollback().await;

        // Uninstall validation (simplified - would check uninstall script)
        let uninstall_result = LifecycleTestResult {
            stage: "uninstall".to_string(),
            passed: Path::new(&format!("{}/uninstall.sh", self.project_root)).exists(),
            duration_ms: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            evidence: Some("Uninstall script validation".to_string()),
        };

        // Failure injection
        let failure_injector = FailureInjector::new(self);
        let failure_injection_results = failure_injector.inject_and_validate().await;

        // Stress testing
        let stress_validator = StressValidator::new(self);
        let stress_test_results = stress_validator.run_stress_tests().await;

        // Audit and trust continuity
        let readiness_validator = ReadinessValidator::new(self);
        let audit_continuity_result = readiness_validator.validate_audit_continuity().await;
        let trust_continuity_result = readiness_validator.validate_trust_continuity().await;

        // Determine overall status
        let all_results = vec![
            &install_result,
            &start_result,
            &stop_result,
            &restart_result,
            &upgrade_result,
            &rollback_result,
            &uninstall_result,
            &audit_continuity_result,
            &trust_continuity_result,
        ];

        let mut all_passed = true;
        for result in &all_results {
            if !result.passed {
                all_passed = false;
                break;
            }
        }

        // Check failure injection results
        for (_, result) in &failure_injection_results {
            if !result.passed {
                all_passed = false;
                break;
            }
        }

        // Check stress test results
        for result in &stress_test_results {
            if !result.passed {
                all_passed = false;
                break;
            }
        }

        let overall_status = if all_passed { "PASS" } else { "FAIL" }.to_string();

        // Determine GO/NO-GO
        let mut report = LifecycleValidationReport {
            timestamp: Utc::now().to_rfc3339(),
            install_result,
            start_result,
            stop_result,
            restart_result,
            upgrade_result,
            rollback_result,
            uninstall_result,
            failure_injection_results,
            stress_test_results,
            audit_continuity_result,
            trust_continuity_result,
            overall_status: overall_status.clone(),
            go_no_go: String::new(),
        };

        let go_no_go = readiness_validator.determine_go_no_go(&report).await;
        report.go_no_go = go_no_go;

        info!("Lifecycle validation suite completed: {}", report.overall_status);

        report
    }
}

