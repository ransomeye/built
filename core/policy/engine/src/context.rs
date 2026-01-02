// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/context.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Evaluation context - deterministic evaluation context

use serde_json::Value;
use chrono::{DateTime, Utc};
use tracing::{error, debug};

use crate::errors::PolicyError;

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub alert_id: String,
    pub alert_severity: String,
    pub kill_chain_stage: String,
    pub asset_class: Option<String>,
    pub asset_id: Option<String>,
    pub producer_id: String,
    pub rule_ids: Vec<String>,
    pub evidence_reference: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Value,
}

impl EvaluationContext {
    pub fn new(
        alert_id: &str,
        alert_severity: &str,
        kill_chain_stage: &str,
        asset_class: Option<String>,
        asset_id: Option<String>,
        producer_id: &str,
        rule_ids: Vec<String>,
        evidence_reference: &str,
        metadata: Value,
    ) -> Self {
        Self {
            alert_id: alert_id.to_string(),
            alert_severity: alert_severity.to_string(),
            kill_chain_stage: kill_chain_stage.to_string(),
            asset_class,
            asset_id,
            producer_id: producer_id.to_string(),
            rule_ids,
            evidence_reference: evidence_reference.to_string(),
            timestamp: Utc::now(),
            metadata,
        }
    }

    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.alert_id.is_empty() {
            return Err(PolicyError::MissingContext("alert_id is required".to_string()));
        }
        
        if self.alert_severity.is_empty() {
            return Err(PolicyError::MissingContext("alert_severity is required".to_string()));
        }
        
        if self.kill_chain_stage.is_empty() {
            return Err(PolicyError::MissingContext("kill_chain_stage is required".to_string()));
        }
        
        if self.producer_id.is_empty() {
            return Err(PolicyError::MissingContext("producer_id is required".to_string()));
        }
        
        if self.evidence_reference.is_empty() {
            return Err(PolicyError::MissingContext("evidence_reference is required".to_string()));
        }
        
        debug!("Evaluation context validated: alert_id={}, severity={}, stage={}",
            self.alert_id, self.alert_severity, self.kill_chain_stage);
        
        Ok(())
    }

    pub fn get_field(&self, field: &str) -> Option<Value> {
        match field {
            "alert_id" => Some(Value::String(self.alert_id.clone())),
            "alert_severity" => Some(Value::String(self.alert_severity.clone())),
            "kill_chain_stage" => Some(Value::String(self.kill_chain_stage.clone())),
            "asset_class" => self.asset_class.as_ref().map(|s| Value::String(s.clone())),
            "asset_id" => self.asset_id.as_ref().map(|s| Value::String(s.clone())),
            "producer_id" => Some(Value::String(self.producer_id.clone())),
            "evidence_reference" => Some(Value::String(self.evidence_reference.clone())),
            _ => self.metadata.get(field).cloned(),
        }
    }
}

