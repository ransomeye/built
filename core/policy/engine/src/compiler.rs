// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/compiler.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy language compiler - validates and compiles policies

use tracing::{error, debug};
use crate::errors::PolicyError;
use crate::policy::Policy;

pub struct PolicyCompiler;

impl PolicyCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, policy: &Policy) -> Result<(), PolicyError> {
        if policy.id.is_empty() {
            return Err(PolicyError::ConfigurationError(
                "Policy ID cannot be empty".to_string()
            ));
        }

        if policy.version.is_empty() {
            return Err(PolicyError::ConfigurationError(
                "Policy version cannot be empty".to_string()
            ));
        }

        if policy.match_conditions.is_empty() {
            return Err(PolicyError::ConfigurationError(
                format!("Policy {} has no match conditions", policy.id)
            ));
        }

        if policy.decision.action.is_empty() {
            return Err(PolicyError::ConfigurationError(
                format!("Policy {} has no decision action", policy.id)
            ));
        }

        for condition in &policy.match_conditions {
            if condition.field.is_empty() {
                return Err(PolicyError::ConfigurationError(
                    format!("Policy {} has empty field in match condition", policy.id)
                ));
            }
            if condition.operator.is_empty() {
                return Err(PolicyError::ConfigurationError(
                    format!("Policy {} has empty operator in match condition", policy.id)
                ));
            }
        }

        debug!("Policy {} validated successfully", policy.id);
        Ok(())
    }

    pub fn compile(&self, policy: &Policy) -> Result<(), PolicyError> {
        self.validate(policy)?;

        if policy.signature.is_none() {
            return Err(PolicyError::UnsignedPolicy(
                format!("Policy {} is not signed", policy.id)
            ));
        }

        debug!("Policy {} compiled successfully", policy.id);
        Ok(())
    }
}

