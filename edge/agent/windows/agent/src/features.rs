// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/features.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Bounded feature extraction for telemetry events

use serde::{Serialize, Deserialize};
use tracing::debug;

use super::errors::AgentError;
use super::process::ProcessEvent;
use super::filesystem::FilesystemEvent;
use super::registry::RegistryEvent;
use super::network::NetworkEvent;

/// Bounded feature set
/// 
/// Lightweight feature extraction for telemetry events.
/// No AI inference, no enrichment - raw features only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    pub event_type: String,
    pub process_activity: bool,
    pub filesystem_activity: bool,
    pub registry_activity: bool,
    pub network_activity: bool,
    pub path_count: usize,
    pub has_command_line: bool,
    pub has_autorun: bool,
    pub has_persistence: bool,
}

impl Features {
    /// Extract features from process event
    pub fn from_process_event(event: &ProcessEvent) -> Self {
        Self {
            event_type: format!("{:?}", event.event_type),
            process_activity: true,
            filesystem_activity: false,
            registry_activity: false,
            network_activity: false,
            path_count: 0,
            has_command_line: event.command_line.is_some(),
            has_autorun: false,
            has_persistence: false,
        }
    }
    
    /// Extract features from filesystem event
    pub fn from_filesystem_event(event: &FilesystemEvent) -> Self {
        Self {
            event_type: format!("{:?}", event.event_type),
            process_activity: false,
            filesystem_activity: true,
            registry_activity: false,
            network_activity: false,
            path_count: 1 + if event.old_path.is_some() { 1 } else { 0 } + if event.new_path.is_some() { 1 } else { 0 },
            has_command_line: false,
            has_autorun: false,
            has_persistence: false,
        }
    }
    
    /// Extract features from registry event
    pub fn from_registry_event(event: &RegistryEvent) -> Self {
        let has_autorun = matches!(event.event_type, super::registry::RegistryEventType::AutorunCreate);
        let has_persistence = matches!(event.event_type, super::registry::RegistryEventType::PersistenceKeyCreate);
        
        Self {
            event_type: format!("{:?}", event.event_type),
            process_activity: false,
            filesystem_activity: false,
            registry_activity: true,
            network_activity: false,
            path_count: 1,
            has_command_line: false,
            has_autorun,
            has_persistence,
        }
    }
    
    /// Extract features from network event
    pub fn from_network_event(event: &NetworkEvent) -> Self {
        Self {
            event_type: format!("{:?}", event.event_type),
            process_activity: false,
            filesystem_activity: false,
            registry_activity: false,
            network_activity: true,
            path_count: 0,
            has_command_line: false,
            has_autorun: false,
            has_persistence: false,
        }
    }
    
    /// Validate feature bounds
    pub fn validate(&self) -> Result<(), AgentError> {
        // Ensure path_count is bounded
        if self.path_count > 1000 {
            return Err(AgentError::FeatureExtractionFailed(
                format!("Path count exceeds bounds: {}", self.path_count)
            ));
        }
        
        Ok(())
    }
}

