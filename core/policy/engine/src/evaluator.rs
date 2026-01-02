// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/evaluator.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy evaluator with runtime safety guards

use std::sync::Arc;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tracing::{error, debug, warn};
use parking_lot::RwLock;
use once_cell::sync::Lazy;

use crate::errors::PolicyError;
use crate::context::EvaluationContext;
use crate::policy::{PolicyLoader, PolicyRule};
use crate::matcher::PolicyMatcher;
use crate::decision::{PolicyDecision, AllowedAction};
use crate::conflict::{ConflictDetector, ConflictResolver};
use crate::precedence::PrecedenceRules;

static RATE_LIMITER: Lazy<Arc<RwLock<RateLimiter>>> = Lazy::new(|| {
    Arc::new(RwLock::new(RateLimiter::new()))
});

struct RateLimiter {
    requests: Vec<Instant>,
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            max_requests: 1000,
            window_seconds: 60,
        }
    }

    fn check_rate_limit(&mut self) -> Result<(), PolicyError> {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.window_seconds);

        self.requests.retain(|&time| time > window_start);

        if self.requests.len() >= self.max_requests {
            return Err(PolicyError::RateLimitExceeded(
                format!("Rate limit exceeded: {} requests in {} seconds", 
                    self.requests.len(), self.window_seconds)
            ));
        }

        self.requests.push(now);
        Ok(())
    }
}

pub struct PolicyEvaluator {
    policy_loader: Arc<PolicyLoader>,
    matcher: Arc<PolicyMatcher>,
    conflict_detector: Arc<ConflictDetector>,
    conflict_resolver: Arc<ConflictResolver>,
    precedence_rules: Arc<PrecedenceRules>,
    max_depth: usize,
}

impl PolicyEvaluator {
    pub fn new(policy_loader: Arc<PolicyLoader>) -> Result<Self, PolicyError> {
        Ok(Self {
            policy_loader,
            matcher: Arc::new(PolicyMatcher::new()),
            conflict_detector: Arc::new(ConflictDetector::new()),
            conflict_resolver: Arc::new(ConflictResolver::new()),
            precedence_rules: Arc::new(PrecedenceRules::new()),
            max_depth: 10,
        })
    }

    pub fn evaluate(&self, context: &EvaluationContext, depth: usize) -> Result<PolicyDecision, PolicyError> {
        if depth > self.max_depth {
            return Err(PolicyError::MaxDepthExceeded(
                format!("Maximum evaluation depth {} exceeded", self.max_depth)
            ));
        }

        {
            let mut limiter = RATE_LIMITER.write();
            limiter.check_rate_limit()?;
        }

        context.validate()?;

        debug!("Evaluating policies for alert: {} (depth: {})", context.alert_id, depth);

        let policies = self.policy_loader.get_all_policies();
        let mut policy_rules: Vec<PolicyRule> = policies.iter()
            .filter(|p| p.enabled)
            .filter_map(|p| self.policy_loader.to_policy_rule(p).ok())
            .collect();

        self.precedence_rules.sort_by_precedence(&mut policy_rules);

        let mut matching_policies = Vec::new();

        for policy_rule in &policy_rules {
            match self.matcher.matches(policy_rule, context) {
                Ok(true) => {
                    matching_policies.push((policy_rule.clone(), 
                        self.policy_loader.get_policy(&policy_rule.id)
                            .ok()
                            .and_then(|p| p.signature.clone())
                            .unwrap_or_default()));
                    debug!("Policy {} matches context", policy_rule.id);
                }
                Ok(false) => {}
                Err(e) => {
                    warn!("Error matching policy {}: {}", policy_rule.id, e);
                }
            }
        }

        if matching_policies.is_empty() {
            warn!("No matching policy found for alert: {}", context.alert_id);
            return Err(PolicyError::NoMatchingPolicy(
                format!("No matching policy for alert: {}", context.alert_id)
            ));
        }

        if matching_policies.len() > 1 {
            let matching_rules: Vec<PolicyRule> = matching_policies.iter()
                .map(|(rule, _)| rule.clone())
                .collect();

            let conflicts = self.conflict_detector.detect_conflicts(&matching_rules)?;
            if !conflicts.is_empty() {
                for conflict in &conflicts {
                    if let Ok(resolution) = self.conflict_resolver.resolve(conflict, &matching_rules) {
                        match resolution {
                            crate::conflict::ConflictResolution::NoAction => {
                                error!("Unresolvable conflict: {}", conflict.policy_ids.join(", "));
                                return Err(PolicyError::PolicyAmbiguity(
                                    format!("Unresolvable conflict between policies: {}", 
                                        conflict.policy_ids.join(", "))
                                ));
                            }
                            crate::conflict::ConflictResolution::UseExplicitDeny => {
                                if let Some((rule, sig)) = matching_policies.iter()
                                    .find(|(r, _)| r.decision == AllowedAction::Deny) {
                                    return self.create_decision(context, rule, sig);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let highest_priority = matching_policies[0].0.priority;
            let same_priority: Vec<_> = matching_policies.iter()
                .filter(|(p, _)| p.priority == highest_priority)
                .collect();

            if same_priority.len() > 1 {
                error!("Policy ambiguity: {} policies match with same priority", same_priority.len());
                return Err(PolicyError::PolicyAmbiguity(
                    format!("Multiple policies match with same priority: {}", same_priority.len())
                ));
            }
        }

        let (policy_rule, policy_signature) = matching_policies.remove(0);
        self.create_decision(context, &policy_rule, &policy_signature)
    }

    fn create_decision(&self, context: &EvaluationContext, policy_rule: &PolicyRule, policy_signature: &str) -> Result<PolicyDecision, PolicyError> {
        let decision = PolicyDecision::new(
            &context.alert_id,
            &policy_rule.id,
            &policy_rule.version,
            policy_rule.decision.clone(),
            policy_rule.allowed_actions.clone(),
            policy_rule.required_approvals.clone(),
            &context.evidence_reference,
            &context.kill_chain_stage,
            &context.alert_severity,
            context.asset_class.clone().or_else(|| context.asset_id.clone()),
            &policy_rule.reasoning,
            policy_signature,
        );

        debug!("Policy decision created: {} (action: {:?})", decision.decision_id, decision.decision);
        Ok(decision)
    }
}

