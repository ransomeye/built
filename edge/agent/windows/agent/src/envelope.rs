// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/envelope.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Phase-4 compliant event envelope creation

use serde::{Serialize, Deserialize};
use chrono::Utc;
use tracing::debug;

use super::errors::AgentError;
use super::process::ProcessEvent;
use super::filesystem::FilesystemEvent;
use super::registry::RegistryEvent;
use super::network::NetworkEvent;
use super::features::Features;

/// Phase-4 event envelope
/// 
/// Compliant with Phase 4 ingestion pipeline.
/// No enrichment, no inference, no policy logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub timestamp: String,
    pub component: String,
    pub component_id: String,
    pub event_type: String,
    pub sequence: u64,
    pub signature: String,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_category: String,
    pub pid: u32,
    pub process_data: Option<ProcessData>,
    pub filesystem_data: Option<FilesystemData>,
    pub registry_data: Option<RegistryData>,
    pub network_data: Option<NetworkData>,
    pub features: FeaturesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessData {
    pub event_type: String,
    pub ppid: Option<u32>,
    pub executable: Option<String>,
    pub command_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemData {
    pub event_type: String,
    pub path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub write_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryData {
    pub event_type: String,
    pub key_path: String,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkData {
    pub event_type: String,
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    pub local_addr: Option<String>,
    pub local_port: Option<u16>,
    pub protocol: String,
    pub bytes_transferred: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesData {
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

pub struct EnvelopeBuilder {
    component: String,
    component_id: String,
    sequence: u64,
}

impl EnvelopeBuilder {
    pub fn new(component: String, component_id: String) -> Self {
        Self {
            component,
            component_id,
            sequence: 0,
        }
    }
    
    /// Create Phase-4 event envelope from process event
    pub fn build_from_process(&mut self, event: &ProcessEvent, features: &Features, signature: String) -> Result<EventEnvelope, AgentError> {
        self.sequence += 1;
        
        let event_id = format!("windows-agent-{}-{}", self.component_id, self.sequence);
        let timestamp = Utc::now().to_rfc3339();
        
        let envelope = EventEnvelope {
            event_id,
            timestamp,
            component: self.component.clone(),
            component_id: self.component_id.clone(),
            event_type: "process_telemetry".to_string(),
            sequence: self.sequence,
            signature,
            data: EventData {
                event_category: "process".to_string(),
                pid: event.pid,
                process_data: Some(ProcessData {
                    event_type: format!("{:?}", event.event_type),
                    ppid: event.ppid,
                    executable: event.executable.clone(),
                    command_line: event.command_line.clone(),
                }),
                filesystem_data: None,
                registry_data: None,
                network_data: None,
                features: FeaturesData {
                    event_type: features.event_type.clone(),
                    process_activity: features.process_activity,
                    filesystem_activity: features.filesystem_activity,
                    registry_activity: features.registry_activity,
                    network_activity: features.network_activity,
                    path_count: features.path_count,
                    has_command_line: features.has_command_line,
                    has_autorun: features.has_autorun,
                    has_persistence: features.has_persistence,
                },
            },
        };
        
        debug!("Created process event envelope: {}", envelope.event_id);
        Ok(envelope)
    }
    
    /// Create Phase-4 event envelope from filesystem event
    pub fn build_from_filesystem(&mut self, event: &FilesystemEvent, features: &Features, signature: String) -> Result<EventEnvelope, AgentError> {
        self.sequence += 1;
        
        let event_id = format!("windows-agent-{}-{}", self.component_id, self.sequence);
        let timestamp = Utc::now().to_rfc3339();
        
        let envelope = EventEnvelope {
            event_id,
            timestamp,
            component: self.component.clone(),
            component_id: self.component_id.clone(),
            event_type: "filesystem_telemetry".to_string(),
            sequence: self.sequence,
            signature,
            data: EventData {
                event_category: "filesystem".to_string(),
                pid: event.pid,
                process_data: None,
                filesystem_data: Some(FilesystemData {
                    event_type: format!("{:?}", event.event_type),
                    path: event.path.clone(),
                    old_path: event.old_path.clone(),
                    new_path: event.new_path.clone(),
                    write_count: event.write_count,
                }),
                registry_data: None,
                network_data: None,
                features: FeaturesData {
                    event_type: features.event_type.clone(),
                    process_activity: features.process_activity,
                    filesystem_activity: features.filesystem_activity,
                    registry_activity: features.registry_activity,
                    network_activity: features.network_activity,
                    path_count: features.path_count,
                    has_command_line: features.has_command_line,
                    has_autorun: features.has_autorun,
                    has_persistence: features.has_persistence,
                },
            },
        };
        
        debug!("Created filesystem event envelope: {}", envelope.event_id);
        Ok(envelope)
    }
    
    /// Create Phase-4 event envelope from registry event
    pub fn build_from_registry(&mut self, event: &RegistryEvent, features: &Features, signature: String) -> Result<EventEnvelope, AgentError> {
        self.sequence += 1;
        
        let event_id = format!("windows-agent-{}-{}", self.component_id, self.sequence);
        let timestamp = Utc::now().to_rfc3339();
        
        let envelope = EventEnvelope {
            event_id,
            timestamp,
            component: self.component.clone(),
            component_id: self.component_id.clone(),
            event_type: "registry_telemetry".to_string(),
            sequence: self.sequence,
            signature,
            data: EventData {
                event_category: "registry".to_string(),
                pid: event.pid,
                process_data: None,
                filesystem_data: None,
                registry_data: Some(RegistryData {
                    event_type: format!("{:?}", event.event_type),
                    key_path: event.key_path.clone(),
                    value_name: event.value_name.clone(),
                    value_data: event.value_data.clone(),
                }),
                network_data: None,
                features: FeaturesData {
                    event_type: features.event_type.clone(),
                    process_activity: features.process_activity,
                    filesystem_activity: features.filesystem_activity,
                    registry_activity: features.registry_activity,
                    network_activity: features.network_activity,
                    path_count: features.path_count,
                    has_command_line: features.has_command_line,
                    has_autorun: features.has_autorun,
                    has_persistence: features.has_persistence,
                },
            },
        };
        
        debug!("Created registry event envelope: {}", envelope.event_id);
        Ok(envelope)
    }
    
    /// Create Phase-4 event envelope from network event
    pub fn build_from_network(&mut self, event: &NetworkEvent, features: &Features, signature: String) -> Result<EventEnvelope, AgentError> {
        self.sequence += 1;
        
        let event_id = format!("windows-agent-{}-{}", self.component_id, self.sequence);
        let timestamp = Utc::now().to_rfc3339();
        
        let envelope = EventEnvelope {
            event_id,
            timestamp,
            component: self.component.clone(),
            component_id: self.component_id.clone(),
            event_type: "network_telemetry".to_string(),
            sequence: self.sequence,
            signature,
            data: EventData {
                event_category: "network".to_string(),
                pid: event.pid,
                process_data: None,
                filesystem_data: None,
                registry_data: None,
                network_data: Some(NetworkData {
                    event_type: format!("{:?}", event.event_type),
                    remote_addr: event.remote_addr.clone(),
                    remote_port: event.remote_port,
                    local_addr: event.local_addr.clone(),
                    local_port: event.local_port,
                    protocol: event.protocol.clone(),
                    bytes_transferred: event.bytes_transferred,
                }),
                features: FeaturesData {
                    event_type: features.event_type.clone(),
                    process_activity: features.process_activity,
                    filesystem_activity: features.filesystem_activity,
                    registry_activity: features.registry_activity,
                    network_activity: features.network_activity,
                    path_count: features.path_count,
                    has_command_line: features.has_command_line,
                    has_autorun: features.has_autorun,
                    has_persistence: features.has_persistence,
                },
            },
        };
        
        debug!("Created network event envelope: {}", envelope.event_id);
        Ok(envelope)
    }
    
    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

