// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/engine.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy engine - deterministic policy evaluation authority with audit logging

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn, debug};

use crate::errors::PolicyError;
use crate::context::EvaluationContext;
use crate::decision::PolicyDecision;
use crate::policy::PolicyLoader;
use crate::evaluator::PolicyEvaluator;
use crate::compiler::PolicyCompiler;
use crate::audit::{initialize_audit_logger, log_decision};

#[path = "../../security/revocation.rs"]
mod revocation;
use revocation::PolicyRevocationChecker;

pub struct PolicyEngine {
    evaluator: Arc<PolicyEvaluator>,
    revocation_checker: Arc<PolicyRevocationChecker>,
    compiler: Arc<PolicyCompiler>,
    started: Arc<AtomicBool>,
    engine_version: String,
    audit_enabled: bool,
}

impl PolicyEngine {
    pub fn new(
        policies_path: &str,
        engine_version: &str,
        trust_store_path: Option<&str>,
        revocation_list_path: Option<&str>,
        audit_log_path: Option<&str>,
    ) -> Result<Self, PolicyError> {
        info!("Initializing Policy Engine (version: {})", engine_version);

        let policy_loader = Arc::new(PolicyLoader::new(policies_path, trust_store_path)?);
        let compiler = Arc::new(PolicyCompiler::new());

        let policies = policy_loader.get_all_policies();
        for policy in policies {
            if policy.signature.is_none() {
                error!("Unsigned policy found: {}", policy.id);
                return Err(PolicyError::EngineRefusedToStart(
                    format!("Unsigned policy: {}", policy.id)
                ));
            }

            if let Err(e) = compiler.compile(policy) {
                error!("Policy compilation failed for {}: {}", policy.id, e);
                return Err(e);
            }
        }

        let revocation_checker = Arc::new(PolicyRevocationChecker::new(
            revocation_list_path.unwrap_or("/etc/ransomeye/policy/revocation.list")
        )?);

        let evaluator = Arc::new(PolicyEvaluator::new(policy_loader.clone())?);

        let audit_enabled = if let Some(audit_path) = audit_log_path {
            initialize_audit_logger(audit_path)?;
            true
        } else {
            false
        };

        info!("Policy Engine initialized successfully");
        
        Ok(Self {
            evaluator,
            revocation_checker,
            compiler,
            started: Arc::new(AtomicBool::new(true)),
            engine_version: engine_version.to_string(),
            audit_enabled,
        })
    }

    pub fn evaluate(&self, context: EvaluationContext) -> Result<PolicyDecision, PolicyError> {
        if !self.started.load(Ordering::Relaxed) {
            return Err(PolicyError::EngineRefusedToStart(
                "Engine is not started".to_string()
            ));
        }

        debug!("Evaluating policy for alert: {}", context.alert_id);

        match self.evaluator.evaluate(&context, 0) {
            Ok(decision) => {
                if !decision.verify() {
                    error!("Decision integrity verification failed");
                    return Err(PolicyError::EvaluationError(
                        "Decision integrity verification failed".to_string()
                    ));
                }

                if self.audit_enabled {
                    if let Err(e) = log_decision(&decision, &decision.policy_signature) {
                        error!("Audit logging failed: {}", e);
                        return Err(PolicyError::AuditLoggingFailed(
                            format!("Failed to log decision: {}", e)
                        ));
                    }
                }

                info!("Policy decision: {} (action: {:?})", decision.decision_id, decision.decision);
                Ok(decision)
            }
            Err(PolicyError::PolicyAmbiguity(msg)) => {
                error!("Policy ambiguity: {}", msg);
                Err(PolicyError::PolicyAmbiguity(msg))
            }
            Err(PolicyError::NoMatchingPolicy(_)) => {
                warn!("No matching policy, defaulting to DENY");
                self.create_deny_decision(&context)
            }
            Err(e) => {
                error!("Policy evaluation error: {}", e);
                Err(e)
            }
        }
    }

    fn create_deny_decision(&self, context: &EvaluationContext) -> Result<PolicyDecision, PolicyError> {
        use crate::decision::AllowedAction;

        let decision = PolicyDecision::new(
            &context.alert_id,
            "default_deny",
            "1.0.0",
            AllowedAction::Deny,
            vec![AllowedAction::Deny],
            Vec::new(),
            &context.evidence_reference,
            &context.kill_chain_stage,
            &context.alert_severity,
            context.asset_class.clone().or_else(|| context.asset_id.clone()),
            "No matching policy found, defaulting to DENY",
            "",
        );

        if self.audit_enabled {
            if let Err(e) = log_decision(&decision, "") {
                warn!("Audit logging failed for deny decision: {}", e);
            }
        }

        Ok(decision)
    }

    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::Relaxed)
    }

    pub fn version(&self) -> &str {
        &self.engine_version
    }
}

