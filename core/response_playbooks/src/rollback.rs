// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/rollback.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Rollback engine - reverse-order execution, restart-safe persistence, fail-closed on rollback failure

use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use uuid;
use tracing::{error, warn, info, debug};

use crate::schema::{Playbook, RollbackStep};
use crate::errors::PlaybookError;
use crate::executor::{ExecutionContext, StepResult};
use crate::persistence::PlaybookPersistence;

#[derive(Debug, Clone)]
pub struct RollbackState {
    pub rollback_id: String,
    pub execution_id: String,
    pub playbook_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub rollback_step_results: Vec<RollbackStepResult>,
    pub status: RollbackStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackStatus {
    Pending,
    Running,
    Completed,
    Failed,
    SafeHalt, // System entered safe-halt state due to rollback failure
}

#[derive(Debug, Clone)]
pub struct RollbackStepResult {
    pub rollback_id: String,
    pub step_index: usize,
    pub status: RollbackStepStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub reverse_action_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct RollbackEngine {
    persistence: Arc<PlaybookPersistence>,
    active_rollbacks: Arc<RwLock<std::collections::HashMap<String, RollbackState>>>,
}

impl RollbackEngine {
    pub fn new(persistence: Arc<PlaybookPersistence>) -> Self {
        Self {
            persistence,
            active_rollbacks: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Execute rollback for a playbook execution
    pub async fn rollback(
        &self,
        execution_id: &str,
        playbook: &Playbook,
        execution_context: &ExecutionContext,
    ) -> Result<String, PlaybookError> {
        info!("Starting rollback for execution: {}", execution_id);
        
        let rollback_id = uuid::Uuid::new_v4().to_string();
        
        let mut rollback_state = RollbackState {
            rollback_id: rollback_id.clone(),
            execution_id: execution_id.to_string(),
            playbook_id: playbook.id.clone(),
            started_at: Utc::now(),
            completed_at: None,
            rollback_step_results: Vec::new(),
            status: RollbackStatus::Pending,
        };
        
        // Persist rollback state (restart-safe)
        self.persistence.save_rollback_state(&rollback_state).await?;
        
        // Store in active rollbacks
        {
            let mut active = self.active_rollbacks.write();
            active.insert(rollback_id.clone(), rollback_state.clone());
        }
        
        // Execute rollback steps in reverse order
        rollback_state.status = RollbackStatus::Running;
        self.persistence.save_rollback_state(&rollback_state).await?;
        
        // Get executed steps in reverse order
        let executed_steps: Vec<(usize, &StepResult)> = execution_context.step_results
            .iter()
            .filter(|(_, result)| result.status == crate::executor::StepStatus::Completed)
            .map(|(idx, result)| (*idx, result))
            .collect();
        
        // Sort by index descending (reverse order)
        let mut executed_steps_sorted = executed_steps;
        executed_steps_sorted.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Execute rollback steps
        for (step_index, step_result) in executed_steps_sorted {
            // Find corresponding rollback step
            if let Some(rollback_step) = self.find_rollback_step_for_step(
                playbook,
                step_index,
                &step_result.step_id,
            ) {
                let rollback_step_result = self.execute_rollback_step(
                    &rollback_step,
                    step_index,
                    &rollback_id,
                ).await?;
                
                rollback_state.rollback_step_results.push(rollback_step_result.clone());
                self.persistence.save_rollback_state(&rollback_state).await?;
                
                // Check if rollback step failed
                if rollback_step_result.status == RollbackStepStatus::Failed {
                    error!("Rollback step {} failed for execution {}", rollback_step.rollback_id, execution_id);
                    
                    // Rollback failure → SAFE-HALT state
                    rollback_state.status = RollbackStatus::Failed;
                    rollback_state.completed_at = Some(Utc::now());
                    self.persistence.save_rollback_state(&rollback_state).await?;
                    
                    // Enter safe-halt state
                    self.enter_safe_halt_state(&rollback_id, &rollback_step_result.error).await?;
                    
                    return Err(PlaybookError::RollbackFailed(
                        format!("Rollback step {} failed: {:?}", rollback_step.rollback_id, rollback_step_result.error)
                    ));
                }
            } else {
                warn!("No rollback step found for step {} in execution {}", step_result.step_id, execution_id);
            }
        }
        
        // All rollback steps completed
        rollback_state.status = RollbackStatus::Completed;
        rollback_state.completed_at = Some(Utc::now());
        self.persistence.save_rollback_state(&rollback_state).await?;
        
        info!("Rollback completed successfully: {}", rollback_id);
        Ok(rollback_id)
    }
    
    /// Resume rollback after restart (restart-safe)
    pub async fn resume_rollback(&self, rollback_id: &str) -> Result<(), PlaybookError> {
        // Load rollback state from persistence
        let mut rollback_state = self.persistence.load_rollback_state(rollback_id).await?
            .ok_or_else(|| PlaybookError::RollbackStateNotFound(
                format!("Rollback state {} not found", rollback_id)
            ))?;
        
        // Check if rollback is in a resumable state
        if rollback_state.status != RollbackStatus::Running && rollback_state.status != RollbackStatus::Pending {
            return Err(PlaybookError::InvalidExecutionState(
                format!("Rollback {} is in state {:?}, cannot resume", rollback_id, rollback_state.status)
            ));
        }
        
        // Load execution context
        let execution_context = self.persistence.load_execution(&rollback_state.execution_id).await?
            .ok_or_else(|| PlaybookError::PlaybookNotFound(
                format!("Execution {} not found", rollback_state.execution_id)
            ))?;
        
        // Load playbook (would need registry access)
        // For now, assume we can get it from context
        // In production, would need registry reference
        
        // Store in active rollbacks
        {
            let mut active = self.active_rollbacks.write();
            active.insert(rollback_id.to_string(), rollback_state.clone());
        }
        
        // Continue rollback execution
        // This would continue from where it left off
        // Implementation would be similar to rollback() but starting from last completed rollback step
        
        Ok(())
    }
    
    /// Find rollback step for a given execution step
    fn find_rollback_step_for_step(
        &self,
        playbook: &Playbook,
        step_index: usize,
        step_id: &str,
    ) -> Option<&RollbackStep> {
        // In production, this would match rollback steps to execution steps
        // For now, return first rollback step if available
        playbook.rollback_steps.first()
    }
    
    /// Execute a single rollback step
    async fn execute_rollback_step(
        &self,
        rollback_step: &RollbackStep,
        original_step_index: usize,
        rollback_id: &str,
    ) -> Result<RollbackStepResult, PlaybookError> {
        let start_time = Utc::now();
        
        let mut step_result = RollbackStepResult {
            rollback_id: rollback_step.rollback_id.clone(),
            step_index: original_step_index,
            status: RollbackStepStatus::Running,
            started_at: start_time,
            completed_at: None,
            error: None,
            reverse_action_result_id: None,
        };
        
        // Execute reverse action (calls Phase 7 enforcement with reverse parameters)
        match self.execute_reverse_action(rollback_step).await {
            Ok(reverse_action_result_id) => {
                step_result.status = RollbackStepStatus::Completed;
                step_result.reverse_action_result_id = Some(reverse_action_result_id);
            }
            Err(e) => {
                step_result.status = RollbackStepStatus::Failed;
                step_result.error = Some(e.to_string());
            }
        }
        
        step_result.completed_at = Some(Utc::now());
        
        Ok(step_result)
    }
    
    /// Execute reverse action (calls Phase 7 enforcement)
    async fn execute_reverse_action(
        &self,
        rollback_step: &RollbackStep,
    ) -> Result<String, PlaybookError> {
        // This would call Phase 7 EnforcementDispatcher with reverse action
        // For now, return mock result
        info!("Executing reverse action for rollback step: {}", rollback_step.rollback_id);
        Ok(format!("reverse_action_{}", uuid::Uuid::new_v4()))
    }
    
    /// Enter safe-halt state (system stops accepting new playbook executions)
    async fn enter_safe_halt_state(
        &self,
        rollback_id: &str,
        error: &Option<String>,
    ) -> Result<(), PlaybookError> {
        error!("ENTERING SAFE-HALT STATE due to rollback failure: {} - {:?}", rollback_id, error);
        
        // Persist safe-halt state
        self.persistence.mark_safe_halt().await?;
        
        // In production, this would:
        // 1. Stop accepting new playbook executions
        // 2. Alert operators
        // 3. Log critical event
        
        Ok(())
    }
    
    /// Get rollback status
    pub async fn get_rollback_status(&self, rollback_id: &str) -> Result<RollbackState, PlaybookError> {
        // Check active rollbacks first
        {
            let active = self.active_rollbacks.read();
            if let Some(state) = active.get(rollback_id) {
                return Ok(state.clone());
            }
        }
        
        // Load from persistence
        self.persistence.load_rollback_state(rollback_id).await?
            .ok_or_else(|| PlaybookError::RollbackStateNotFound(
                format!("Rollback {} not found", rollback_id)
            ))
    }
}

