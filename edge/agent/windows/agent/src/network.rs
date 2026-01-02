// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/network.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network monitoring - socket/connect events (light)

#[cfg(windows)]
use winapi::um::winsock2::*;
#[cfg(windows)]
use winapi::um::ws2def::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Network event types
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkEventType {
    Connect,
    Disconnect,
    Send,
    Receive,
}

/// Network event
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub event_type: NetworkEventType,
    pub pid: u32,
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    pub local_addr: Option<String>,
    pub local_port: Option<u16>,
    pub protocol: String,
    pub bytes_transferred: Option<u64>,
    pub timestamp: u64,
}

/// Network monitor
/// 
/// Lightweight network monitoring: socket/connect events.
/// Bounded memory for connection tracking.
pub struct NetworkMonitor {
    connections: Arc<parking_lot::RwLock<std::collections::HashMap<u64, ConnectionInfo>>>,
    max_connections: usize,
    events_processed: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    pid: u32,
    remote_addr: String,
    remote_port: u16,
    local_addr: String,
    local_port: u16,
    protocol: String,
    created_at: u64,
}

impl NetworkMonitor {
    /// Create new network monitor
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            max_connections,
            events_processed: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Get network connect event
    pub fn get_connect_event(&self, pid: u32, remote_addr: String, remote_port: u16, local_addr: String, local_port: u16, protocol: String) -> Result<NetworkEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::NetworkMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        // Track connection
        let connection_id = self.generate_connection_id(&remote_addr, remote_port, &local_addr, local_port);
        let mut connections = self.connections.write();
        
        connections.insert(connection_id, ConnectionInfo {
            pid,
            remote_addr: remote_addr.clone(),
            remote_port,
            local_addr: local_addr.clone(),
            local_port,
            protocol: protocol.clone(),
            created_at: timestamp,
        });
        
        // Enforce memory bounds
        if connections.len() > self.max_connections {
            self.evict_oldest_connections(&mut connections);
        }
        
        self.events_processed.fetch_add(1, Ordering::AcqRel);
        
        Ok(NetworkEvent {
            event_type: NetworkEventType::Connect,
            pid,
            remote_addr: Some(remote_addr),
            remote_port: Some(remote_port),
            local_addr: Some(local_addr),
            local_port: Some(local_port),
            protocol,
            bytes_transferred: None,
            timestamp,
        })
    }
    
    /// Get network disconnect event
    pub fn get_disconnect_event(&self, remote_addr: String, remote_port: u16, local_addr: String, local_port: u16) -> Result<Option<NetworkEvent>, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::NetworkMonitoringFailed(
                format!("Failed to get timestamp: {}", e)
            ))?
            .as_secs();
        
        let connection_id = self.generate_connection_id(&remote_addr, remote_port, &local_addr, local_port);
        let mut connections = self.connections.write();
        
        if let Some(conn) = connections.remove(&connection_id) {
            self.events_processed.fetch_add(1, Ordering::AcqRel);
            
            return Ok(Some(NetworkEvent {
                event_type: NetworkEventType::Disconnect,
                pid: conn.pid,
                remote_addr: Some(conn.remote_addr),
                remote_port: Some(conn.remote_port),
                local_addr: Some(conn.local_addr),
                local_port: Some(conn.local_port),
                protocol: conn.protocol,
                bytes_transferred: None,
                timestamp,
            }));
        }
        
        Ok(None)
    }
    
    /// Generate connection ID
    fn generate_connection_id(&self, remote_addr: &str, remote_port: u16, local_addr: &str, local_port: u16) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        remote_addr.hash(&mut hasher);
        remote_port.hash(&mut hasher);
        local_addr.hash(&mut hasher);
        local_port.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Evict oldest connections to maintain memory bounds
    fn evict_oldest_connections(&self, connections: &mut std::collections::HashMap<u64, ConnectionInfo>) {
        if connections.len() <= self.max_connections {
            return;
        }
        
        let mut sorted: Vec<_> = connections.iter().collect();
        sorted.sort_by_key(|(_, info)| info.created_at);
        
        let to_remove = connections.len() - self.max_connections;
        // Collect keys first to avoid holding an immutable borrow of `connections`
        // while mutating it (Rust borrow rules).
        let ids_to_remove: Vec<u64> = sorted
            .iter()
            .take(to_remove)
            .map(|(id, _)| **id)
            .collect();
        for id in ids_to_remove {
            connections.remove(&id);
        }
        
        debug!("Evicted {} oldest connections", to_remove);
    }
    
    /// Get events processed count
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Acquire)
    }
}

