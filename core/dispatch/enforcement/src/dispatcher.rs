// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/dispatcher.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main enforcement dispatcher - orchestrates all safety checks and execution

use std::sync::Arc;
use serde_json::{Value, json};
use tracing::{error, warn, info, debug};
use chrono::Utc;

use crate::errors::EnforcementError;
use crate::output::{EnforcementResult, ExecutionStatus};
use crate::validator::DecisionValidator;
use crate::approvals::ApprovalManager;
use crate::guardrails::Guardrails;
use crate::rate_limit::RateLimiter;
use crate::blast_radius::BlastRadiusLimiter;
use crate::rollback::RollbackManager;
use crate::dry_run::DryRunExecutor;
use crate::adapters::{LinuxAgentAdapter, WindowsAgentAdapter, NetworkAdapter};
use crate::security::signature::SignatureVerifier;
use crate::security::verification::DecisionVerifier;
use crate::security::revocation::RevocationChecker;

pub struct EnforcementDispatcher {
    validator: DecisionValidator,
    approval_manager: Arc<ApprovalManager>,
    guardrails: Arc<Guardrails>,
    rate_limiter: Arc<RateLimiter>,
    blast_radius_limiter: Arc<BlastRadiusLimiter>,
    rollback_manager: Arc<RollbackManager>,
    dry_run_executor: DryRunExecutor,
    linux_adapter: Option<LinuxAgentAdapter>,
    windows_adapter: Option<WindowsAgentAdapter>,
    network_adapter: Option<NetworkAdapter>,
}

impl EnforcementDispatcher {
    pub fn new() -> Result<Self, EnforcementError> {
        // Initialize signature verifier
        let public_key_path = std::env::var("RANSOMEYE_POLICY_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/keys/policy_public_key.pem".to_string());
        
        let public_key_bytes = std::fs::read(&public_key_path)
            .map_err(|e| EnforcementError::ConfigurationError(
                format!("Failed to read public key from {}: {}", public_key_path, e)
            ))?;
        
        let signature_verifier = SignatureVerifier::new(&public_key_bytes)?;
        let decision_verifier = DecisionVerifier::new(signature_verifier);
        
        // Initialize revocation checker
        let revocation_list_path = std::env::var("RANSOMEYE_REVOCATION_LIST_PATH")
            .ok();
        
        let revocation_checker = RevocationChecker::new(revocation_list_path);
        
        // Initialize validator
        let validator = DecisionValidator::new(decision_verifier, revocation_checker);
        
        // Initialize safety components
        let approval_manager = Arc::new(ApprovalManager::new());
        let guardrails = Arc::new(Guardrails::new());
        
        let max_actions = std::env::var("RANSOMEYE_ENFORCEMENT_RATE_LIMIT_MAX_ACTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        
        let window_seconds = std::env::var("RANSOMEYE_ENFORCEMENT_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);
        
        let rate_limiter = Arc::new(RateLimiter::new(max_actions, window_seconds));
        
        let max_hosts = std::env::var("RANSOMEYE_ENFORCEMENT_BLAST_RADIUS_MAX_HOSTS")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<usize>()
            .unwrap_or(50);
        
        let blast_radius_limiter = Arc::new(BlastRadiusLimiter::new(max_hosts, window_seconds));
        let rollback_manager = Arc::new(RollbackManager::new());
        let dry_run_executor = DryRunExecutor::new();
        
        // Initialize adapters (optional)
        let linux_adapter = LinuxAgentAdapter::new().ok();
        let windows_adapter = WindowsAgentAdapter::new().ok();
        let network_adapter = NetworkAdapter::new().ok();
        
        Ok(Self {
            validator,
            approval_manager,
            guardrails,
            rate_limiter,
            blast_radius_limiter,
            rollback_manager,
            dry_run_executor,
            linux_adapter,
            windows_adapter,
            network_adapter,
        })
    }
    
    /// Dispatch enforcement action with all safety checks
    pub async fn dispatch(&self, decision_json: &str, targets: &[String], dry_run: bool) -> Result<EnforcementResult, EnforcementError> {
        let start_time = Utc::now();
        
        // Parse decision
        let decision: Value = serde_json::from_str(decision_json)
            .map_err(|e| EnforcementError::InvalidFormat(format!("Invalid JSON: {}", e)))?;
        
        let decision_id = decision.get("decision_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision_id".to_string()))?;
        
        info!("Dispatching enforcement for decision {}", decision_id);
        
        // Step 1: Validate decision integrity
        debug!("Step 1: Validating decision integrity");
        self.validator.validate(decision_json)?;
        
        let mut result = if dry_run {
            self.dry_run_executor.simulate(&decision, targets)?
        } else {
            EnforcementResult::new(decision_id, false)
        };
        
        result.evidence.validator_checks.push("Decision signature verified".to_string());
        result.evidence.validator_checks.push("Decision hash verified".to_string());
        result.evidence.validator_checks.push("Decision not revoked".to_string());
        
        // Step 2: Check approvals
        debug!("Step 2: Checking required approvals");
        if self.approval_manager.requires_approval(&decision) {
            match self.approval_manager.check_approvals(&decision) {
                Ok(approval_statuses) => {
                    result.evidence.approval_status = approval_statuses;
                    debug!("All required approvals present");
                }
                Err(e) => {
                    result.status = ExecutionStatus::Held;
                    result.action_taken = Some(format!("Held pending approval: {}", e));
                    return Ok(result);
                }
            }
        }
        
        // Step 3: Apply guardrails
        debug!("Step 3: Applying guardrails");
        let guardrail_checks = self.guardrails.check(&decision, targets.len())?;
        result.evidence.guardrail_checks = guardrail_checks;
        
        // Step 4: Check rate limit
        debug!("Step 4: Checking rate limit");
        let rate_limit_key = format!("enforcement:{}", decision_id);
        let rate_limit_status = self.rate_limiter.check(&rate_limit_key)?;
        result.evidence.rate_limit_status = Some(rate_limit_status);
        
        // Step 5: Check blast radius
        debug!("Step 5: Checking blast radius");
        let blast_radius_key = format!("blast_radius:{}", decision_id);
        let blast_radius_status = self.blast_radius_limiter.check(&blast_radius_key, targets)?;
        result.evidence.blast_radius_status = Some(blast_radius_status);
        
        // Step 6: Execute via adapter
        if !dry_run {
            debug!("Step 6: Executing via adapter");
            let decision_action = decision.get("decision")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            let adapter_response = self.execute_via_adapter(&decision, targets, decision_action).await?;
            result.evidence.adapter_response = Some(adapter_response.clone());
            result.action_taken = Some(adapter_response);
            result.targets = targets.to_vec();
            
            // Record for rollback
            let rollback_commands = self.generate_rollback_commands(decision_action, targets);
            if !rollback_commands.is_empty() {
                let rollback_id = self.rollback_manager.record_execution(
                    &result.execution_id,
                    decision_id,
                    decision_action,
                    targets,
                    &rollback_commands,
                )?;
                result.rollback_available = true;
                result.rollback_id = Some(rollback_id);
            }
        } else {
            result.targets = targets.to_vec();
        }
        
        // Calculate execution duration
        let duration = Utc::now().signed_duration_since(start_time);
        result.evidence.execution_duration_ms = duration.num_milliseconds() as u64;
        
        info!("Enforcement dispatch completed for decision {}: status={:?}", 
            decision_id, result.status);
        
        Ok(result)
    }
    
    async fn execute_via_adapter(&self, decision: &Value, targets: &[String], action: &str) -> Result<String, EnforcementError> {
        // Determine adapter based on target type or decision metadata
        // For now, try Linux adapter first, then Windows, then network
        
        if let Some(ref adapter) = self.linux_adapter {
            match adapter.execute(decision, targets, false).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!("Linux adapter failed: {}, trying next adapter", e);
                }
            }
        }
        
        if let Some(ref adapter) = self.windows_adapter {
            match adapter.execute(decision, targets, false).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!("Windows adapter failed: {}, trying next adapter", e);
                }
            }
        }
        
        if let Some(ref adapter) = self.network_adapter {
            match adapter.execute(decision, targets, false).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!("Network adapter failed: {}", e);
                }
            }
        }
        
        Err(EnforcementError::AdapterFailure(
            "No adapter available or all adapters failed".to_string()
        ))
    }
    
    fn generate_rollback_commands(&self, action: &str, targets: &[String]) -> Vec<String> {
        let mut commands = Vec::new();
        
        if let Some(ref adapter) = self.linux_adapter {
            commands.extend(adapter.generate_rollback(action, targets));
        }
        
        if let Some(ref adapter) = self.windows_adapter {
            commands.extend(adapter.generate_rollback(action, targets));
        }
        
        if let Some(ref adapter) = self.network_adapter {
            commands.extend(adapter.generate_rollback(action, targets));
        }
        
        commands
    }
    
    /// Rollback an execution
    pub fn rollback(&self, execution_id: &str) -> Result<(), EnforcementError> {
        info!("Rolling back execution {}", execution_id);
        self.rollback_manager.rollback(execution_id)
    }
    
    /// Record an approval
    pub fn record_approval(&self, decision_id: &str, approval_type: &str, approver: &str) -> Result<(), EnforcementError> {
        self.approval_manager.record_approval(decision_id, approval_type, approver)
    }
}

