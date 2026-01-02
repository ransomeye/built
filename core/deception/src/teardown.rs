// Path and File Name : /home/ransomeye/rebuild/core/deception/src/teardown.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Teardown and rollback engine - explicit teardown, automatic teardown on timeout, emergency teardown via playbook rollback

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use tracing::{error, warn, info, debug};

use crate::asset::{DeceptionAsset, TeardownAction};
use crate::deployer::{DeceptionDeployer, DeploymentState};
use crate::errors::DeceptionError;
use crate::registry::DeceptionRegistry;

#[derive(Debug, Clone)]
pub struct TeardownResult {
    pub asset_id: String,
    pub teardown_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: TeardownStatus,
    pub steps_completed: Vec<String>,
    pub steps_failed: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeardownStatus {
    Pending,
    Running,
    Completed,
    Failed,
    SafeHalt, // System entered safe-halt state due to teardown failure
}

pub struct TeardownEngine {
    registry: Arc<DeceptionRegistry>,
    deployer: Arc<DeceptionDeployer>,
    active_teardowns: Arc<RwLock<HashMap<String, TeardownResult>>>,
}

impl TeardownEngine {
    /// Create new teardown engine
    pub fn new(
        registry: Arc<DeceptionRegistry>,
        deployer: Arc<DeceptionDeployer>,
    ) -> Self {
        Self {
            registry,
            deployer,
            active_teardowns: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Explicit teardown of asset
    pub async fn teardown_asset(&self, asset_id: &str) -> Result<TeardownResult, DeceptionError> {
        info!("Starting explicit teardown for asset: {}", asset_id);
        
        // Get asset
        let asset = self.registry.get_asset(asset_id)
            .ok_or_else(|| DeceptionError::AssetNotFound(asset_id.to_string()))?;
        
        // Get deployment state
        let deployment = self.deployer.get_deployment(asset_id)
            .ok_or_else(|| DeceptionError::AssetNotFound(
                format!("Asset {} is not deployed", asset_id)
            ))?;
        
        // Execute teardown
        self.execute_teardown(&asset, &deployment).await
    }
    
    /// Automatic teardown on timeout
    pub async fn teardown_expired(&self) -> Result<Vec<String>, DeceptionError> {
        info!("Checking for expired deployments to teardown");
        
        let expired_asset_ids = self.deployer.check_expired();
        let mut teardown_results = Vec::new();
        
        for asset_id in &expired_asset_ids {
            match self.teardown_asset(asset_id).await {
                Ok(result) => {
                    teardown_results.push(asset_id.clone());
                    info!("Successfully tore down expired asset: {}", asset_id);
                }
                Err(e) => {
                    error!("Failed to teardown expired asset {}: {}", asset_id, e);
                    // Continue with other assets
                }
            }
        }
        
        Ok(teardown_results)
    }
    
    /// Emergency teardown via playbook rollback
    /// 
    /// Called when playbook rollback is triggered
    /// Removes all assets associated with the playbook execution
    pub async fn emergency_teardown(&self, asset_ids: &[String]) -> Result<Vec<String>, DeceptionError> {
        info!("Starting emergency teardown for {} assets", asset_ids.len());
        
        let mut teardown_results = Vec::new();
        
        for asset_id in asset_ids {
            match self.teardown_asset(asset_id).await {
                Ok(result) => {
                    if result.status == TeardownStatus::Completed {
                        teardown_results.push(asset_id.clone());
                    } else {
                        error!("Emergency teardown failed for asset: {}", asset_id);
                        // FAIL-CLOSED: Enter safe-halt if emergency teardown fails
                        return Err(DeceptionError::SafeHalt(
                            format!("Emergency teardown failed for asset: {}", asset_id)
                        ));
                    }
                }
                Err(e) => {
                    error!("Emergency teardown error for asset {}: {}", asset_id, e);
                    // FAIL-CLOSED: Enter safe-halt on emergency teardown failure
                    return Err(DeceptionError::SafeHalt(
                        format!("Emergency teardown error for asset {}: {}", asset_id, e)
                    ));
                }
            }
        }
        
        info!("Emergency teardown completed for {} assets", teardown_results.len());
        Ok(teardown_results)
    }
    
    /// Execute teardown procedure
    async fn execute_teardown(
        &self,
        asset: &DeceptionAsset,
        deployment: &DeploymentState,
    ) -> Result<TeardownResult, DeceptionError> {
        let teardown_id = uuid::Uuid::new_v4().to_string();
        let started_at = Utc::now();
        
        let mut result = TeardownResult {
            asset_id: asset.asset_id.clone(),
            teardown_id: teardown_id.clone(),
            started_at,
            completed_at: None,
            status: TeardownStatus::Running,
            steps_completed: Vec::new(),
            steps_failed: Vec::new(),
            error: None,
        };
        
        // Store teardown state
        {
            let mut teardowns = self.active_teardowns.write();
            teardowns.insert(teardown_id.clone(), result.clone());
        }
        
        // Execute teardown steps
        for (step_idx, step) in asset.teardown_procedure.steps.iter().enumerate() {
            debug!("Executing teardown step {} for asset {}", step_idx, asset.asset_id);
            
            match self.execute_teardown_step(step, &deployment.deployment_metadata).await {
                Ok(step_id) => {
                    result.steps_completed.push(step_id);
                }
                Err(e) => {
                    let error_msg = format!("Teardown step {} failed: {}", step_idx, e);
                    result.steps_failed.push(error_msg.clone());
                    error!("{}", error_msg);
                    
                    // FAIL-CLOSED: Teardown failure → Safe-halt
                    result.status = TeardownStatus::Failed;
                    result.error = Some(error_msg.clone());
                    result.completed_at = Some(Utc::now());
                    
                    // Update teardown state
                    {
                        let mut teardowns = self.active_teardowns.write();
                        teardowns.insert(teardown_id.clone(), result.clone());
                    }
                    
                    // Enter safe-halt state
                    return Err(DeceptionError::SafeHalt(error_msg));
                }
            }
        }
        
        // All steps completed
        result.status = TeardownStatus::Completed;
        result.completed_at = Some(Utc::now());
        
        // Update deployment status
        // TODO: Update deployment state to TeardownComplete
        
        // Update teardown state
        {
            let mut teardowns = self.active_teardowns.write();
            teardowns.insert(teardown_id.clone(), result.clone());
        }
        
        info!("Successfully completed teardown for asset: {}", asset.asset_id);
        Ok(result)
    }
    
    /// Execute single teardown step
    async fn execute_teardown_step(
        &self,
        step: &crate::asset::TeardownStep,
        deployment_metadata: &HashMap<String, String>,
    ) -> Result<String, DeceptionError> {
        let step_id = uuid::Uuid::new_v4().to_string();
        
        match step.action {
            TeardownAction::StopService => {
                // Stop decoy service
                // TODO: Actual implementation would stop the service
                debug!("Stopping service (step: {})", step_id);
                Ok(step_id)
            }
            TeardownAction::RemoveListener => {
                // Remove network listener
                // TODO: Actual implementation would remove listener
                debug!("Removing listener (step: {})", step_id);
                Ok(step_id)
            }
            TeardownAction::DeleteFile => {
                // Delete filesystem lure file
                // TODO: Actual implementation would delete file
                debug!("Deleting file (step: {})", step_id);
                Ok(step_id)
            }
            TeardownAction::RemoveCredential => {
                // Remove credential lure
                // TODO: Actual implementation would remove credential
                debug!("Removing credential (step: {})", step_id);
                Ok(step_id)
            }
        }
    }
    
    /// Get teardown result
    pub fn get_teardown_result(&self, teardown_id: &str) -> Option<TeardownResult> {
        self.active_teardowns.read().get(teardown_id).cloned()
    }
    
    /// Get all active teardowns
    pub fn get_active_teardowns(&self) -> Vec<TeardownResult> {
        self.active_teardowns.read()
            .values()
            .filter(|t| t.status == TeardownStatus::Running || t.status == TeardownStatus::Pending)
            .cloned()
            .collect()
    }
}

