// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/schema.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Playbook data structures and schema validation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub version: String,
    pub severity: Severity,
    pub trigger_conditions: TriggerConditions,
    pub steps: Vec<PlaybookStep>,
    pub rollback_steps: Vec<RollbackStep>,
    pub approvals_required: ApprovalsRequired,
    pub dry_run_supported: bool,
    pub timeout_per_step: u64,
    pub max_execution_time: u64,
    pub signature: String,
    pub signature_hash: String,
    pub metadata: Option<PlaybookMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConditions {
    pub policy_outcomes: Vec<String>,
    pub alert_severity: Vec<Severity>,
    pub kill_chain_stage: Vec<String>,
    #[serde(default)]
    pub custom_conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStep {
    pub step_id: String,
    pub step_name: String,
    pub action_type: ActionType,
    pub enforcement_action: EnforcementAction,
    pub timeout_seconds: u64,
    pub retry_on_failure: bool,
    pub continue_on_failure: bool,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub postconditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Isolate,
    Quarantine,
    Block,
    Notify,
    Collect,
    Analyze,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementAction {
    pub adapter: AdapterType,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterType {
    LinuxAgent,
    WindowsAgent,
    Network,
    Core,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    pub rollback_id: String,
    pub rollback_name: String,
    pub reverse_action: EnforcementAction,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalsRequired {
    pub human_approval: bool,
    pub auto_approval: bool,
    #[serde(default)]
    pub approval_timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookMetadata {
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Playbook {
    /// Validate playbook structure and required fields
    pub fn validate(&self) -> Result<(), String> {
        // Validate UUID format
        Uuid::parse_str(&self.id)
            .map_err(|_| format!("Invalid playbook ID format: {}", self.id))?;
        
        // Validate version format (semantic versioning)
        let version_parts: Vec<&str> = self.version.split('.').collect();
        if version_parts.len() != 3 {
            return Err(format!("Invalid version format: {}", self.version));
        }
        for part in &version_parts {
            part.parse::<u32>()
                .map_err(|_| format!("Invalid version component: {}", part))?;
        }
        
        // Validate steps
        if self.steps.is_empty() {
            return Err("Playbook must have at least one step".to_string());
        }
        
        for (idx, step) in self.steps.iter().enumerate() {
            if step.step_id.is_empty() {
                return Err(format!("Step {} has empty step_id", idx));
            }
            if step.timeout_seconds == 0 || step.timeout_seconds > 3600 {
                return Err(format!("Step {} has invalid timeout: {}", idx, step.timeout_seconds));
            }
        }
        
        // Validate rollback steps
        for (idx, rollback) in self.rollback_steps.iter().enumerate() {
            if rollback.rollback_id.is_empty() {
                return Err(format!("Rollback step {} has empty rollback_id", idx));
            }
            if rollback.timeout_seconds == 0 || rollback.timeout_seconds > 3600 {
                return Err(format!("Rollback step {} has invalid timeout: {}", idx, rollback.timeout_seconds));
            }
        }
        
        // Validate timeouts
        if self.timeout_per_step == 0 || self.timeout_per_step > 3600 {
            return Err(format!("Invalid timeout_per_step: {}", self.timeout_per_step));
        }
        if self.max_execution_time == 0 || self.max_execution_time > 86400 {
            return Err(format!("Invalid max_execution_time: {}", self.max_execution_time));
        }
        
        // Validate signature fields are present
        if self.signature.is_empty() {
            return Err("Playbook signature is required".to_string());
        }
        if self.signature_hash.is_empty() {
            return Err("Playbook signature_hash is required".to_string());
        }
        
        Ok(())
    }
    
    /// Compute content hash (before signature)
    pub fn compute_content_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        
        let mut playbook_clone = self.clone();
        playbook_clone.signature = String::new();
        playbook_clone.signature_hash = String::new();
        
        let json_bytes = serde_json::to_vec(&playbook_clone)
            .expect("Failed to serialize playbook for hashing");
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        let hash = hasher.finalize();
        
        hex::encode(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_playbook_validation() {
        let playbook = Playbook {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Test Playbook".to_string(),
            version: "1.0.0".to_string(),
            severity: Severity::High,
            trigger_conditions: TriggerConditions {
                policy_outcomes: vec!["deny".to_string()],
                alert_severity: vec![Severity::High],
                kill_chain_stage: vec!["exploitation".to_string()],
                custom_conditions: None,
            },
            steps: vec![PlaybookStep {
                step_id: "step_00000001".to_string(),
                step_name: "Test Step".to_string(),
                action_type: ActionType::Isolate,
                enforcement_action: EnforcementAction {
                    adapter: AdapterType::LinuxAgent,
                    parameters: serde_json::json!({}),
                },
                timeout_seconds: 60,
                retry_on_failure: false,
                continue_on_failure: false,
                preconditions: vec![],
                postconditions: vec![],
            }],
            rollback_steps: vec![],
            approvals_required: ApprovalsRequired {
                human_approval: false,
                auto_approval: true,
                approval_timeout_seconds: None,
            },
            dry_run_supported: true,
            timeout_per_step: 60,
            max_execution_time: 3600,
            signature: "test_signature".to_string(),
            signature_hash: "a".repeat(64),
            metadata: None,
        };
        
        assert!(playbook.validate().is_ok());
    }
}

