// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/output.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Enforcement result output structure - execution evidence

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    pub execution_id: String,
    pub decision_id: String,
    pub created_at: DateTime<Utc>,
    pub status: ExecutionStatus,
    pub action_taken: Option<String>,
    pub targets: Vec<String>,
    pub evidence: ExecutionEvidence,
    pub dry_run: bool,
    pub rollback_available: bool,
    pub rollback_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Executed,
    DryRun,
    Rejected,
    Held, // Waiting for approval
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvidence {
    pub validator_checks: Vec<String>,
    pub approval_status: Vec<ApprovalStatus>,
    pub guardrail_checks: Vec<String>,
    pub rate_limit_status: Option<String>,
    pub blast_radius_status: Option<String>,
    pub adapter_response: Option<String>,
    pub execution_timestamp: DateTime<Utc>,
    pub execution_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStatus {
    pub approval_type: String,
    pub approved: bool,
    pub approver: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
}

impl EnforcementResult {
    pub fn new(decision_id: &str, dry_run: bool) -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            decision_id: decision_id.to_string(),
            created_at: Utc::now(),
            status: if dry_run {
                ExecutionStatus::DryRun
            } else {
                ExecutionStatus::Executed
            },
            action_taken: None,
            targets: Vec::new(),
            evidence: ExecutionEvidence {
                validator_checks: Vec::new(),
                approval_status: Vec::new(),
                guardrail_checks: Vec::new(),
                rate_limit_status: None,
                blast_radius_status: None,
                adapter_response: None,
                execution_timestamp: Utc::now(),
                execution_duration_ms: 0,
            },
            dry_run,
            rollback_available: false,
            rollback_id: None,
        }
    }
    
    pub fn rejected(reason: &str) -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            decision_id: String::new(),
            created_at: Utc::now(),
            status: ExecutionStatus::Rejected,
            action_taken: Some(format!("Rejected: {}", reason)),
            targets: Vec::new(),
            evidence: ExecutionEvidence {
                validator_checks: vec![reason.to_string()],
                approval_status: Vec::new(),
                guardrail_checks: Vec::new(),
                rate_limit_status: None,
                blast_radius_status: None,
                adapter_response: None,
                execution_timestamp: Utc::now(),
                execution_duration_ms: 0,
            },
            dry_run: false,
            rollback_available: false,
            rollback_id: None,
        }
    }
}

