// Path and File Name : /home/ransomeye/rebuild/core/deception/src/asset.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deception asset data structures and validation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    DecoyHost,
    DecoyService,
    CredentialLure,
    FilesystemLure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentScope {
    Network,
    Host,
    Identity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VisibilityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConditions {
    pub interaction_types: Vec<String>,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
}

fn default_min_confidence() -> f64 {
    0.9
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryFields {
    pub source_ip: String,
    pub destination_ip: String,
    pub timestamp: DateTime<Utc>,
    pub interaction_type: String,
    #[serde(default)]
    pub additional_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeardownStep {
    pub action: TeardownAction,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeardownAction {
    StopService,
    RemoveListener,
    DeleteFile,
    RemoveCredential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeardownProcedure {
    pub steps: Vec<TeardownStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionAsset {
    pub asset_id: String,
    pub asset_type: AssetType,
    pub deployment_scope: DeploymentScope,
    pub visibility_level: VisibilityLevel,
    pub trigger_conditions: TriggerConditions,
    pub telemetry_fields: TelemetryFields,
    pub teardown_procedure: TeardownProcedure,
    pub max_lifetime: u64,
    pub signature: String,
    pub signature_hash: String,
    #[serde(default)]
    pub metadata: Option<AssetMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl DeceptionAsset {
    /// Validate asset schema
    pub fn validate_schema(&self) -> Result<(), String> {
        // Validate asset_id is UUID
        Uuid::parse_str(&self.asset_id)
            .map_err(|e| format!("Invalid asset_id UUID: {}", e))?;
        
        // Validate min_confidence is in range
        if self.trigger_conditions.min_confidence < 0.0 || self.trigger_conditions.min_confidence > 1.0 {
            return Err("min_confidence must be between 0.0 and 1.0".to_string());
        }
        
        // Validate max_lifetime > 0
        if self.max_lifetime == 0 {
            return Err("max_lifetime must be > 0".to_string());
        }
        
        // Validate teardown procedure has steps
        if self.teardown_procedure.steps.is_empty() {
            return Err("teardown_procedure must have at least one step".to_string());
        }
        
        // Validate interaction_types not empty
        if self.trigger_conditions.interaction_types.is_empty() {
            return Err("trigger_conditions.interaction_types cannot be empty".to_string());
        }
        
        Ok(())
    }
    
    /// Check if asset has expired
    pub fn is_expired(&self, created_at: DateTime<Utc>) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(created_at);
        age.num_seconds() >= self.max_lifetime as i64
    }
    
    /// Get asset type as string for validation
    pub fn asset_type_str(&self) -> &'static str {
        match self.asset_type {
            AssetType::DecoyHost => "decoy_host",
            AssetType::DecoyService => "decoy_service",
            AssetType::CredentialLure => "credential_lure",
            AssetType::FilesystemLure => "filesystem_lure",
        }
    }
}

