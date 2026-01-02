// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/kill_chain/transitions.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Kill-chain transition rules - defines allowed and forbidden transitions

use crate::kill_chain::stages::RansomwareStage;
use std::collections::{HashMap, HashSet};

/// Transition validation result
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionResult {
    /// Transition is allowed
    Allowed,
    /// Transition is forbidden (stage skip without evidence)
    Forbidden(String),
    /// Transition requires additional evidence
    RequiresEvidence(Vec<String>),
}

/// Kill-chain transition rules
pub struct TransitionRules {
    /// Allowed transitions: from_stage -> to_stage
    allowed_transitions: HashMap<RansomwareStage, HashSet<RansomwareStage>>,
    /// Forbidden transitions (explicitly disallowed)
    forbidden_transitions: HashSet<(RansomwareStage, RansomwareStage)>,
}

impl TransitionRules {
    /// Create default transition rules for ransomware kill-chain
    pub fn default() -> Self {
        let mut allowed = HashMap::new();
        
        // Sequential transitions (normal progression)
        let stages = RansomwareStage::all_stages();
        for i in 0..stages.len() {
            let from = stages[i];
            let mut to_set = HashSet::new();
            
            // Allow next stage
            if i + 1 < stages.len() {
                to_set.insert(stages[i + 1]);
            }
            
            // Allow self (staying in same stage with new evidence)
            to_set.insert(from);
            
            // Some stages allow skipping (with evidence)
            match from {
                RansomwareStage::InitialAccess => {
                    // Can go directly to Execution
                    to_set.insert(RansomwareStage::Execution);
                }
                RansomwareStage::Execution => {
                    // Can skip Persistence if privilege escalation happens immediately
                    to_set.insert(RansomwareStage::PrivilegeEscalation);
                    // Can also go to EncryptionExecution with strong evidence (file_modification + encryption_activity)
                    to_set.insert(RansomwareStage::EncryptionExecution);
                }
                RansomwareStage::Discovery => {
                    // Can go directly to EncryptionPreparation
                    to_set.insert(RansomwareStage::EncryptionPreparation);
                    // Can also go directly to EncryptionExecution with strong evidence
                    to_set.insert(RansomwareStage::EncryptionExecution);
                }
                RansomwareStage::EncryptionPreparation => {
                    // Can go to EncryptionExecution
                    to_set.insert(RansomwareStage::EncryptionExecution);
                }
                RansomwareStage::EncryptionExecution => {
                    // Can go to Impact
                    to_set.insert(RansomwareStage::Impact);
                }
                // Allow EncryptionExecution from any earlier stage with strong evidence
                RansomwareStage::Persistence | 
                RansomwareStage::PrivilegeEscalation |
                RansomwareStage::LateralMovement |
                RansomwareStage::CredentialAccess => {
                    // Can jump to EncryptionExecution with strong evidence (ransomware pattern)
                    to_set.insert(RansomwareStage::EncryptionExecution);
                }
                _ => {}
            }
            
            allowed.insert(from, to_set);
        }
        
        // Forbidden transitions (regression or impossible jumps)
        let mut forbidden = HashSet::new();
        
        // No regression allowed (except staying in same stage)
        for i in 0..stages.len() {
            for j in 0..i {
                if stages[i] != stages[j] {
                    forbidden.insert((stages[i], stages[j]));
                }
            }
        }
        
        // Forbidden large jumps (require evidence)
        forbidden.insert((RansomwareStage::InitialAccess, RansomwareStage::LateralMovement));
        forbidden.insert((RansomwareStage::InitialAccess, RansomwareStage::EncryptionExecution));
        // Execution -> EncryptionExecution is allowed with strong evidence (file_modification + encryption_activity)
        
        Self {
            allowed_transitions: allowed,
            forbidden_transitions: forbidden,
        }
    }

    /// Validate if transition from `from` to `to` is allowed
    pub fn validate_transition(
        &self,
        from: Option<RansomwareStage>,
        to: RansomwareStage,
        has_evidence: bool,
    ) -> TransitionResult {
        // If no previous stage, only InitialAccess is allowed
        let from_stage = match from {
            None => {
                return if to == RansomwareStage::InitialAccess {
                    TransitionResult::Allowed
                } else {
                    TransitionResult::Forbidden("Must start with InitialAccess".to_string())
                };
            }
            Some(s) => s,
        };

        // Check forbidden transitions first
        if self.forbidden_transitions.contains(&(from_stage, to)) {
            return TransitionResult::Forbidden(format!(
                "Transition from {:?} to {:?} is forbidden",
                from_stage, to
            ));
        }

        // Check if transition is explicitly allowed
        if let Some(allowed_set) = self.allowed_transitions.get(&from_stage) {
            if allowed_set.contains(&to) {
                // Sequential transition or explicitly allowed
                if from_stage.index() + 1 == to.index() {
                    // Sequential - always allowed
                    return TransitionResult::Allowed;
                } else {
                    // Non-sequential - requires evidence
                    if has_evidence {
                        return TransitionResult::Allowed;
                    } else {
                        return TransitionResult::RequiresEvidence(vec![
                            format!("Evidence required for transition from {:?} to {:?}", from_stage, to)
                        ]);
                    }
                }
            }
        }

        // Default: transition not in allowed set
        TransitionResult::Forbidden(format!(
            "Transition from {:?} to {:?} is not allowed",
            from_stage, to
        ))
    }

    /// Get all allowed next stages from a given stage
    pub fn get_allowed_next_stages(&self, from: RansomwareStage) -> Vec<RansomwareStage> {
        self.allowed_transitions
            .get(&from)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_transitions() {
        let rules = TransitionRules::default();
        let stages = RansomwareStage::all_stages();
        
        for i in 0..stages.len() - 1 {
            let from = stages[i];
            let to = stages[i + 1];
            let result = rules.validate_transition(Some(from), to, false);
            assert_eq!(result, TransitionResult::Allowed, 
                "Sequential transition from {:?} to {:?} should be allowed", from, to);
        }
    }

    #[test]
    fn test_regression_forbidden() {
        let rules = TransitionRules::default();
        let stages = RansomwareStage::all_stages();
        
        // Regression should be forbidden
        for i in 1..stages.len() {
            for j in 0..i {
                if stages[i] != stages[j] {
                    let result = rules.validate_transition(Some(stages[i]), stages[j], false);
                    assert!(matches!(result, TransitionResult::Forbidden(_)),
                        "Regression from {:?} to {:?} should be forbidden", stages[i], stages[j]);
                }
            }
        }
    }

    #[test]
    fn test_initial_access_only_start() {
        let rules = TransitionRules::default();
        
        // Can start with InitialAccess
        assert_eq!(
            rules.validate_transition(None, RansomwareStage::InitialAccess, false),
            TransitionResult::Allowed
        );
        
        // Cannot start with other stages
        for stage in RansomwareStage::all_stages() {
            if stage != RansomwareStage::InitialAccess {
                assert!(matches!(
                    rules.validate_transition(None, stage, false),
                    TransitionResult::Forbidden(_)
                ));
            }
        }
    }
}

