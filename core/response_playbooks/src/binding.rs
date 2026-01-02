// Path and File Name : /home/ransomeye/rebuild/core/response_playbooks/src/binding.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy → Playbook binding - explicit mapping, no implicit actions, fail-closed on missing binding

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, info};

use crate::errors::PlaybookError;
use crate::registry::PlaybookRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPlaybookBinding {
    pub policy_outcome: String,
    pub alert_severity: Option<String>,
    pub kill_chain_stage: Option<String>,
    pub playbook_id: String,
    pub priority: u32,
}

pub struct PolicyPlaybookBindingManager {
    registry: Arc<PlaybookRegistry>,
    bindings: Arc<RwLock<HashMap<String, Vec<PolicyPlaybookBinding>>>>,
    binding_file_path: String,
}

impl PolicyPlaybookBindingManager {
    pub fn new(
        registry: Arc<PlaybookRegistry>,
        binding_file_path: &str,
    ) -> Result<Self, PlaybookError> {
        let manager = Self {
            registry,
            bindings: Arc::new(RwLock::new(HashMap::new())),
            binding_file_path: binding_file_path.to_string(),
        };
        
        manager.reload()?;
        Ok(manager)
    }
    
    /// Reload bindings from file
    pub fn reload(&self) -> Result<(), PlaybookError> {
        use std::fs;
        
        if !std::path::Path::new(&self.binding_file_path).exists() {
            warn!("Binding file does not exist: {}, creating empty bindings", self.binding_file_path);
            let mut bindings = self.bindings.write();
            bindings.clear();
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.binding_file_path)
            .map_err(|e| PlaybookError::ConfigurationError(
                format!("Failed to read binding file {}: {}", self.binding_file_path, e)
            ))?;
        
        let bindings_list: Vec<PolicyPlaybookBinding> = serde_yaml::from_str(&content)
            .map_err(|e| PlaybookError::ConfigurationError(
                format!("Failed to parse binding file: {}", e)
            ))?;
        
        // Validate all bindings reference existing playbooks
        let mut validated_bindings: HashMap<String, Vec<PolicyPlaybookBinding>> = HashMap::new();
        
        for binding in bindings_list {
            // Verify playbook exists
            if !self.registry.has_playbook(&binding.playbook_id) {
                error!("Binding references non-existent playbook: {}", binding.playbook_id);
                return Err(PlaybookError::PolicyBindingNotFound(
                    format!("Playbook {} not found in registry", binding.playbook_id)
                ));
            }
            
            // Group by policy_outcome
            let key = binding.policy_outcome.clone();
            validated_bindings.entry(key).or_insert_with(Vec::new).push(binding);
        }
        
        // Sort bindings by priority (higher priority first)
        for bindings in validated_bindings.values_mut() {
            bindings.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        let mut stored_bindings = self.bindings.write();
        *stored_bindings = validated_bindings;
        
        info!("Loaded {} policy-playbook bindings", stored_bindings.len());
        Ok(())
    }
    
    /// Find playbook for policy outcome
    pub fn find_playbook(
        &self,
        policy_outcome: &str,
        alert_severity: Option<&str>,
        kill_chain_stage: Option<&str>,
    ) -> Result<Option<String>, PlaybookError> {
        let bindings = self.bindings.read();
        
        // Get bindings for this policy outcome
        let candidate_bindings = bindings.get(policy_outcome);
        
        if candidate_bindings.is_none() || candidate_bindings.unwrap().is_empty() {
            // No binding found → NO ACTION (fail-closed)
            warn!("No playbook binding found for policy outcome: {}", policy_outcome);
            return Ok(None);
        }
        
        // Find best matching binding
        for binding in candidate_bindings.unwrap() {
            // Check severity match if specified
            if let Some(sev) = alert_severity {
                if let Some(ref binding_sev) = binding.alert_severity {
                    if sev != binding_sev {
                        continue;
                    }
                }
            }
            
            // Check kill chain stage match if specified
            if let Some(kcs) = kill_chain_stage {
                if let Some(ref binding_kcs) = binding.kill_chain_stage {
                    if kcs != binding_kcs {
                        continue;
                    }
                }
            }
            
            // Match found
            info!("Found playbook binding: {} -> {}", policy_outcome, binding.playbook_id);
            return Ok(Some(binding.playbook_id.clone()));
        }
        
        // No exact match found → NO ACTION
        warn!("No exact playbook binding match for policy outcome: {} (severity: {:?}, kill_chain: {:?})",
              policy_outcome, alert_severity, kill_chain_stage);
        Ok(None)
    }
    
    /// Add binding (for testing/admin)
    pub fn add_binding(&self, binding: PolicyPlaybookBinding) -> Result<(), PlaybookError> {
        // Verify playbook exists
        if !self.registry.has_playbook(&binding.playbook_id) {
            return Err(PlaybookError::PolicyBindingNotFound(
                format!("Playbook {} not found", binding.playbook_id)
            ));
        }
        
        let mut bindings = self.bindings.write();
        let key = binding.policy_outcome.clone();
        bindings.entry(key).or_insert_with(Vec::new).push(binding);
        
        // Persist to file
        self.save_bindings()?;
        
        Ok(())
    }
    
    /// Save bindings to file
    fn save_bindings(&self) -> Result<(), PlaybookError> {
        use std::fs;
        
        let bindings = self.bindings.read();
        let mut all_bindings: Vec<PolicyPlaybookBinding> = Vec::new();
        
        for bindings_list in bindings.values() {
            all_bindings.extend(bindings_list.clone());
        }
        
        let yaml_content = serde_yaml::to_string(&all_bindings)
            .map_err(|e| PlaybookError::InternalError(
                format!("Failed to serialize bindings: {}", e)
            ))?;
        
        fs::write(&self.binding_file_path, yaml_content)
            .map_err(|e| PlaybookError::ConfigurationError(
                format!("Failed to write binding file: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// List all bindings
    pub fn list_bindings(&self) -> Vec<PolicyPlaybookBinding> {
        let bindings = self.bindings.read();
        let mut all_bindings: Vec<PolicyPlaybookBinding> = Vec::new();
        
        for bindings_list in bindings.values() {
            all_bindings.extend(bindings_list.clone());
        }
        
        all_bindings
    }
}

// Export the manager as the main type
pub use PolicyPlaybookBindingManager as PolicyPlaybookBinding;

