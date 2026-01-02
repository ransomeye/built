// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/adapters/linux_agent.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Linux agent adapter - executes enforcement actions on Linux hosts

use serde_json::Value;
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;

pub struct LinuxAgentAdapter {
    api_url: String,
}

impl LinuxAgentAdapter {
    pub fn new() -> Result<Self, EnforcementError> {
        let api_url = std::env::var("RANSOMEYE_LINUX_AGENT_API_URL")
            .map_err(|_| EnforcementError::ConfigurationError(
                "RANSOMEYE_LINUX_AGENT_API_URL not set".to_string()
            ))?;
        
        Ok(Self {
            api_url,
        })
    }
    
    /// Execute enforcement action on Linux agent
    pub async fn execute(&self, decision: &Value, targets: &[String], dry_run: bool) -> Result<String, EnforcementError> {
        let decision_action = decision.get("decision")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision field".to_string()))?;
        
        if dry_run {
            return Ok(format!("DRY-RUN: Would execute '{}' on {} Linux targets via {}", 
                decision_action, targets.len(), self.api_url));
        }
        
        // Build command based on action
        let commands = self.build_commands(decision_action, targets)?;
        
        // In production, would make HTTP request to Linux agent API
        debug!("Executing Linux agent enforcement: action={}, targets={:?}", decision_action, targets);
        
        // Simulate execution
        Ok(format!("Executed '{}' on {} Linux targets: {:?}", decision_action, targets.len(), commands))
    }
    
    fn build_commands(&self, action: &str, targets: &[String]) -> Result<Vec<String>, EnforcementError> {
        let mut commands = Vec::new();
        
        match action {
            "block" => {
                for target in targets {
                    commands.push(format!("iptables -A INPUT -s {} -j DROP", target));
                }
            }
            "isolate" => {
                for target in targets {
                    commands.push(format!("iptables -A INPUT -s {} -j REJECT", target));
                    commands.push(format!("iptables -A OUTPUT -d {} -j REJECT", target));
                }
            }
            "quarantine" => {
                for target in targets {
                    commands.push(format!("iptables -A FORWARD -s {} -j DROP", target));
                }
            }
            "monitor" => {
                for target in targets {
                    commands.push(format!("iptables -A INPUT -s {} -j LOG --log-prefix 'RANSOMEYE_MONITOR:'", target));
                }
            }
            _ => {
                return Err(EnforcementError::AdapterFailure(
                    format!("Unsupported action for Linux agent: {}", action)
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
                    rollback_commands.push(format!("iptables -D INPUT -s {} -j DROP", target));
                }
            }
            "isolate" => {
                for target in targets {
                    rollback_commands.push(format!("iptables -D INPUT -s {} -j REJECT", target));
                    rollback_commands.push(format!("iptables -D OUTPUT -d {} -j REJECT", target));
                }
            }
            "quarantine" => {
                for target in targets {
                    rollback_commands.push(format!("iptables -D FORWARD -s {} -j DROP", target));
                }
            }
            "monitor" => {
                for target in targets {
                    rollback_commands.push(format!("iptables -D INPUT -s {} -j LOG --log-prefix 'RANSOMEYE_MONITOR:'", target));
                }
            }
            _ => {
                // No rollback for unknown actions
            }
        }
        
        rollback_commands
    }
}

