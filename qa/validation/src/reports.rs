// Path and File Name : /home/ransomeye/rebuild/qa/validation/src/reports.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Report structures for Phase 10 validation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::contract_integrity::ContractIntegrityResult;
use crate::cryptographic_continuity::CryptographicContinuityResult;
use crate::advisory_boundary::AdvisoryBoundaryResult;
use crate::resource_ceilings::{MemoryMetrics, BackpressureMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: i64,
    pub contract_integrity: ContractIntegrityResult,
    pub cryptographic_continuity: CryptographicContinuityResult,
    pub determinism_proof: DeterminismProof,
    pub trust_chain_verification: TrustChainVerification,
    pub failure_isolation_matrix: FailureIsolationMatrix,
    pub resource_ceiling_verification: ResourceCeilingVerification,
    pub advisory_boundary: AdvisoryBoundaryResult,
    pub overall_decision: GoNoGoDecision,
    pub total_violations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismProof {
    pub identical_input_output: bool,
    pub replay_consistency: bool,
    pub no_hidden_nondeterminism: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChainVerification {
    pub signature_verification_valid: bool,
    pub trust_chain_valid: bool,
    pub replay_resistance_valid: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureIsolationMatrix {
    pub sensor_failure_isolation: bool,
    pub dispatcher_failure_isolation: bool,
    pub ai_advisory_failure_isolation: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCeilingVerification {
    pub memory_limits_respected: bool,
    pub backpressure_behavior_valid: bool,
    pub no_unbounded_growth: bool,
    pub memory_metrics: Option<MemoryMetrics>,
    pub backpressure_metrics: Option<BackpressureMetrics>,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoNoGoDecision {
    Go,
    NoGo {
        reason: String,
        critical_violations: usize,
    },
}

