// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/output/audit_record.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Audit record output structure

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: DateTime<Utc>,
    pub policy_id: String,
    pub policy_version: String,
    pub signature_hash: String,
    pub input_reference: String,
    pub decision: String,
    pub decision_id: String,
    pub previous_hash: Option<String>,
    pub record_hash: String,
}

impl AuditRecord {
    pub fn new(
        timestamp: DateTime<Utc>,
        policy_id: String,
        policy_version: String,
        signature_hash: String,
        input_reference: String,
        decision: String,
        decision_id: String,
        previous_hash: Option<String>,
        record_hash: String,
    ) -> Self {
        Self {
            timestamp,
            policy_id,
            policy_version,
            signature_hash,
            input_reference,
            decision,
            decision_id,
            previous_hash,
            record_hash,
        }
    }
}

