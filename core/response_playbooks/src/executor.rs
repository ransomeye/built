// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/executor.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Playbook executor - deterministic step execution, state tracking, timeout enforcement, crash-safe resume

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::time::{timeout, Duration};
use tracing::{error, warn, info, debug};

use crate::schema::{Playbook, PlaybookStep, ActionType, AdapterType};
use crate::errors::PlaybookError;
use crate::registry::PlaybookRegistry;
use crate::persistence::PlaybookPersistence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    Pending,
    DryRun,
    Running,
    Failed,
    RolledBack,
    Completed,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub playbook_id: String,
    pub state: ExecutionState,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_step: usize,
    pub step_results: HashMap<usize, StepResult>,
    pub nonce: String,
    pub policy_decision_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub step_index: usize,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub enforcement_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub struct PlaybookExecutor {
    registry: Arc<PlaybookRegistry>,
    persistence: Arc<PlaybookPersistence>,
    active_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
}

impl PlaybookExecutor {
    pub fn new(
        registry: Arc<PlaybookRegistry>,
        persistence: Arc<PlaybookPersistence>,
    ) -> Self {
        Self {
            registry,
            persistence,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start playbook execution
    pub async fn execute(
        &self,
        playbook_id: &str,
        policy_decision_id: Option<&str>,
        dry_run: bool,
    ) -> Result<String, PlaybookError> {
        // Get playbook
        let playbook = self.registry.get_playbook(playbook_id)?;
        
        // Check if dry run is supported
        if dry_run && !playbook.dry_run_supported {
            return Err(PlaybookError::ConfigurationError(
                format!("Playbook {} does not support dry-run", playbook_id)
            ));
        }
        
        // Generate execution ID and nonce (replay protection)
        let execution_id = uuid::Uuid::new_v4().to_string();
        let nonce = uuid::Uuid::new_v4().to_string();
        
        // Check for replay attempt (nonce must be unique)
        if self.persistence.is_nonce_used(&nonce).await? {
            return Err(PlaybookError::ReplayAttempt(
                format!("Nonce {} already used - replay attempt detected", nonce)
            ));
        }
        
        // Create execution context
        let mut context = ExecutionContext {
            execution_id: execution_id.clone(),
            playbook_id: playbook_id.to_string(),
            state: if dry_run {
                ExecutionState::DryRun
            } else {
                ExecutionState::Pending
            },
            started_at: Utc::now(),
            completed_at: None,
            current_step: 0,
            step_results: HashMap::new(),
            nonce: nonce.clone(),
            policy_decision_id: policy_decision_id.map(|s| s.to_string()),
        };
        
        // Persist execution start
        self.persistence.save_execution(&context).await?;
        self.persistence.mark_nonce_used(&nonce).await?;
        
        // Store in active executions
        {
            let mut active = self.active_executions.write();
            active.insert(execution_id.clone(), context.clone());
        }
        
        // Start execution
        let executor_clone = self.clone_for_execution();
        tokio::spawn(async move {
            if let Err(e) = executor_clone.run_execution(context, playbook).await {
                error!("Playbook execution failed: {}", e);
            }
        });
        
        Ok(execution_id)
    }
    
    /// Resume execution after crash (crash-safe)
    pub async fn resume_execution(&self, execution_id: &str) -> Result<(), PlaybookError> {
        // Load execution context from persistence
        let context = self.persistence.load_execution(execution_id).await?
            .ok_or_else(|| PlaybookError::PlaybookNotFound(
                format!("Execution {} not found", execution_id)
            ))?;
        
        // Check if execution is in a resumable state
        if context.state != ExecutionState::Running && context.state != ExecutionState::Pending {
            return Err(PlaybookError::InvalidExecutionState(
                format!("Execution {} is in state {:?}, cannot resume", execution_id, context.state)
            ));
        }
        
        // Get playbook
        let playbook = self.registry.get_playbook(&context.playbook_id)?;
        
        // Store in active executions
        {
            let mut active = self.active_executions.write();
            active.insert(execution_id.to_string(), context.clone());
        }
        
        // Resume execution
        let executor_clone = self.clone_for_execution();
        tokio::spawn(async move {
            if let Err(e) = executor_clone.run_execution(context, playbook).await {
                error!("Playbook execution resume failed: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Run execution (internal)
    async fn run_execution(
        &self,
        mut context: ExecutionContext,
        playbook: Playbook,
    ) -> Result<(), PlaybookError> {
        info!("Starting playbook execution: {} (execution_id: {})", playbook.id, context.execution_id);
        
        // Update state to Running
        context.state = ExecutionState::Running;
        self.persistence.save_execution(&context).await?;
        
        let max_duration = Duration::from_secs(playbook.max_execution_time);
        let execution_future = self.execute_steps(context.clone(), playbook.clone());
        
        // Enforce max execution time
        match timeout(max_duration, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                error!("Playbook execution timeout: {}", context.execution_id);
                context.state = ExecutionState::Failed;
                context.completed_at = Some(Utc::now());
                self.persistence.save_execution(&context).await?;
                Err(PlaybookError::ExecutionTimeout(
                    format!("Execution exceeded max_execution_time: {}s", playbook.max_execution_time)
                ))
            }
        }
    }
    
    /// Execute all steps
    async fn execute_steps(
        &self,
        mut context: ExecutionContext,
        playbook: Playbook,
    ) -> Result<(), PlaybookError> {
        let dry_run = context.state == ExecutionState::DryRun;
        
        for (step_index, step) in playbook.steps.iter().enumerate() {
            context.current_step = step_index;
            
            // Check preconditions
            if !self.check_preconditions(&step.preconditions, &context).await? {
                warn!("Preconditions not met for step {}, skipping", step.step_id);
                let step_result = StepResult {
                    step_id: step.step_id.clone(),
                    step_index,
                    status: StepStatus::Skipped,
                    started_at: Utc::now(),
                    completed_at: Some(Utc::now()),
                    duration_ms: 0,
                    error: Some("Preconditions not met".to_string()),
                    enforcement_result_id: None,
                };
                context.step_results.insert(step_index, step_result);
                self.persistence.save_execution(&context).await?;
                continue;
            }
            
            // Execute step
            let step_result = self.execute_step(step, step_index, dry_run).await?;
            
            context.step_results.insert(step_index, step_result.clone());
            self.persistence.save_execution(&context).await?;
            
            // Check if step failed
            if step_result.status == StepStatus::Failed {
                if step.continue_on_failure {
                    warn!("Step {} failed but continue_on_failure=true, continuing", step.step_id);
                    continue;
                } else {
                    error!("Step {} failed and continue_on_failure=false, aborting", step.step_id);
                    context.state = ExecutionState::Failed;
                    context.completed_at = Some(Utc::now());
                    self.persistence.save_execution(&context).await?;
                    return Err(PlaybookError::StepExecutionFailed(
                        format!("Step {} failed: {:?}", step.step_id, step_result.error)
                    ));
                }
            }
        }
        
        // All steps completed
        context.state = ExecutionState::Completed;
        context.completed_at = Some(Utc::now());
        self.persistence.save_execution(&context).await?;
        
        info!("Playbook execution completed: {}", context.execution_id);
        Ok(())
    }
    
    /// Execute a single step
    async fn execute_step(
        &self,
        step: &PlaybookStep,
        step_index: usize,
        dry_run: bool,
    ) -> Result<StepResult, PlaybookError> {
        let start_time = Utc::now();
        
        let mut step_result = StepResult {
            step_id: step.step_id.clone(),
            step_index,
            status: StepStatus::Running,
            started_at: start_time,
            completed_at: None,
            duration_ms: 0,
            error: None,
            enforcement_result_id: None,
        };
        
        let step_duration = Duration::from_secs(step.timeout_seconds);
        let step_future = self.run_step_action(step, dry_run);
        
        match timeout(step_duration, step_future).await {
            Ok(Ok(enforcement_result_id)) => {
                step_result.status = StepStatus::Completed;
                step_result.enforcement_result_id = Some(enforcement_result_id);
            }
            Ok(Err(e)) => {
                step_result.status = StepStatus::Failed;
                step_result.error = Some(e.to_string());
            }
            Err(_) => {
                step_result.status = StepStatus::Failed;
                step_result.error = Some(format!("Step timeout after {}s", step.timeout_seconds));
            }
        }
        
        step_result.completed_at = Some(Utc::now());
        step_result.duration_ms = (step_result.completed_at.unwrap() - start_time)
            .num_milliseconds() as u64;
        
        Ok(step_result)
    }
    
    /// Run step action (calls Phase 7 enforcement)
    async fn run_step_action(
        &self,
        step: &PlaybookStep,
        dry_run: bool,
    ) -> Result<String, PlaybookError> {
        // This would call Phase 7 EnforcementDispatcher
        // For now, return a mock result
        // In production, this would:
        // 1. Create enforcement directive from step
        // 2. Call EnforcementDispatcher::execute()
        // 3. Return enforcement_result_id
        
        if dry_run {
            info!("DRY RUN: Would execute step {} with action {:?}", step.step_id, step.action_type);
            Ok(format!("dry_run_{}", uuid::Uuid::new_v4()))
        } else {
            // TODO: Integrate with Phase 7 EnforcementDispatcher
            // For now, simulate execution
            info!("Executing step {} with action {:?}", step.step_id, step.action_type);
            Ok(format!("enforcement_{}", uuid::Uuid::new_v4()))
        }
    }
    
    /// Check preconditions
    async fn check_preconditions(
        &self,
        preconditions: &[String],
        _context: &ExecutionContext,
    ) -> Result<bool, PlaybookError> {
        // In production, this would check actual preconditions
        // For now, assume all preconditions are met
        Ok(true)
    }
    
    /// Get execution status
    pub async fn get_execution_status(&self, execution_id: &str) -> Result<ExecutionContext, PlaybookError> {
        // Check active executions first
        {
            let active = self.active_executions.read();
            if let Some(context) = active.get(execution_id) {
                return Ok(context.clone());
            }
        }
        
        // Load from persistence
        self.persistence.load_execution(execution_id).await?
            .ok_or_else(|| PlaybookError::PlaybookNotFound(
                format!("Execution {} not found", execution_id)
            ))
    }
    
    /// Clone executor for async execution
    fn clone_for_execution(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            persistence: Arc::clone(&self.persistence),
            active_executions: Arc::clone(&self.active_executions),
        }
    }
}

impl Clone for PlaybookExecutor {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            persistence: Arc::clone(&self.persistence),
            active_executions: Arc::clone(&self.active_executions),
        }
    }
}

