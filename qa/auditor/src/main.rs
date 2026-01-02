// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/src/main.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main validation orchestrator - coordinates all validation suites and generates release decision

use std::path::PathBuf;
use std::time::Instant;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use thiserror::Error;

pub mod core;
pub mod chaos;
pub mod replay;
pub mod verifier;
pub mod auditor;
pub mod release_gate;
mod suites;

use crate::core::{Finding, Severity, ValidationResult as CoreValidationResult};
use crate::release_gate::{ReleaseGate, Decision as ReleaseGateDecision};
use suites::{
    security::SecuritySuite,
    performance::PerformanceSuite,
    stress::StressSuite,
    fault_injection::FaultInjectionSuite,
    compliance::ComplianceSuite,
    regression::RegressionSuite,
    lifecycle::LifecycleSuite,
};

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Security validation failed: {0}")]
    SecurityFailed(String),
    #[error("Performance validation failed: {0}")]
    PerformanceFailed(String),
    #[error("Stress validation failed: {0}")]
    StressFailed(String),
    #[error("Fault injection validation failed: {0}")]
    FaultInjectionFailed(String),
    #[error("Compliance validation failed: {0}")]
    ComplianceFailed(String),
    #[error("Regression validation failed: {0}")]
    RegressionFailed(String),
    #[error("Lifecycle validation failed: {0}")]
    LifecycleFailed(String),
    #[error("Report generation failed: {0}")]
    ReportFailed(String),
}

// Suite result wrapper for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub result: String, // "Pass", "Hold", or "Fail"
    pub findings: Vec<Finding>,
    pub timestamp: chrono::DateTime<Utc>,
}

// ReleaseDecision is now defined in release_gate module
// This local type is kept for backward compatibility with existing code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalReleaseDecision {
    pub decision: Decision,
    pub timestamp: chrono::DateTime<Utc>,
    pub suite_results: Vec<SuiteResult>,
    pub justification: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Block,
    Hold,
}

pub struct ValidationOrchestrator {
    reports_dir: PathBuf,
}

impl ValidationOrchestrator {
    pub fn new(reports_dir: PathBuf) -> Self {
        Self { reports_dir }
    }

    pub async fn run_full_validation(&self) -> Result<LocalReleaseDecision, ValidationError> {
        info!("Starting full validation suite");
        let start_time = Instant::now();
        
        let mut results: Vec<(String, CoreValidationResult)> = Vec::new();
        
        // Security validation
        info!("Running security validation suite");
        let security_result = SecuritySuite::new().run().await
            .map_err(|e| ValidationError::SecurityFailed(e.to_string()))?;
        results.push(("security".to_string(), security_result));
        
        // Performance validation
        info!("Running performance validation suite");
        let perf_result = PerformanceSuite::new().run().await
            .map_err(|e| ValidationError::PerformanceFailed(e.to_string()))?;
        results.push(("performance".to_string(), perf_result));
        
        // Stress validation
        info!("Running stress validation suite");
        let stress_result = StressSuite::new().run().await
            .map_err(|e| ValidationError::StressFailed(e.to_string()))?;
        results.push(("stress".to_string(), stress_result));
        
        // Fault injection validation
        info!("Running fault injection validation suite");
        let fault_result = FaultInjectionSuite::new().run().await
            .map_err(|e| ValidationError::FaultInjectionFailed(e.to_string()))?;
        results.push(("fault_injection".to_string(), fault_result));
        
        // Compliance validation
        info!("Running compliance validation suite");
        let compliance_result = ComplianceSuite::new().run().await
            .map_err(|e| ValidationError::ComplianceFailed(e.to_string()))?;
        results.push(("compliance".to_string(), compliance_result));
        
        // Regression validation
        info!("Running regression validation suite");
        let regression_result = RegressionSuite::new().run().await
            .map_err(|e| ValidationError::RegressionFailed(e.to_string()))?;
        results.push(("regression".to_string(), regression_result));
        
        // Lifecycle validation
        info!("Running lifecycle validation suite");
        let lifecycle_result = LifecycleSuite::new().run().await
            .map_err(|e| ValidationError::LifecycleFailed(e.to_string()))?;
        results.push(("lifecycle".to_string(), lifecycle_result));
        
        let duration = start_time.elapsed();
        info!("Validation completed in {:?}", duration);
        
        // Generate release decision with fail-closed semantics
        let decision = self.generate_decision(&results)?;
        
        // Generate reports
        self.generate_reports(&results, &decision).await?;
        
        Ok(decision)
    }
    
    fn generate_decision(&self, results: &[(String, CoreValidationResult)]) -> Result<LocalReleaseDecision, ValidationError> {
        // Fail-closed semantics: Default is BLOCK, explicit ALLOW only if no Fail, no High, no Critical
        let mut has_fail = false;
        let mut has_high_or_critical = false;
        let mut failed_suites = Vec::new();
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut suite_results = Vec::new();
        
        for (suite_name, result) in results {
            let result_str = match result {
                CoreValidationResult::Pass(_) => "Pass",
                CoreValidationResult::Hold(_) => "Hold",
                CoreValidationResult::Fail(_) => "Fail",
            };
            
            if result.is_fail() {
                has_fail = true;
                failed_suites.push(suite_name.clone());
            }
            
            let critical_findings = result.critical_findings();
            if !critical_findings.is_empty() {
                has_high_or_critical = true;
                for finding in critical_findings {
                    match finding.severity {
                        Severity::Critical => critical_count += 1,
                        Severity::High => high_count += 1,
                        _ => {}
                    }
                }
            }
            
            suite_results.push(SuiteResult {
                suite_name: suite_name.clone(),
                result: result_str.to_string(),
                findings: result.findings().to_vec(),
                timestamp: Utc::now(),
            });
        }
        
        // Fail-closed decision logic
        let decision = if has_fail || has_high_or_critical {
            Decision::Block
        } else {
            // Only ALLOW if no Fail results and no High/Critical findings
            Decision::Allow
        };
        
        let justification = if decision == Decision::Block {
            if has_fail {
                format!("Release BLOCKED: {} suite(s) failed: {}", 
                    failed_suites.len(), 
                    failed_suites.join(", "))
            } else {
                format!("Release BLOCKED: {} critical, {} high severity findings found", 
                    critical_count, high_count)
            }
        } else {
            "All validation suites passed. No failures, no critical or high severity findings.".to_string()
        };
        
        Ok(LocalReleaseDecision {
            decision,
            timestamp: Utc::now(),
            suite_results,
            justification,
            signature: None, // Will be signed by auditor
        })
    }
    
    async fn generate_reports(
        &self,
        results: &[(String, CoreValidationResult)],
        decision: &LocalReleaseDecision,
    ) -> Result<(), ValidationError> {
        use std::fs;
        use std::io::Write;
        
        // Generate security report
        let security_report = self.generate_security_report(results)?;
        let security_path = self.reports_dir.join("security_report.md");
        fs::File::create(&security_path)
            .and_then(|mut f| f.write_all(security_report.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write security report: {}", e)))?;
        
        // Generate performance report
        let perf_report = self.generate_performance_report(results)?;
        let perf_path = self.reports_dir.join("performance_report.md");
        fs::File::create(&perf_path)
            .and_then(|mut f| f.write_all(perf_report.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write performance report: {}", e)))?;
        
        // Generate stress report
        let stress_report = self.generate_stress_report(results)?;
        let stress_path = self.reports_dir.join("stress_report.md");
        fs::File::create(&stress_path)
            .and_then(|mut f| f.write_all(stress_report.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write stress report: {}", e)))?;
        
        // Generate compliance report
        let compliance_report = self.generate_compliance_report(results)?;
        let compliance_path = self.reports_dir.join("compliance_report.md");
        fs::File::create(&compliance_path)
            .and_then(|mut f| f.write_all(compliance_report.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write compliance report: {}", e)))?;
        
        // Generate release decision report
        let decision_report = self.generate_decision_report(decision)?;
        let decision_path = self.reports_dir.join("release_decision.md");
        fs::File::create(&decision_path)
            .and_then(|mut f| f.write_all(decision_report.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write release decision: {}", e)))?;
        
        // Generate JSON summary
        let json_summary = serde_json::to_string_pretty(decision)
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to serialize decision: {}", e)))?;
        let json_path = self.reports_dir.join("release_decision.json");
        fs::File::create(&json_path)
            .and_then(|mut f| f.write_all(json_summary.as_bytes()))
            .map_err(|e| ValidationError::ReportFailed(format!("Failed to write JSON summary: {}", e)))?;
        
        info!("All reports generated in {:?}", self.reports_dir);
        Ok(())
    }
    
    fn generate_security_report(&self, results: &[(String, CoreValidationResult)]) -> Result<String, ValidationError> {
        let mut report = String::new();
        report.push_str("# Security Validation Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
        
        if let Some((_, security_result)) = results.iter().find(|(name, _)| name == "security") {
            let status = match security_result {
                CoreValidationResult::Pass(_) => "PASS",
                CoreValidationResult::Hold(_) => "HOLD",
                CoreValidationResult::Fail(_) => "FAIL",
            };
            report.push_str(&format!("## Status: {}\n\n", status));
            
            let findings = security_result.findings();
            if !findings.is_empty() {
                report.push_str("## Findings\n\n");
                for finding in findings {
                    report.push_str(&format!("### {:?}\n", finding.severity));
                    report.push_str(&format!("**Description:** {}\n\n", finding.description));
                }
            }
        }
        
        Ok(report)
    }
    
    fn generate_performance_report(&self, results: &[(String, CoreValidationResult)]) -> Result<String, ValidationError> {
        let mut report = String::new();
        report.push_str("# Performance Validation Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
        
        if let Some((_, perf_result)) = results.iter().find(|(name, _)| name == "performance") {
            let status = match perf_result {
                CoreValidationResult::Pass(_) => "PASS",
                CoreValidationResult::Hold(_) => "HOLD",
                CoreValidationResult::Fail(_) => "FAIL",
            };
            report.push_str(&format!("## Status: {}\n\n", status));
            
            let findings = perf_result.findings();
            if !findings.is_empty() {
                report.push_str("## Metrics\n\n");
                for finding in findings {
                    report.push_str(&format!("- **{:?}:** {}\n", finding.severity, finding.description));
                }
            }
        }
        
        Ok(report)
    }
    
    fn generate_stress_report(&self, results: &[(String, CoreValidationResult)]) -> Result<String, ValidationError> {
        let mut report = String::new();
        report.push_str("# Stress Validation Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
        
        if let Some((_, stress_result)) = results.iter().find(|(name, _)| name == "stress") {
            let status = match stress_result {
                CoreValidationResult::Pass(_) => "PASS",
                CoreValidationResult::Hold(_) => "HOLD",
                CoreValidationResult::Fail(_) => "FAIL",
            };
            report.push_str(&format!("## Status: {}\n\n", status));
            
            let findings = stress_result.findings();
            if !findings.is_empty() {
                report.push_str("## Stress Test Results\n\n");
                for finding in findings {
                    report.push_str(&format!("- **{:?}:** {}\n", finding.severity, finding.description));
                }
            }
        }
        
        Ok(report)
    }
    
    fn generate_compliance_report(&self, results: &[(String, CoreValidationResult)]) -> Result<String, ValidationError> {
        let mut report = String::new();
        report.push_str("# Compliance Validation Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
        
        if let Some((_, compliance_result)) = results.iter().find(|(name, _)| name == "compliance") {
            let status = match compliance_result {
                CoreValidationResult::Pass(_) => "PASS",
                CoreValidationResult::Hold(_) => "HOLD",
                CoreValidationResult::Fail(_) => "FAIL",
            };
            report.push_str(&format!("## Status: {}\n\n", status));
            
            let findings = compliance_result.findings();
            if !findings.is_empty() {
                report.push_str("## Compliance Checks\n\n");
                for finding in findings {
                    report.push_str(&format!("- **{:?}:** {}\n", finding.severity, finding.description));
                }
            }
        }
        
        Ok(report)
    }
    
    fn generate_decision_report(&self, decision: &LocalReleaseDecision) -> Result<String, ValidationError> {
        let mut report = String::new();
        report.push_str("# Release Decision Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", decision.timestamp.to_rfc3339()));
        report.push_str(&format!("## Decision: {:?}\n\n", decision.decision));
        report.push_str(&format!("## Justification\n\n{}\n\n", decision.justification));
        
        report.push_str("## Validation Summary\n\n");
        for suite_result in &decision.suite_results {
            report.push_str(&format!("- **{}:** {}\n", suite_result.suite_name, suite_result.result));
        }
        
        report.push_str("\n---\n");
        report.push_str("© RansomEye.Tech | Support: Gagan@RansomEye.Tech\n");
        
        Ok(report)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let project_root = PathBuf::from("/home/ransomeye/rebuild");
    let reports_dir = project_root.join("ransomeye_validation/reports");
    std::fs::create_dir_all(&reports_dir)?;
    
    // Run validation suites first
    let orchestrator = ValidationOrchestrator::new(reports_dir.clone());
    let _validation_decision = orchestrator.run_full_validation().await?;
    
    // Run Release Gate (FINAL AUTHORITY)
    info!("Invoking Release Gate (FINAL AUTHORITY)");
    let release_gate = ReleaseGate::new(project_root, reports_dir.clone())?;
    let release_decision = release_gate.evaluate().await?;
    
    // Print decision summary
    println!("========================================");
    println!("RELEASE GATE DECISION: {:?}", release_decision.decision);
    println!("========================================");
    println!("Justification: {}", release_decision.justification);
    println!("\nSuite Results:");
    for suite in &release_decision.suite_results {
        println!("  - {}: {}", suite.suite_name, suite.result);
    }
    
    if !release_decision.blocking_issues.is_empty() {
        println!("\nBlocking Issues:");
        for issue in &release_decision.blocking_issues {
            println!("  - {}", issue);
        }
    }
    
    println!("\nArtifacts Verified: {}", release_decision.artifacts_verified.len());
    println!("========================================");
    
        // Exit code based on decision (FAIL-CLOSED)
        match release_decision.decision {
            ReleaseGateDecision::Allow => {
                println!("✓ Release ALLOWED");
                Ok(())
            }
            ReleaseGateDecision::Block => {
                eprintln!("✗ Release BLOCKED");
                std::process::exit(1)
            }
            ReleaseGateDecision::Hold => {
                eprintln!("⚠ Release HOLD");
                std::process::exit(2)
            }
        }
}

