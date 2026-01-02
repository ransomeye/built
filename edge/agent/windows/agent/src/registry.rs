// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/registry.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Registry monitoring - autoruns, persistence keys

#[cfg(windows)]
use winapi::um::winreg::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn, info, debug};

use super::errors::AgentError;

/// Registry event types
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryEventType {
    KeyCreate,
    KeyDelete,
    ValueSet,
    AutorunCreate,
    PersistenceKeyCreate,
}

/// Registry event
#[derive(Debug, Clone)]
pub struct RegistryEvent {
    pub event_type: RegistryEventType,
    pub key_path: String,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
    pub pid: u32,
    pub timestamp: u64,
}

/// Registry monitor
/// 
/// Tracks registry events: autoruns, persistence keys.
/// Focuses on security-relevant registry locations.
pub struct RegistryMonitor {
    monitored_keys: Vec<String>,
    events_processed: Arc<AtomicU64>,
}

impl RegistryMonitor {
    /// Create new registry monitor
    pub fn new(monitored_keys: Vec<String>) -> Self {
        Self {
            monitored_keys,
            events_processed: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Get registry key create event
    pub fn get_key_create_event(&self, key_path: String, pid: u32) -> Result<RegistryEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::RegistryMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let event_type = if self.is_autorun_key(&key_path) {
            RegistryEventType::AutorunCreate
        } else if self.is_persistence_key(&key_path) {
            RegistryEventType::PersistenceKeyCreate
        } else {
            RegistryEventType::KeyCreate
        };
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(RegistryEvent {
            event_type,
            key_path,
            value_name: None,
            value_data: None,
            pid,
            timestamp,
        })
    }
    
    /// Get registry key delete event
    pub fn get_key_delete_event(&self, key_path: String, pid: u32) -> Result<RegistryEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::RegistryMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(RegistryEvent {
            event_type: RegistryEventType::KeyDelete,
            key_path,
            value_name: None,
            value_data: None,
            pid,
            timestamp,
        })
    }
    
    /// Get registry value set event
    pub fn get_value_set_event(&self, key_path: String, value_name: String, value_data: String, pid: u32) -> Result<RegistryEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::RegistryMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let event_type = if self.is_autorun_key(&key_path) {
            RegistryEventType::AutorunCreate
        } else if self.is_persistence_key(&key_path) {
            RegistryEventType::PersistenceKeyCreate
        } else {
            RegistryEventType::ValueSet
        };
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(RegistryEvent {
            event_type,
            key_path,
            value_name: Some(value_name),
            value_data: Some(value_data),
            pid,
            timestamp,
        })
    }
    
    /// Check if key is an autorun key
    fn is_autorun_key(&self, key_path: &str) -> bool {
        let autorun_keys = vec![
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
        ];
        
        autorun_keys.iter().any(|k| key_path.contains(k))
    }
    
    /// Check if key is a persistence key
    fn is_persistence_key(&self, key_path: &str) -> bool {
        let persistence_keys = vec![
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
            "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services",
        ];
        
        persistence_keys.iter().any(|k| key_path.contains(k))
    }
    
    /// Get events processed count
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Acquire)
    }
}

