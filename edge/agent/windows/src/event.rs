// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/event.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Event structures for Windows Agent telemetry (process, file, registry, auth, network)

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_type: String,
    pub pid: u32,
    pub ppid: u32,
    pub process_name: String,
    pub command_line: String,
    pub user_sid: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub event_type: String,
    pub path: String,
    pub operation: String,
    pub process_id: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub event_type: String,
    pub key_path: String,
    pub operation: String,
    pub value_name: String,
    pub process_id: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    pub event_type: String,
    pub user: String,
    pub source: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub event_type: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub process_id: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AgentEvent {
    #[serde(rename = "process")]
    Process(ProcessEvent),
    #[serde(rename = "file")]
    File(FileEvent),
    #[serde(rename = "registry")]
    Registry(RegistryEvent),
    #[serde(rename = "auth")]
    Auth(AuthEvent),
    #[serde(rename = "network")]
    Network(NetworkEvent),
}

impl AgentEvent {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
    
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AgentEvent::Process(e) => e.timestamp,
            AgentEvent::File(e) => e.timestamp,
            AgentEvent::Registry(e) => e.timestamp,
            AgentEvent::Auth(e) => e.timestamp,
            AgentEvent::Network(e) => e.timestamp,
        }
    }
}
