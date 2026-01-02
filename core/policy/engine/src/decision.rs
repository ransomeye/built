// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/decision.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy decision structure - deterministic decision output

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllowedAction {
    Allow,
    Deny,
    Quarantine,
    Isolate,
    Block,
    Monitor,
    Escalate,
    RequireApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub decision_id: String,
    pub created_at: DateTime<Utc>,
    pub alert_id: String,
    pub policy_id: String,
    pub policy_version: String,
    pub decision: AllowedAction,
    pub allowed_actions: Vec<AllowedAction>,
    pub required_approvals: Vec<String>,
    pub evidence_reference: String,
    pub kill_chain_stage: String,
    pub severity: String,
    pub asset_class: Option<String>,
    pub reasoning: String,
    pub policy_signature: String,
    pub decision_hash: String,
}

impl PolicyDecision {
    pub fn new(
        alert_id: &str,
        policy_id: &str,
        policy_version: &str,
        decision: AllowedAction,
        allowed_actions: Vec<AllowedAction>,
        required_approvals: Vec<String>,
        evidence_reference: &str,
        kill_chain_stage: &str,
        severity: &str,
        asset_class: Option<String>,
        reasoning: &str,
        policy_signature: &str,
    ) -> Self {
        let decision_id = Uuid::new_v4().to_string();
        
        let mut decision_obj = Self {
            decision_id: decision_id.clone(),
            created_at: Utc::now(),
            alert_id: alert_id.to_string(),
            policy_id: policy_id.to_string(),
            policy_version: policy_version.to_string(),
            decision: decision.clone(),
            allowed_actions: allowed_actions.clone(),
            required_approvals,
            evidence_reference: evidence_reference.to_string(),
            kill_chain_stage: kill_chain_stage.to_string(),
            severity: severity.to_string(),
            asset_class,
            reasoning: reasoning.to_string(),
            policy_signature: policy_signature.to_string(),
            decision_hash: String::new(),
        };
        
        decision_obj.decision_hash = decision_obj.compute_hash();
        
        decision_obj
    }
    
    fn compute_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        use hex;
        
        let json_bytes = serde_json::to_vec(self).expect("Failed to serialize decision");
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        hex::encode(hasher.finalize())
    }
    
    pub fn verify(&self) -> bool {
        let computed_hash = self.compute_hash();
        computed_hash == self.decision_hash
    }
    
    pub fn is_action_allowed(&self, action: &AllowedAction) -> bool {
        self.allowed_actions.contains(action)
    }
    
    pub fn is_deny(&self) -> bool {
        self.decision == AllowedAction::Deny
    }
    
    pub fn requires_approval(&self) -> bool {
        !self.required_approvals.is_empty()
    }
}

