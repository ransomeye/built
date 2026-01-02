// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/matcher.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy matcher - deterministic policy matching

use serde_json::Value;
use tracing::{debug, warn};
use regex::Regex;

use crate::errors::PolicyError;
use crate::context::EvaluationContext;
use crate::policy::{PolicyRule, PolicyMatchCondition};

pub struct PolicyMatcher {
    regex_cache: parking_lot::RwLock<std::collections::HashMap<String, Regex>>,
}

impl PolicyMatcher {
    pub fn new() -> Self {
        Self {
            regex_cache: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn matches(&self, policy: &PolicyRule, context: &EvaluationContext) -> Result<bool, PolicyError> {
        debug!("Matching policy {} against context", policy.id);

        for condition in &policy.match_conditions {
            if !self.evaluate_condition(condition, context)? {
                debug!("Policy {} does not match: condition failed", policy.id);
                return Ok(false);
            }
        }

        debug!("Policy {} matches context", policy.id);
        Ok(true)
    }

    fn evaluate_condition(&self, condition: &PolicyMatchCondition, context: &EvaluationContext) -> Result<bool, PolicyError> {
        let field_value = context.get_field(&condition.field)
            .ok_or_else(|| PolicyError::MissingContext(
                format!("Field {} not found in context", condition.field)
            ))?;

        match condition.operator.as_str() {
            "equals" => {
                Ok(field_value == condition.value)
            }
            "contains" => {
                if let (Some(s), Some(v)) = (field_value.as_str(), condition.value.as_str()) {
                    Ok(s.contains(v))
                } else {
                    Ok(false)
                }
            }
            "matches" => {
                let pattern = condition.value.as_str()
                    .ok_or_else(|| PolicyError::EvaluationError(
                        "Regex pattern must be a string".to_string()
                    ))?;

                let regex = self.get_or_compile_regex(pattern)?;
                if let Some(s) = field_value.as_str() {
                    Ok(regex.is_match(s))
                } else {
                    Ok(false)
                }
            }
            "in" => {
                if let Some(arr) = condition.value.as_array() {
                    Ok(arr.contains(&field_value))
                } else {
                    Ok(false)
                }
            }
            "greater_than" => {
                if let (Some(a), Some(b)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(a > b)
                } else {
                    Ok(false)
                }
            }
            "less_than" => {
                if let (Some(a), Some(b)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(a < b)
                } else {
                    Ok(false)
                }
            }
            _ => {
                warn!("Unknown operator: {}", condition.operator);
                Err(PolicyError::EvaluationError(
                    format!("Unknown operator: {}", condition.operator)
                ))
            }
        }
    }

    fn get_or_compile_regex(&self, pattern: &str) -> Result<Regex, PolicyError> {
        {
            let cache = self.regex_cache.read();
            if let Some(regex) = cache.get(pattern) {
                return Ok(regex.clone());
            }
        }

        let regex = Regex::new(pattern)
            .map_err(|e| PolicyError::EvaluationError(
                format!("Invalid regex pattern {}: {}", pattern, e)
            ))?;

        {
            let mut cache = self.regex_cache.write();
            cache.insert(pattern.to_string(), regex.clone());
        }

        Ok(regex)
    }
}

