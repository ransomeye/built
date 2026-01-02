// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/guardrails.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Safety guardrails enforcement - max hosts, asset restrictions, environment constraints

use std::env;
use serde_json::Value;
use tracing::{warn, debug};
use crate::errors::EnforcementError;

pub struct Guardrails {
    max_hosts_per_action: usize,
    max_actions_per_window: usize,
    allowed_asset_classes: Vec<String>,
    allowed_environments: Vec<String>,
    destructive_actions_require_approval: bool,
}

impl Guardrails {
    pub fn new() -> Self {
        let max_hosts = env::var("RANSOMEYE_ENFORCEMENT_MAX_HOSTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<usize>()
            .unwrap_or(10);
        
        let max_actions = env::var("RANSOMEYE_ENFORCEMENT_MAX_ACTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        
        let asset_classes = env::var("RANSOMEYE_ENFORCEMENT_ALLOWED_ASSET_CLASSES")
            .unwrap_or_else(|_| "production,staging,development".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let environments = env::var("RANSOMEYE_ENFORCEMENT_ALLOWED_ENVIRONMENTS")
            .unwrap_or_else(|_| "production,staging,development".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        Self {
            max_hosts_per_action: max_hosts,
            max_actions_per_window: max_actions,
            allowed_asset_classes: asset_classes,
            allowed_environments: environments,
            destructive_actions_require_approval: true,
        }
    }
    
    /// Check guardrails for a decision
    pub fn check(&self, decision: &Value, target_count: usize) -> Result<Vec<String>, EnforcementError> {
        let mut checks = Vec::new();
        
        // Check max hosts per action
        if target_count > self.max_hosts_per_action {
            return Err(EnforcementError::GuardrailViolation(
                format!("Target count {} exceeds max hosts per action {}", target_count, self.max_hosts_per_action)
            ));
        }
        checks.push(format!("Target count {} within limit {}", target_count, self.max_hosts_per_action));
        
        // Check asset class restrictions
        if let Some(asset_class) = decision.get("asset_class").and_then(|v| v.as_str()) {
            if !self.allowed_asset_classes.contains(&asset_class.to_string()) {
                return Err(EnforcementError::GuardrailViolation(
                    format!("Asset class '{}' not in allowed list: {:?}", asset_class, self.allowed_asset_classes)
                ));
            }
            checks.push(format!("Asset class '{}' is allowed", asset_class));
        }
        
        // Check for destructive actions
        if let Some(decision_action) = decision.get("decision").and_then(|v| v.as_str()) {
            let destructive_actions = vec!["block", "isolate", "quarantine", "deny"];
            if destructive_actions.contains(&decision_action) {
                checks.push(format!("Destructive action '{}' detected", decision_action));
                if self.destructive_actions_require_approval {
                    // This will be checked by approvals module
                    checks.push("Destructive action requires approval".to_string());
                }
            }
        }
        
        debug!("Guardrail checks passed: {:?}", checks);
        Ok(checks)
    }
    
    /// Check if action is destructive
    pub fn is_destructive(&self, action: &str) -> bool {
        let destructive_actions = vec!["block", "isolate", "quarantine", "deny"];
        destructive_actions.contains(&action)
    }
}

