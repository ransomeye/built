// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/precedence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Explicit precedence rules for policy evaluation

use tracing::debug;
use crate::policy::PolicyRule;

pub struct PrecedenceRules;

impl PrecedenceRules {
    pub fn new() -> Self {
        Self
    }

    pub fn compare(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> std::cmp::Ordering {
        if policy1.priority != policy2.priority {
            return policy1.priority.cmp(&policy2.priority).reverse();
        }

        let specificity1 = self.compute_specificity(policy1);
        let specificity2 = self.compute_specificity(policy2);
        if specificity1 != specificity2 {
            return specificity1.cmp(&specificity2).reverse();
        }

        use crate::decision::AllowedAction;
        if policy1.decision == AllowedAction::Deny && policy2.decision != AllowedAction::Deny {
            return std::cmp::Ordering::Less;
        }
        if policy2.decision == AllowedAction::Deny && policy1.decision != AllowedAction::Deny {
            return std::cmp::Ordering::Greater;
        }

        policy1.id.cmp(&policy2.id)
    }

    fn compute_specificity(&self, policy: &PolicyRule) -> usize {
        policy.match_conditions.len()
    }

    pub fn sort_by_precedence(&self, policies: &mut [PolicyRule]) {
        policies.sort_by(|a, b| self.compare(a, b));
        debug!("Sorted {} policies by precedence", policies.len());
    }
}

