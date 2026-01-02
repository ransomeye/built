// Path and File Name : /home/ransomeye/rebuild/core/deception/src/visibility.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SOC Copilot visibility - read-only access to deception assets, health, interactions, and triggered playbooks

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::asset::DeceptionAsset;
use crate::deployer::{DeceptionDeployer, DeploymentState};
use crate::signals::DeceptionSignal;
use crate::registry::DeceptionRegistry;

/// SOC Copilot visibility interface
/// 
/// Rules:
/// - READ-ONLY access only
/// - Cannot deploy assets
/// - Cannot modify assets
/// - Cannot tear down assets
/// - Can view: deployed assets, asset health, interaction history, triggered playbooks
pub struct DeceptionVisibility {
    registry: Arc<DeceptionRegistry>,
    deployer: Arc<DeceptionDeployer>,
}

impl DeceptionVisibility {
    /// Create new visibility interface
    pub fn new(
        registry: Arc<DeceptionRegistry>,
        deployer: Arc<DeceptionDeployer>,
    ) -> Self {
        Self { registry, deployer }
    }
    
    /// Get all deployed deception assets (read-only)
    pub fn get_deployed_assets(&self) -> Vec<DeploymentView> {
        let deployments = self.deployer.get_active_deployments();
        let mut views = Vec::new();
        
        for deployment in deployments {
            if let Some(asset) = self.registry.get_asset(&deployment.asset_id) {
                views.push(DeploymentView {
                    asset_id: asset.asset_id.clone(),
                    asset_type: format!("{:?}", asset.asset_type),
                    deployment_scope: format!("{:?}", asset.deployment_scope),
                    visibility_level: format!("{:?}", asset.visibility_level),
                    deployed_at: deployment.deployed_at,
                    expires_at: deployment.expires_at,
                    status: format!("{:?}", deployment.status),
                    health: self.compute_asset_health(&deployment),
                });
            }
        }
        
        views
    }
    
    /// Get asset health status
    fn compute_asset_health(&self, deployment: &DeploymentState) -> AssetHealth {
        let now = Utc::now();
        let time_remaining = deployment.expires_at.signed_duration_since(now);
        
        if deployment.status != crate::deployer::DeploymentStatus::Active {
            return AssetHealth::Inactive;
        }
        
        if time_remaining.num_seconds() < 0 {
            return AssetHealth::Expired;
        }
        
        // Health is based on time remaining
        let percent_remaining = (time_remaining.num_seconds() as f64) / (deployment.expires_at.signed_duration_since(deployment.deployed_at).num_seconds() as f64);
        
        if percent_remaining > 0.5 {
            AssetHealth::Healthy
        } else if percent_remaining > 0.2 {
            AssetHealth::Warning
        } else {
            AssetHealth::Critical
        }
    }
    
    /// Get interaction history for asset (read-only)
    pub fn get_interaction_history(&self, asset_id: &str) -> Vec<InteractionView> {
        // TODO: Integrate with signal storage to retrieve interaction history
        // For now, return empty (would be populated from signal storage)
        Vec::new()
    }
    
    /// Get triggered playbooks for asset (read-only)
    pub fn get_triggered_playbooks(&self, asset_id: &str) -> Vec<PlaybookView> {
        // TODO: Integrate with Phase 6 playbook executor to retrieve triggered playbooks
        // For now, return empty (would be populated from playbook execution history)
        Vec::new()
    }
    
    /// Get asset details (read-only)
    pub fn get_asset_details(&self, asset_id: &str) -> Option<AssetDetails> {
        let asset = self.registry.get_asset(asset_id)?;
        let deployment = self.deployer.get_deployment(asset_id);
        
        Some(AssetDetails {
            asset_id: asset.asset_id.clone(),
            asset_type: format!("{:?}", asset.asset_type),
            deployment_scope: format!("{:?}", asset.deployment_scope),
            visibility_level: format!("{:?}", asset.visibility_level),
            trigger_conditions: asset.trigger_conditions.interaction_types.clone(),
            max_lifetime: asset.max_lifetime,
            deployment: deployment.map(|d| DeploymentInfo {
                deployed_at: d.deployed_at,
                expires_at: d.expires_at,
                status: format!("{:?}", d.status),
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentView {
    pub asset_id: String,
    pub asset_type: String,
    pub deployment_scope: String,
    pub visibility_level: String,
    pub deployed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub health: AssetHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetHealth {
    Healthy,
    Warning,
    Critical,
    Expired,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionView {
    pub interaction_id: String,
    pub interaction_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence_score: f64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookView {
    pub playbook_id: String,
    pub playbook_name: String,
    pub triggered_at: DateTime<Utc>,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetails {
    pub asset_id: String,
    pub asset_type: String,
    pub deployment_scope: String,
    pub visibility_level: String,
    pub trigger_conditions: Vec<String>,
    pub max_lifetime: u64,
    pub deployment: Option<DeploymentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

