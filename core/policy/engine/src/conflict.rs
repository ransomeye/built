// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/conflict.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Deterministic conflict detection and resolution

use tracing::{error, debug, warn};
use crate::errors::PolicyError;
use crate::policy::PolicyRule;

#[derive(Debug, Clone)]
pub struct PolicyConflict {
    pub policy_ids: Vec<String>,
    pub conflict_type: ConflictType,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    SamePriority,
    ContradictoryActions,
    OverlappingScope,
    TimeBoundOverride,
}

#[derive(Debug, Clone)]
pub enum ConflictResolution {
    UseHighestPriority,
    UseMostSpecific,
    UseExplicitDeny,
    NoAction,
}

pub struct ConflictDetector;

impl ConflictDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_conflicts(&self, policies: &[PolicyRule]) -> Result<Vec<PolicyConflict>, PolicyError> {
        let mut conflicts = Vec::new();

        for i in 0..policies.len() {
            for j in (i + 1)..policies.len() {
                if let Some(conflict) = self.check_conflict(&policies[i], &policies[j]) {
                    conflicts.push(conflict);
                }
            }
        }

        if !conflicts.is_empty() {
            debug!("Detected {} conflicts", conflicts.len());
        }

        Ok(conflicts)
    }

    fn check_conflict(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> Option<PolicyConflict> {
        if policy1.priority == policy2.priority {
            if self.overlapping_scope(policy1, policy2) {
                return Some(PolicyConflict {
                    policy_ids: vec![policy1.id.clone(), policy2.id.clone()],
                    conflict_type: ConflictType::SamePriority,
                    resolution: None,
                });
            }
        }

        if self.contradictory_actions(policy1, policy2) && self.overlapping_scope(policy1, policy2) {
            return Some(PolicyConflict {
                policy_ids: vec![policy1.id.clone(), policy2.id.clone()],
                conflict_type: ConflictType::ContradictoryActions,
                resolution: None,
            });
        }

        None
    }

    fn overlapping_scope(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> bool {
        for cond1 in &policy1.match_conditions {
            for cond2 in &policy2.match_conditions {
                if cond1.field == cond2.field {
                    if self.conditions_overlap(cond1, cond2) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn conditions_overlap(&self, cond1: &crate::policy::PolicyMatchCondition, cond2: &crate::policy::PolicyMatchCondition) -> bool {
        if cond1.operator == "equals" && cond2.operator == "equals" {
            return cond1.value == cond2.value;
        }
        if cond1.operator == "in" && cond2.operator == "equals" {
            if let (Some(arr), Some(val)) = (cond1.value.as_array(), cond2.value.as_str()) {
                return arr.contains(&serde_json::Value::String(val.to_string()));
            }
        }
        if cond2.operator == "in" && cond1.operator == "equals" {
            if let (Some(arr), Some(val)) = (cond2.value.as_array(), cond1.value.as_str()) {
                return arr.contains(&serde_json::Value::String(val.to_string()));
            }
        }
        false
    }

    fn contradictory_actions(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> bool {
        use crate::decision::AllowedAction;
        matches!((&policy1.decision, &policy2.decision),
            (AllowedAction::Allow, AllowedAction::Deny) |
            (AllowedAction::Deny, AllowedAction::Allow) |
            (AllowedAction::Isolate, AllowedAction::Allow) |
            (AllowedAction::Allow, AllowedAction::Isolate)
        )
    }
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, conflict: &PolicyConflict, policies: &[PolicyRule]) -> Result<ConflictResolution, PolicyError> {
        match conflict.conflict_type {
            ConflictType::SamePriority => {
                warn!("Unresolvable conflict: policies with same priority");
                Ok(ConflictResolution::NoAction)
            }
            ConflictType::ContradictoryActions => {
                if self.has_explicit_deny(policies, &conflict.policy_ids) {
                    debug!("Resolving conflict using explicit deny precedence");
                    Ok(ConflictResolution::UseExplicitDeny)
                } else {
                    warn!("Unresolvable conflict: contradictory actions");
                    Ok(ConflictResolution::NoAction)
                }
            }
            ConflictType::OverlappingScope => {
                if self.has_more_specific(policies, &conflict.policy_ids) {
                    debug!("Resolving conflict using specificity");
                    Ok(ConflictResolution::UseMostSpecific)
                } else {
                    Ok(ConflictResolution::UseHighestPriority)
                }
            }
            ConflictType::TimeBoundOverride => {
                Ok(ConflictResolution::UseHighestPriority)
            }
        }
    }

    fn has_explicit_deny(&self, policies: &[PolicyRule], policy_ids: &[String]) -> bool {
        for policy in policies {
            if policy_ids.contains(&policy.id) {
                use crate::decision::AllowedAction;
                if policy.decision == AllowedAction::Deny {
                    return true;
                }
            }
        }
        false
    }

    fn has_more_specific(&self, policies: &[PolicyRule], policy_ids: &[String]) -> bool {
        let mut specificity: Vec<(usize, &PolicyRule)> = Vec::new();
        for policy in policies {
            if policy_ids.contains(&policy.id) {
                specificity.push((policy.match_conditions.len(), policy));
            }
        }
        specificity.len() > 1 && specificity.iter().any(|(s, _)| *s > specificity[0].0)
    }
}

