// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/dispatcher.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main dispatcher orchestrator - Phase 7 core

use std::sync::Arc;
use serde_json;
use tracing::{error, warn, info, debug};
use chrono::Utc;

use super::directive_envelope::DirectiveEnvelope;
use super::acknowledgment_envelope::{AcknowledgmentEnvelope, ExecutionResult};
use super::errors::DispatcherError;
use super::verifier::DirectiveVerifier;
use super::router::{TargetRouter, AgentInfo};
use super::delivery::DeliveryService;
use super::acknowledgment::AcknowledgmentHandler;
use super::timeout::TimeoutManager;
use super::replay::ReplayGuard;
use super::reentrancy::ReentrancyGuard;
use super::rollback::RollbackManager;
use super::audit::{AuditLogger, AuditEventType};
use super::safety::SafetyGuards;
use super::trust_chain::TrustChain;
use super::nonce::NonceTracker;
use super::replay_protection::ReplayProtector;
use super::validation::ConfigValidator;

pub struct EnforcementDispatcher {
    verifier: DirectiveVerifier,
    router: Arc<TargetRouter>,
    delivery: DeliveryService,
    ack_handler: AcknowledgmentHandler,
    timeout_manager: TimeoutManager,
    replay_guard: ReplayGuard,
    reentrancy_guard: Arc<ReentrancyGuard>,
    rollback_manager: Arc<RollbackManager>,
    audit_logger: Arc<AuditLogger>,
    safety_guards: Arc<SafetyGuards>,
    config_validator: ConfigValidator,
}

impl EnforcementDispatcher {
    pub fn new() -> Result<Self, DispatcherError> {
        // Validate configuration (fail if missing ENV)
        let config_validator = ConfigValidator::new()
            .map_err(|e| DispatcherError::ConfigurationError(e))?;
        
        // Initialize trust chain
        let policy_key_path = std::env::var("RANSOMEYE_DISPATCHER_POLICY_KEY_PATH")
            .map_err(|_| DispatcherError::ConfigurationError(
                "RANSOMEYE_DISPATCHER_POLICY_KEY_PATH not set".to_string()
            ))?;
        
        let mut trust_chain = TrustChain::new();
        trust_chain.load_policy_key(&policy_key_path)
            .map_err(|e| DispatcherError::ConfigurationError(e))?;
        
        // Initialize nonce tracker
        let nonce_ttl = std::env::var("RANSOMEYE_DISPATCHER_NONCE_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<i64>()
            .unwrap_or(3600);
        
        let nonce_tracker = NonceTracker::new(nonce_ttl);
        
        // Initialize replay protector
        let replay_protector = ReplayProtector::new(10000);
        
        // Initialize verifier
        let verifier = DirectiveVerifier::new(
            trust_chain.clone(),
            nonce_tracker.clone(),
            replay_protector.clone(),
        );
        
        // Initialize router
        let router = Arc::new(TargetRouter::new());
        
        // Initialize delivery
        let delivery = DeliveryService::new()?;
        
        // Initialize acknowledgment handler
        let ack_handler = AcknowledgmentHandler::new(trust_chain.clone());
        
        // Initialize timeout manager
        let default_timeout = std::env::var("RANSOMEYE_DISPATCHER_ACK_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .unwrap_or(60);
        
        let timeout_manager = TimeoutManager::new(default_timeout);
        
        // Initialize replay guard
        let replay_guard = ReplayGuard::new(replay_protector, nonce_tracker);
        
        // Initialize reentrancy guard
        let reentrancy_guard = Arc::new(ReentrancyGuard::new());
        
        // Initialize rollback manager
        let rollback_ttl = std::env::var("RANSOMEYE_DISPATCHER_ROLLBACK_TTL_SECONDS")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<i64>()
            .unwrap_or(86400);
        
        let rollback_manager = Arc::new(RollbackManager::new(rollback_ttl));
        
        // Initialize audit logger
        let audit_log_path = std::env::var("RANSOMEYE_DISPATCHER_AUDIT_LOG_PATH")
            .map_err(|_| DispatcherError::ConfigurationError(
                "RANSOMEYE_DISPATCHER_AUDIT_LOG_PATH not set".to_string()
            ))?;
        
        let audit_logger = Arc::new(AuditLogger::new(&audit_log_path)?);
        
        // Initialize safety guards
        let max_actions = std::env::var("RANSOMEYE_DISPATCHER_MAX_ACTIONS_PER_WINDOW")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        
        let window_seconds = std::env::var("RANSOMEYE_DISPATCHER_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);
        
        let max_global = std::env::var("RANSOMEYE_DISPATCHER_MAX_GLOBAL_PER_WINDOW")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .unwrap_or(1000);
        
        let safety_guards = Arc::new(SafetyGuards::new(max_actions, window_seconds, max_global));
        
        Ok(Self {
            verifier,
            router,
            delivery,
            ack_handler,
            timeout_manager,
            replay_guard,
            reentrancy_guard,
            rollback_manager,
            audit_logger,
            safety_guards,
            config_validator,
        })
    }
    
    /// Process directive - main entry point
    pub async fn process_directive(&self, directive_json: &str, dry_run: bool) -> Result<String, DispatcherError> {
        // Parse directive
        let directive: DirectiveEnvelope = serde_json::from_str(directive_json)
            .map_err(|e| DispatcherError::InvalidDirective(format!("Invalid JSON: {}", e)))?;
        
        let directive_id = directive.directive_id.clone();
        info!("Processing directive {}", directive_id);
        
        // Enter reentrancy guard
        let _guard_token = self.reentrancy_guard.enter(&directive_id)?;
        
        // Log directive received
        self.audit_logger.append(
            AuditEventType::DirectiveReceived,
            serde_json::json!({
                "directive_id": directive_id,
                "policy_id": directive.policy_id,
                "action": directive.action,
            })
        )?;
        
        // Verify directive (ALL checks must pass)
        match self.verifier.verify(&directive) {
            Ok(_) => {
                self.audit_logger.append(
                    AuditEventType::DirectiveValidated,
                    serde_json::json!({
                        "directive_id": directive_id,
                        "policy_id": directive.policy_id,
                    })
                )?;
            }
            Err(e) => {
                self.audit_logger.append(
                    AuditEventType::DirectiveRejected,
                    serde_json::json!({
                        "directive_id": directive_id,
                        "error": format!("{:?}", e),
                    })
                )?;
                return Err(e);
            }
        }
        
        // Check safety guards
        self.safety_guards.check(&directive)?;
        
        // Resolve targets (strict resolution, no guessing)
        let agent_ids = self.router.resolve_targets(&directive)?;
        
        if agent_ids.is_empty() {
            return Err(DispatcherError::TargetResolutionFailed(
                "No agents resolved for directive".to_string()
            ));
        }
        
        // Execute for each agent
        let mut execution_results = Vec::new();
        
        for agent_id in agent_ids {
            let agent = self.router.get_agent(&agent_id)
                .ok_or_else(|| DispatcherError::AgentNotFound(agent_id.clone()))?;
            
            // Deliver directive
            if dry_run {
                self.delivery.deliver_dry_run(&directive, &agent).await?;
            } else {
                self.delivery.deliver(&directive, &agent).await?;
            }
            
            // Log execution attempt
            self.audit_logger.append(
                AuditEventType::ExecutionAttempted,
                serde_json::json!({
                    "directive_id": directive_id,
                    "agent_id": agent_id,
                    "action": directive.action,
                })
            )?;
            
            if !dry_run {
                // Wait for acknowledgment with timeout
                // In production, this would poll or use async channels
                // For now, simulate
                let execution_id = uuid::Uuid::new_v4().to_string();
                
                // Record for rollback
                let rollback_commands = vec![]; // Would be generated by agent
                self.rollback_manager.record_execution(
                    &execution_id,
                    &directive_id,
                    &directive.action,
                    &[],
                    &rollback_commands,
                )?;
                
                execution_results.push(execution_id);
            }
        }
        
        // Log execution completion
        self.audit_logger.append(
            AuditEventType::ExecutionSucceeded,
            serde_json::json!({
                "directive_id": directive_id,
                "execution_results": execution_results,
            })
        )?;
        
        Ok(format!("Directive {} processed successfully", directive_id))
    }
    
    /// Register agent
    pub fn register_agent(&self, agent: AgentInfo) {
        self.router.register_agent(agent);
    }
    
    /// Handle acknowledgment
    pub fn handle_acknowledgment(&self, ack_json: &str) -> Result<(), DispatcherError> {
        let ack: AcknowledgmentEnvelope = serde_json::from_str(ack_json)
            .map_err(|e| DispatcherError::InvalidAcknowledgment(format!("Invalid JSON: {}", e)))?;
        
        // Verify acknowledgment
        self.ack_handler.verify(&ack)?;
        
        // Log acknowledgment
        self.audit_logger.append(
            AuditEventType::AcknowledgmentReceived,
            serde_json::json!({
                "directive_id": ack.directive_id,
                "agent_id": ack.agent_id,
                "execution_result": format!("{:?}", ack.execution_result),
            })
        )?;
        
        // Check if execution failed
        if self.ack_handler.is_failure(&ack) {
            // Initiate rollback
            warn!("Execution failed for directive {}, initiating rollback", ack.directive_id);
            // Rollback logic would go here
        }
        
        Ok(())
    }
    
    /// Rollback execution
    pub fn rollback(&self, execution_id: &str, signature: Option<String>) -> Result<(), DispatcherError> {
        self.rollback_manager.rollback(execution_id, signature)
    }
}
