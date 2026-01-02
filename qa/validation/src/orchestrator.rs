// Path and File Name : /home/ransomeye/rebuild/qa/validation/src/orchestrator.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: System-wide validation orchestrator - coordinates all validation suites

use tracing::{info, error, warn};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::errors::{ValidationError, ValidationResult};
use crate::contract_integrity::{ContractIntegrityValidator, ContractIntegrityResult};
use crate::cryptographic_continuity::{CryptographicContinuityValidator, CryptographicContinuityResult};
use crate::determinism_replay::{DeterminismValidator, DeterminismResult};
use crate::failure_isolation::{FailureIsolationValidator, FailureIsolationResult};
use crate::resource_ceilings::{ResourceCeilingValidator, ResourceCeilingResult, BackpressureMetrics};
use crate::advisory_boundary::{AdvisoryBoundaryValidator, AdvisoryBoundaryResult};
use crate::reports::{ValidationReport, DeterminismProof, TrustChainVerification, FailureIsolationMatrix, ResourceCeilingVerification, GoNoGoDecision};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemValidationResult {
    pub contract_integrity: ContractIntegrityResult,
    pub cryptographic_continuity: CryptographicContinuityResult,
    pub determinism_replay: DeterminismResult,
    pub failure_isolation: FailureIsolationResult,
    pub resource_ceilings: ResourceCeilingResult,
    pub advisory_boundary: AdvisoryBoundaryResult,
    pub timestamp: chrono::DateTime<Utc>,
    pub overall_decision: GoNoGoDecision,
}

pub struct SystemValidator {
    output_dir: PathBuf,
}

impl SystemValidator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
    
    /// Run complete system-wide validation suite
    pub async fn run_full_validation(&self) -> ValidationResult<SystemValidationResult> {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("RansomEye System-Wide Validation, Integration & Trust Continuity");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| ValidationError::Internal(format!("Failed to create output directory: {}", e)))?;
        
        let start_time = Utc::now();
        
        // 1. Contract Integrity Validation
        info!("[1/6] Running Contract Integrity validation...");
        let contract_validator = ContractIntegrityValidator::new();
        let contract_result = contract_validator.run_validation_suite().await
            .map_err(|e| ValidationError::ContractIntegrity(format!("Contract integrity validation failed: {}", e)))?;
        info!("Contract Integrity: {} violations", contract_result.violations.len());
        
        // 2. Cryptographic Continuity Validation
        info!("[2/6] Running Cryptographic Continuity validation...");
        // Note: In real implementation, would load actual public keys
        let crypto_validator = CryptographicContinuityValidator::new(vec![]);
        let crypto_result = crypto_validator.run_validation_suite().await
            .map_err(|e| ValidationError::CryptographicContinuity(format!("Cryptographic continuity validation failed: {}", e)))?;
        info!("Cryptographic Continuity: {} violations", crypto_result.violations.len());
        
        // 3. Determinism & Replay Validation
        info!("[3/6] Running Determinism & Replay validation...");
        let mut determinism_validator = DeterminismValidator::new();
        let determinism_result = determinism_validator.run_validation_suite().await
            .map_err(|e| ValidationError::Determinism(format!("Determinism validation failed: {}", e)))?;
        info!("Determinism & Replay: {} violations", determinism_result.violations.len());
        
        // 4. Failure Isolation Validation
        info!("[4/6] Running Failure Isolation validation...");
        let failure_validator = FailureIsolationValidator::new();
        let failure_result = failure_validator.run_validation_suite().await
            .map_err(|e| ValidationError::FailureIsolation(format!("Failure isolation validation failed: {}", e)))?;
        info!("Failure Isolation: {} violations", failure_result.violations.len());
        
        // 5. Resource Ceilings Validation
        info!("[5/6] Running Resource Ceilings validation...");
        let resource_validator = ResourceCeilingValidator::new();
        let resource_result = resource_validator.run_validation_suite().await
            .map_err(|e| ValidationError::ResourceCeiling(format!("Resource ceiling validation failed: {}", e)))?;
        info!("Resource Ceilings: {} violations", resource_result.violations.len());
        
        // 6. Advisory Boundary Proof Validation
        info!("[6/6] Running Advisory Boundary Proof validation...");
        let advisory_validator = AdvisoryBoundaryValidator::new();
        let advisory_result = advisory_validator.run_validation_suite().await
            .map_err(|e| ValidationError::AdvisoryBoundary(format!("Advisory boundary validation failed: {}", e)))?;
        info!("Advisory Boundary: {} violations", advisory_result.violations.len());
        
        let end_time = Utc::now();
        let duration = end_time - start_time;
        
        // Aggregate all violations
        let total_violations = 
            contract_result.violations.len() +
            crypto_result.violations.len() +
            determinism_result.violations.len() +
            failure_result.violations.len() +
            resource_result.violations.len() +
            advisory_result.violations.len();
        
        // Make Go/No-Go decision
        let decision = if total_violations == 0 {
            GoNoGoDecision::Go
        } else {
            GoNoGoDecision::NoGo {
                reason: format!("{} total violations detected", total_violations),
                critical_violations: total_violations,
            }
        };
        
        let result = SystemValidationResult {
            contract_integrity: contract_result,
            cryptographic_continuity: crypto_result,
            determinism_replay: determinism_result,
            failure_isolation: failure_result,
            resource_ceilings: resource_result,
            advisory_boundary: advisory_result,
            timestamp: end_time,
            overall_decision: decision.clone(),
        };
        
        // Generate reports
        info!("Generating validation reports...");
        let reports = self.generate_reports(&result, duration).await?;
        
        // Save reports
        self.save_reports(reports).await?;
        
        match &decision {
            GoNoGoDecision::Go => {
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                info!("✅ SYSTEM VALIDATION: GO");
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
            GoNoGoDecision::NoGo { reason, .. } => {
                error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                error!("❌ SYSTEM VALIDATION: NO-GO - {}", reason);
                error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
        }
        
        Ok(result)
    }
    
    async fn generate_reports(&self, result: &SystemValidationResult, duration: chrono::Duration) -> ValidationResult<ValidationReport> {
        // Generate determinism proof
        let determinism_proof = DeterminismProof {
            identical_input_output: result.determinism_replay.identical_input_output,
            replay_consistency: result.determinism_replay.replay_consistency,
            no_hidden_nondeterminism: result.determinism_replay.no_hidden_nondeterminism,
            evidence: format!("{} test cases executed", result.determinism_replay.test_cases.len()),
        };
        
        // Generate trust chain verification
        let trust_chain_verification = TrustChainVerification {
            signature_verification_valid: result.cryptographic_continuity.signature_verification_valid,
            trust_chain_valid: result.cryptographic_continuity.trust_chain_valid,
            replay_resistance_valid: result.cryptographic_continuity.replay_resistance_valid,
            evidence: format!("{} cryptographic tests executed", result.cryptographic_continuity.test_cases.len()),
        };
        
        // Generate failure isolation matrix
        let failure_isolation_matrix = FailureIsolationMatrix {
            sensor_failure_isolation: result.failure_isolation.sensor_failure_isolation,
            dispatcher_failure_isolation: result.failure_isolation.dispatcher_failure_isolation,
            ai_advisory_failure_isolation: result.failure_isolation.ai_advisory_failure_isolation,
            evidence: format!("{} failure isolation tests executed", result.failure_isolation.test_cases.len()),
        };
        
        // Generate resource ceiling verification
        let resource_ceiling_verification = ResourceCeilingVerification {
            memory_limits_respected: result.resource_ceilings.memory_limits_respected,
            backpressure_behavior_valid: result.resource_ceilings.backpressure_behavior_valid,
            no_unbounded_growth: result.resource_ceilings.no_unbounded_growth,
            memory_metrics: result.resource_ceilings.memory_metrics.clone(),
            backpressure_metrics: result.resource_ceilings.backpressure_metrics.clone(),
            evidence: format!("{} resource ceiling tests executed", result.resource_ceilings.test_cases.len()),
        };
        
        let report = ValidationReport {
            timestamp: result.timestamp,
            duration_seconds: duration.num_seconds(),
            contract_integrity: result.contract_integrity.clone(),
            cryptographic_continuity: result.cryptographic_continuity.clone(),
            determinism_proof,
            trust_chain_verification,
            failure_isolation_matrix,
            resource_ceiling_verification,
            advisory_boundary: result.advisory_boundary.clone(),
            overall_decision: result.overall_decision.clone(),
            total_violations: 
                result.contract_integrity.violations.len() +
                result.cryptographic_continuity.violations.len() +
                result.determinism_replay.violations.len() +
                result.failure_isolation.violations.len() +
                result.resource_ceilings.violations.len() +
                result.advisory_boundary.violations.len(),
        };
        
        Ok(report)
    }
    
    async fn save_reports(&self, report: ValidationReport) -> ValidationResult<()> {
        // Save JSON report
        let json_path = self.output_dir.join("validation_report.json");
        let json_str = serde_json::to_string_pretty(&report)
            .map_err(|e| ValidationError::Serialization(e))?;
        std::fs::write(&json_path, json_str)
            .map_err(|e| ValidationError::Io(e))?;
        info!("Saved JSON report to: {:?}", json_path);
        
        // Save markdown report
        let md_path = self.output_dir.join("validation_report.md");
        let md_str = self.generate_markdown_report(&report);
        std::fs::write(&md_path, md_str)
            .map_err(|e| ValidationError::Io(e))?;
        info!("Saved Markdown report to: {:?}", md_path);
        
        Ok(())
    }
    
    fn generate_markdown_report(&self, report: &ValidationReport) -> String {
        let mut md = String::new();
        
        md.push_str("# RansomEye System-Wide Validation Report\n\n");
        md.push_str(&format!("**Generated:** {}\n", report.timestamp));
        md.push_str(&format!("**Duration:** {} seconds\n\n", report.duration_seconds));
        
        md.push_str("## Executive Summary\n\n");
        match &report.overall_decision {
            GoNoGoDecision::Go => {
                md.push_str("**Decision: ✅ GO**\n\n");
            }
            GoNoGoDecision::NoGo { reason, critical_violations } => {
                md.push_str(&format!("**Decision: ❌ NO-GO**\n\n"));
                md.push_str(&format!("**Reason:** {}\n\n", reason));
                md.push_str(&format!("**Critical Violations:** {}\n\n", critical_violations));
            }
        }
        md.push_str(&format!("**Total Violations:** {}\n\n", report.total_violations));
        
        md.push_str("## 1. Contract Integrity\n\n");
        md.push_str(&format!("- Envelope Schema Valid: {}\n", report.contract_integrity.envelope_schema_valid));
        md.push_str(&format!("- Version Compatibility Valid: {}\n", report.contract_integrity.version_compatibility_valid));
        md.push_str(&format!("- Fail-Closed Verified: {}\n", report.contract_integrity.fail_closed_verified));
        md.push_str(&format!("- Violations: {}\n\n", report.contract_integrity.violations.len()));
        
        md.push_str("## 2. Cryptographic Continuity\n\n");
        md.push_str(&format!("- Signature Verification Valid: {}\n", report.cryptographic_continuity.signature_verification_valid));
        md.push_str(&format!("- Trust Chain Valid: {}\n", report.cryptographic_continuity.trust_chain_valid));
        md.push_str(&format!("- Replay Resistance Valid: {}\n", report.cryptographic_continuity.replay_resistance_valid));
        md.push_str(&format!("- Violations: {}\n\n", report.cryptographic_continuity.violations.len()));
        
        md.push_str("## 3. Determinism Proof\n\n");
        md.push_str(&format!("- Identical Input → Identical Output: {}\n", report.determinism_proof.identical_input_output));
        md.push_str(&format!("- Replay Consistency: {}\n", report.determinism_proof.replay_consistency));
        md.push_str(&format!("- No Hidden Non-Determinism: {}\n", report.determinism_proof.no_hidden_nondeterminism));
        md.push_str(&format!("- Evidence: {}\n\n", report.determinism_proof.evidence));
        
        md.push_str("## 4. Trust Chain Verification\n\n");
        md.push_str(&format!("- Signature Verification: {}\n", report.trust_chain_verification.signature_verification_valid));
        md.push_str(&format!("- Trust Chain: {}\n", report.trust_chain_verification.trust_chain_valid));
        md.push_str(&format!("- Replay Resistance: {}\n", report.trust_chain_verification.replay_resistance_valid));
        md.push_str(&format!("- Evidence: {}\n\n", report.trust_chain_verification.evidence));
        
        md.push_str("## 5. Failure Isolation Matrix\n\n");
        md.push_str(&format!("- Sensor Failure Isolation: {}\n", report.failure_isolation_matrix.sensor_failure_isolation));
        md.push_str(&format!("- Dispatcher Failure Isolation: {}\n", report.failure_isolation_matrix.dispatcher_failure_isolation));
        md.push_str(&format!("- AI Advisory Failure Isolation: {}\n", report.failure_isolation_matrix.ai_advisory_failure_isolation));
        md.push_str(&format!("- Evidence: {}\n\n", report.failure_isolation_matrix.evidence));
        
        md.push_str("## 6. Resource Ceiling Verification\n\n");
        md.push_str(&format!("- Memory Limits Respected: {}\n", report.resource_ceiling_verification.memory_limits_respected));
        md.push_str(&format!("- Backpressure Behavior Valid: {}\n", report.resource_ceiling_verification.backpressure_behavior_valid));
        md.push_str(&format!("- No Unbounded Growth: {}\n", report.resource_ceiling_verification.no_unbounded_growth));
        md.push_str(&format!("- Evidence: {}\n\n", report.resource_ceiling_verification.evidence));
        
        md.push_str("## 7. Advisory Boundary Proof\n\n");
        md.push_str(&format!("- Compile-Time Checks Enforced: {}\n", report.advisory_boundary.compile_time_checks_enforced));
        md.push_str(&format!("- Runtime Checks Enforced: {}\n", report.advisory_boundary.runtime_checks_enforced));
        md.push_str(&format!("- No Enforcement Influence: {}\n", report.advisory_boundary.no_enforcement_influence));
        md.push_str(&format!("- Violations: {}\n\n", report.advisory_boundary.violations.len()));
        
        md.push_str("---\n");
        md.push_str("© RansomEye.Tech | Support: Gagan@RansomEye.Tech\n");
        
        md
    }
}

