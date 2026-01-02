// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/approvals.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Approval workflow enforcement - validates required approvals before execution

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde_json::Value;
use tracing::{warn, debug};
use crate::errors::EnforcementError;
use crate::output::ApprovalStatus;

#[derive(Debug, Clone)]
pub struct Approval {
    pub approval_type: String,
    pub approver: String,
    pub approved_at: DateTime<Utc>,
    pub decision_id: String,
}

pub struct ApprovalManager {
    approvals: Arc<RwLock<HashMap<String, Vec<Approval>>>>, // decision_id -> approvals
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            approvals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if all required approvals are present
    pub fn check_approvals(&self, decision: &Value) -> Result<Vec<ApprovalStatus>, EnforcementError> {
        let decision_id = decision.get("decision_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision_id".to_string()))?;
        
        let required_approvals = decision.get("required_approvals")
            .and_then(|v| v.as_array())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing required_approvals".to_string()))?;
        
        if required_approvals.is_empty() {
            debug!("No approvals required for decision {}", decision_id);
            return Ok(Vec::new());
        }
        
        let approvals_map = self.approvals.read()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let decision_approvals = approvals_map.get(decision_id)
            .cloned()
            .unwrap_or_default();
        
        let mut statuses = Vec::new();
        let mut missing_approvals = Vec::new();
        
        for approval_type_val in required_approvals {
            let approval_type = approval_type_val.as_str()
                .ok_or_else(|| EnforcementError::InvalidFormat("Invalid approval type format".to_string()))?;
            
            let approval = decision_approvals.iter()
                .find(|a| a.approval_type == approval_type);
            
            match approval {
                Some(approval) => {
                    statuses.push(ApprovalStatus {
                        approval_type: approval_type.to_string(),
                        approved: true,
                        approver: Some(approval.approver.clone()),
                        approved_at: Some(approval.approved_at),
                    });
                }
                None => {
                    statuses.push(ApprovalStatus {
                        approval_type: approval_type.to_string(),
                        approved: false,
                        approver: None,
                        approved_at: None,
                    });
                    missing_approvals.push(approval_type.to_string());
                }
            }
        }
        
        if !missing_approvals.is_empty() {
            return Err(EnforcementError::MissingApproval(
                format!("Missing required approvals: {:?}", missing_approvals)
            ));
        }
        
        debug!("All approvals present for decision {}", decision_id);
        Ok(statuses)
    }
    
    /// Record an approval
    pub fn record_approval(&self, decision_id: &str, approval_type: &str, approver: &str) -> Result<(), EnforcementError> {
        let mut approvals_map = self.approvals.write()
            .map_err(|e| EnforcementError::InternalError(format!("Lock poisoned: {}", e)))?;
        
        let approvals = approvals_map.entry(decision_id.to_string())
            .or_insert_with(Vec::new);
        
        approvals.push(Approval {
            approval_type: approval_type.to_string(),
            approver: approver.to_string(),
            approved_at: Utc::now(),
            decision_id: decision_id.to_string(),
        });
        
        debug!("Recorded approval {} for decision {} by {}", approval_type, decision_id, approver);
        Ok(())
    }
    
    /// Check if decision requires approval
    pub fn requires_approval(&self, decision: &Value) -> bool {
        if let Some(required_approvals) = decision.get("required_approvals") {
            if let Some(arr) = required_approvals.as_array() {
                return !arr.is_empty();
            }
        }
        false
    }
}

