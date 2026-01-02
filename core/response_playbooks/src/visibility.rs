// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/visibility.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SOC Copilot visibility interface - read-only access to playbook intent, execution status, rollback status

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::schema::Playbook;
use crate::executor::{ExecutionContext, ExecutionState};
use crate::rollback::RollbackState;
use crate::registry::PlaybookRegistry;
use crate::executor::PlaybookExecutor;
use crate::rollback::RollbackEngine;
use crate::errors::PlaybookError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookIntent {
    pub playbook_id: String,
    pub name: String,
    pub severity: String,
    pub steps: Vec<StepIntent>,
    pub rollback_steps: Vec<RollbackStepIntent>,
    pub approvals_required: ApprovalsIntent,
    pub dry_run_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepIntent {
    pub step_id: String,
    pub step_name: String,
    pub action_type: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStepIntent {
    pub rollback_id: String,
    pub rollback_name: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalsIntent {
    pub human_approval: bool,
    pub auto_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatusView {
    pub execution_id: String,
    pub playbook_id: String,
    pub state: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub current_step: usize,
    pub total_steps: usize,
    pub step_statuses: Vec<StepStatusView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStatusView {
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStatusView {
    pub rollback_id: String,
    pub execution_id: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub rollback_step_count: usize,
}

pub struct PlaybookVisibility {
    registry: Arc<PlaybookRegistry>,
    executor: Arc<PlaybookExecutor>,
    rollback_engine: Arc<RollbackEngine>,
}

impl PlaybookVisibility {
    pub fn new(
        registry: Arc<PlaybookRegistry>,
        executor: Arc<PlaybookExecutor>,
        rollback_engine: Arc<RollbackEngine>,
    ) -> Self {
        Self {
            registry,
            executor,
            rollback_engine,
        }
    }
    
    /// Get playbook intent (read-only)
    pub async fn get_playbook_intent(&self, playbook_id: &str) -> Result<PlaybookIntent, PlaybookError> {
        let playbook = self.registry.get_playbook(playbook_id)?;
        
        let steps: Vec<StepIntent> = playbook.steps.iter().map(|step| {
            StepIntent {
                step_id: step.step_id.clone(),
                step_name: step.step_name.clone(),
                action_type: format!("{:?}", step.action_type),
                timeout_seconds: step.timeout_seconds,
            }
        }).collect();
        
        let rollback_steps: Vec<RollbackStepIntent> = playbook.rollback_steps.iter().map(|rb| {
            RollbackStepIntent {
                rollback_id: rb.rollback_id.clone(),
                rollback_name: rb.rollback_name.clone(),
                timeout_seconds: rb.timeout_seconds,
            }
        }).collect();
        
        Ok(PlaybookIntent {
            playbook_id: playbook.id.clone(),
            name: playbook.name.clone(),
            severity: format!("{:?}", playbook.severity),
            steps,
            rollback_steps,
            approvals_required: ApprovalsIntent {
                human_approval: playbook.approvals_required.human_approval,
                auto_approval: playbook.approvals_required.auto_approval,
            },
            dry_run_supported: playbook.dry_run_supported,
        })
    }
    
    /// Get execution status (read-only)
    pub async fn get_execution_status(&self, execution_id: &str) -> Result<ExecutionStatusView, PlaybookError> {
        let context = self.executor.get_execution_status(execution_id).await?;
        let playbook = self.registry.get_playbook(&context.playbook_id)?;
        
        let step_statuses: Vec<StepStatusView> = context.step_results.iter().map(|(idx, result)| {
            StepStatusView {
                step_id: result.step_id.clone(),
                step_name: playbook.steps.get(*idx)
                    .map(|s| s.step_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
                status: format!("{:?}", result.status),
                started_at: result.started_at.to_rfc3339(),
                completed_at: result.completed_at.map(|dt| dt.to_rfc3339()),
                error: result.error.clone(),
            }
        }).collect();
        
        Ok(ExecutionStatusView {
            execution_id: context.execution_id.clone(),
            playbook_id: context.playbook_id.clone(),
            state: format!("{:?}", context.state),
            started_at: context.started_at.to_rfc3339(),
            completed_at: context.completed_at.map(|dt| dt.to_rfc3339()),
            current_step: context.current_step,
            total_steps: playbook.steps.len(),
            step_statuses,
        })
    }
    
    /// Get rollback status (read-only)
    pub async fn get_rollback_status(&self, rollback_id: &str) -> Result<RollbackStatusView, PlaybookError> {
        let state = self.rollback_engine.get_rollback_status(rollback_id).await?;
        
        Ok(RollbackStatusView {
            rollback_id: state.rollback_id.clone(),
            execution_id: state.execution_id.clone(),
            status: format!("{:?}", state.status),
            started_at: state.started_at.to_rfc3339(),
            completed_at: state.completed_at.map(|dt| dt.to_rfc3339()),
            rollback_step_count: state.rollback_step_results.len(),
        })
    }
    
    /// List all playbooks (read-only)
    pub fn list_playbooks(&self) -> Vec<String> {
        self.registry.list_playbooks()
    }
    
    /// Get planned steps for a playbook (read-only)
    pub async fn get_planned_steps(&self, playbook_id: &str) -> Result<Vec<StepIntent>, PlaybookError> {
        let playbook = self.registry.get_playbook(playbook_id)?;
        
        Ok(playbook.steps.iter().map(|step| {
            StepIntent {
                step_id: step.step_id.clone(),
                step_name: step.step_name.clone(),
                action_type: format!("{:?}", step.action_type),
                timeout_seconds: step.timeout_seconds,
            }
        }).collect())
    }
}

// Note: This interface is READ-ONLY
// SOC Copilot cannot modify playbooks, only view intent and status

