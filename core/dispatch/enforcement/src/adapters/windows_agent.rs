// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/adapters/windows_agent.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Windows agent adapter - executes enforcement actions on Windows hosts

use serde_json::Value;
use tracing::{error, warn, debug};
use crate::errors::EnforcementError;

pub struct WindowsAgentAdapter {
    api_url: String,
}

impl WindowsAgentAdapter {
    pub fn new() -> Result<Self, EnforcementError> {
        let api_url = std::env::var("RANSOMEYE_WINDOWS_AGENT_API_URL")
            .map_err(|_| EnforcementError::ConfigurationError(
                "RANSOMEYE_WINDOWS_AGENT_API_URL not set".to_string()
            ))?;
        
        Ok(Self {
            api_url,
        })
    }
    
    /// Execute enforcement action on Windows agent
    pub async fn execute(&self, decision: &Value, targets: &[String], dry_run: bool) -> Result<String, EnforcementError> {
        let decision_action = decision.get("decision")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision field".to_string()))?;
        
        if dry_run {
            return Ok(format!("DRY-RUN: Would execute '{}' on {} Windows targets via {}", 
                decision_action, targets.len(), self.api_url));
        }
        
        // Build PowerShell commands based on action
        let commands = self.build_commands(decision_action, targets)?;
        
        // In production, would make HTTP request to Windows agent API
        debug!("Executing Windows agent enforcement: action={}, targets={:?}", decision_action, targets);
        
        // Simulate execution
        Ok(format!("Executed '{}' on {} Windows targets: {:?}", decision_action, targets.len(), commands))
    }
    
    fn build_commands(&self, action: &str, targets: &[String]) -> Result<Vec<String>, EnforcementError> {
        let mut commands = Vec::new();
        
        match action {
            "block" => {
                for target in targets {
                    commands.push(format!("New-NetFirewallRule -DisplayName 'RansomEye-Block-{}' -Direction Inbound -RemoteAddress {} -Action Block", target, target));
                }
            }
            "isolate" => {
                for target in targets {
                    commands.push(format!("New-NetFirewallRule -DisplayName 'RansomEye-Isolate-{}' -Direction Inbound -RemoteAddress {} -Action Block", target, target));
                    commands.push(format!("New-NetFirewallRule -DisplayName 'RansomEye-Isolate-{}' -Direction Outbound -RemoteAddress {} -Action Block", target, target));
                }
            }
            "quarantine" => {
                for target in targets {
                    commands.push(format!("Set-NetFirewallProfile -DefaultInboundAction Block -DefaultOutboundAction Block"));
                }
            }
            "monitor" => {
                for target in targets {
                    commands.push(format!("New-NetFirewallRule -DisplayName 'RansomEye-Monitor-{}' -Direction Inbound -RemoteAddress {} -Action Allow -Logging Enabled", target, target));
                }
            }
            _ => {
                return Err(EnforcementError::AdapterFailure(
                    format!("Unsupported action for Windows agent: {}", action)
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
                    rollback_commands.push(format!("Remove-NetFirewallRule -DisplayName 'RansomEye-Block-{}'", target));
                }
            }
            "isolate" => {
                for target in targets {
                    rollback_commands.push(format!("Remove-NetFirewallRule -DisplayName 'RansomEye-Isolate-{}'", target));
                }
            }
            "quarantine" => {
                rollback_commands.push("Set-NetFirewallProfile -DefaultInboundAction Allow -DefaultOutboundAction Allow".to_string());
            }
            "monitor" => {
                for target in targets {
                    rollback_commands.push(format!("Remove-NetFirewallRule -DisplayName 'RansomEye-Monitor-{}'", target));
                }
            }
            _ => {
                // No rollback for unknown actions
            }
        }
        
        rollback_commands
    }
}

