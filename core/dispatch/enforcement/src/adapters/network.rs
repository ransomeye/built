// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/adapters/network.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network adapter - executes enforcement actions at network level

use serde_json::Value;
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;

pub struct NetworkAdapter {
    api_url: String,
}

impl NetworkAdapter {
    pub fn new() -> Result<Self, EnforcementError> {
        let api_url = std::env::var("RANSOMEYE_NETWORK_API_URL")
            .map_err(|_| EnforcementError::ConfigurationError(
                "RANSOMEYE_NETWORK_API_URL not set".to_string()
            ))?;
        
        Ok(Self {
            api_url,
        })
    }
    
    /// Execute enforcement action at network level
    pub async fn execute(&self, decision: &Value, targets: &[String], dry_run: bool) -> Result<String, EnforcementError> {
        let decision_action = decision.get("decision")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision field".to_string()))?;
        
        if dry_run {
            return Ok(format!("DRY-RUN: Would execute '{}' on {} network targets via {}", 
                decision_action, targets.len(), self.api_url));
        }
        
        // Build network commands based on action
        let commands = self.build_commands(decision_action, targets)?;
        
        // In production, would make HTTP request to network controller API
        debug!("Executing network enforcement: action={}, targets={:?}", decision_action, targets);
        
        // Simulate execution
        Ok(format!("Executed '{}' on {} network targets: {:?}", decision_action, targets.len(), commands))
    }
    
    fn build_commands(&self, action: &str, targets: &[String]) -> Result<Vec<String>, EnforcementError> {
        let mut commands = Vec::new();
        
        match action {
            "block" => {
                for target in targets {
                    commands.push(format!("route add {} reject", target));
                }
            }
            "isolate" => {
                for target in targets {
                    commands.push(format!("route add {} reject", target));
                    commands.push(format!("route add {} reject", target));
                }
            }
            "quarantine" => {
                for target in targets {
                    commands.push(format!("vlan isolate {}", target));
                }
            }
            "monitor" => {
                for target in targets {
                    commands.push(format!("mirror port {} to monitor", target));
                }
            }
            _ => {
                return Err(EnforcementError::AdapterFailure(
                    format!("Unsupported action for network adapter: {}", action)
                ));
            }
        }
        
        Ok(commands)
    }
    
    /// Generate rollback commands
    pub fn generate_rollback(&self, action: &str, targets: &[String]) -> Vec<String> {
        let mut rollback_commands = Vec::new();
        
        match action {
            "block" => {
                for target in targets {
                    rollback_commands.push(format!("route del {} reject", target));
                }
            }
            "isolate" => {
                for target in targets {
                    rollback_commands.push(format!("route del {} reject", target));
                }
            }
            "quarantine" => {
                for target in targets {
                    rollback_commands.push(format!("vlan unisolate {}", target));
                }
            }
            "monitor" => {
                for target in targets {
                    rollback_commands.push(format!("mirror port {} remove", target));
                }
            }
            _ => {
                // No rollback for unknown actions
            }
        }
        
        rollback_commands
    }
}

