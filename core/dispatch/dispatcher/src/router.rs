// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/dispatcher/src/router.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Strict target resolution - no guessing, validate identity/capability

use tracing::{debug, error, warn};
use super::directive_envelope::{DirectiveEnvelope, TargetScope};
use super::errors::DispatcherError;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: String,
    pub platform: String,
    pub capabilities: Vec<String>,
    pub api_url: String,
    pub asset_class: Option<String>,
    pub environment: Option<String>,
}

pub struct TargetRouter {
    /// Registered agents by ID
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    
    /// Agents by platform
    agents_by_platform: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl TargetRouter {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            agents_by_platform: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an agent
    pub fn register_agent(&self, agent: AgentInfo) {
        let agent_id = agent.agent_id.clone();
        let platform = agent.platform.clone();
        
        {
            let mut agents = self.agents.write();
            agents.insert(agent_id.clone(), agent);
        }
        
        {
            let mut by_platform = self.agents_by_platform.write();
            by_platform
                .entry(platform)
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }
        
        debug!("Agent registered: {}", agent_id);
    }
    
    /// Resolve target agents for a directive
    /// Returns exact agent IDs - NO guessing, NO ambiguity
    pub fn resolve_targets(&self, directive: &DirectiveEnvelope) -> Result<Vec<String>, DispatcherError> {
        let scope = &directive.target_scope;
        let mut resolved_agents = Vec::new();
        
        // Case 1: Specific agent IDs provided
        if let Some(ref agent_ids) = scope.agent_ids {
            for agent_id in agent_ids {
                let agents = self.agents.read();
                let agent = agents.get(agent_id.as_str())
                    .ok_or_else(|| DispatcherError::AgentNotFound(agent_id.clone()))?;
                
                // Validate agent capability
                if !agent.capabilities.contains(&directive.action) {
                    return Err(DispatcherError::AgentCapabilityMismatch(
                        format!("Agent {} does not support action {}", agent_id, directive.action)
                    ));
                }
                
                // Validate platform compatibility
                if let Some(ref required_platform) = scope.platform {
                    if agent.platform != *required_platform {
                        return Err(DispatcherError::PlatformMismatch(
                            format!("Agent {} platform {} does not match required {}", 
                                agent_id, agent.platform, required_platform)
                        ));
                    }
                }
                
                // Validate asset class
                if let Some(ref required_asset_class) = scope.asset_class {
                    if let Some(ref agent_asset_class) = agent.asset_class {
                        if agent_asset_class != required_asset_class {
                            warn!("Asset class mismatch: agent {} has {}, required {}", 
                                agent_id, agent_asset_class, required_asset_class);
                        }
                    }
                }
                
                // Validate environment
                if let Some(ref required_env) = scope.environment {
                    if let Some(ref agent_env) = agent.environment {
                        if agent_env != required_env {
                            warn!("Environment mismatch: agent {} has {}, required {}", 
                                agent_id, agent_env, required_env);
                        }
                    }
                }
                
                resolved_agents.push(agent_id.clone());
            }
            
            if resolved_agents.is_empty() {
                return Err(DispatcherError::TargetResolutionFailed(
                    "No valid agents found for specified agent IDs".to_string()
                ));
            }
            
            debug!("Resolved {} agents by ID", resolved_agents.len());
            return Ok(resolved_agents);
        }
        
        // Case 2: Platform-based resolution
        if let Some(ref platform) = scope.platform {
            let by_platform = self.agents_by_platform.read();
            let platform_agents = by_platform.get(platform)
                .ok_or_else(|| DispatcherError::TargetResolutionFailed(
                    format!("No agents found for platform {}", platform)
                ))?;
            
            let agents = self.agents.read();
            for agent_id in platform_agents {
                let agent_id_clone = agent_id.clone();
                let agent = agents.get(agent_id)
                    .ok_or_else(|| DispatcherError::AgentNotFound(agent_id_clone))?;
                
                // Validate capability
                if !agent.capabilities.contains(&directive.action) {
                    continue; // Skip agents without required capability
                }
                
                // Validate asset class
                if let Some(ref required_asset_class) = scope.asset_class {
                    if let Some(ref agent_asset_class) = agent.asset_class {
                        if agent_asset_class != required_asset_class {
                            continue;
                        }
                    }
                }
                
                // Validate environment
                if let Some(ref required_env) = scope.environment {
                    if let Some(ref agent_env) = agent.environment {
                        if agent_env != required_env {
                            continue;
                        }
                    }
                }
                
                resolved_agents.push(agent_id.clone());
            }
            
            if resolved_agents.is_empty() {
                return Err(DispatcherError::TargetResolutionFailed(
                    format!("No agents found matching platform {} and requirements", platform)
                ));
            }
            
            debug!("Resolved {} agents by platform", resolved_agents.len());
            return Ok(resolved_agents);
        }
        
        // Case 3: Host address-based resolution (would require host-to-agent mapping)
        if let Some(ref _host_addresses) = scope.host_addresses {
            // This would require a host-to-agent mapping table
            // For now, return error if only host addresses provided
            return Err(DispatcherError::TargetResolutionFailed(
                "Host address resolution not implemented - specify agent_ids or platform".to_string()
            ));
        }
        
        // If no target scope specified, this is ambiguous - ABORT
        Err(DispatcherError::TargetResolutionFailed(
            "Target scope is ambiguous - must specify agent_ids, platform, or host_addresses".to_string()
        ))
    }
    
    /// Get agent info
    pub fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        self.agents.read().get(agent_id).cloned()
    }
}
