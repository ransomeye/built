// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/enforcement.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Enforcement directive generation - outputs decisions only, no actions

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;

use crate::decision::PolicyDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementDirective {
    pub directive_id: String,
    pub created_at: DateTime<Utc>,
    pub decision_id: String,
    pub target_scope: TargetScope,
    pub action_type: String,
    pub ttl_seconds: u64,
    pub preconditions: Vec<String>,
    pub rollback_instructions: String,
    pub signature: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetScope {
    pub entity_id: Option<String>,
    pub host_id: Option<String>,
    pub network_id: Option<String>,
    pub time_window: Option<TimeWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub struct DirectiveGenerator;

impl DirectiveGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, decision: &PolicyDecision, ttl_seconds: u64) -> Result<EnforcementDirective, Box<dyn std::error::Error>> {
        let directive_id = Uuid::new_v4().to_string();
        let nonce = Uuid::new_v4().to_string();

        let target_scope = TargetScope {
            entity_id: decision.asset_class.clone(),
            host_id: decision.asset_class.clone(),
            network_id: None,
            time_window: Some(TimeWindow {
                start: Utc::now(),
                end: Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
            }),
        };

        let action_type = format!("{:?}", decision.decision);

        let mut directive = EnforcementDirective {
            directive_id: directive_id.clone(),
            created_at: Utc::now(),
            decision_id: decision.decision_id.clone(),
            target_scope,
            action_type,
            ttl_seconds,
            preconditions: vec!["policy_evaluation_passed".to_string()],
            rollback_instructions: format!("Revert directive {}", directive_id),
            signature: String::new(),
            nonce,
        };

        directive.signature = self.sign_directive(&directive)?;

        Ok(directive)
    }

    fn sign_directive(&self, directive: &EnforcementDirective) -> Result<String, Box<dyn std::error::Error>> {
        let json_bytes = serde_json::to_vec(directive)?;
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        let hash = hasher.finalize();
        Ok(base64::encode(hash))
    }

    pub fn verify_directive(&self, directive: &EnforcementDirective) -> Result<bool, Box<dyn std::error::Error>> {
        let mut directive_clone = directive.clone();
        let expected_signature = directive_clone.signature.clone();
        directive_clone.signature = String::new();

        let computed_signature = self.sign_directive(&directive_clone)?;
        Ok(computed_signature == expected_signature)
    }
}

