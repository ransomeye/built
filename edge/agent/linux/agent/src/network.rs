// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/network.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network monitoring - socket operations (light)

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use super::errors::AgentError;

/// Network event types
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkEventType {
    SocketCreate,
    SocketConnect,
    SocketBind,
    SocketListen,
    SocketAccept,
    SocketSend,
    SocketRecv,
}

/// Network event
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub event_type: NetworkEventType,
    pub pid: u32,
    pub uid: u32,
    pub gid: u32,
    pub socket_family: u32, // AF_INET, AF_INET6, etc.
    pub socket_type: u32,   // SOCK_STREAM, SOCK_DGRAM, etc.
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    pub local_addr: Option<String>,
    pub local_port: Option<u16>,
    pub bytes_transferred: Option<u64>,
    pub timestamp: u64,
}

/// Network monitor
/// 
/// Tracks network socket operations (light monitoring).
/// Bounded memory for connection tracking.
pub struct NetworkMonitor {
    connections: Arc<parking_lot::RwLock<std::collections::HashMap<u64, ConnectionInfo>>>,
    max_connections: usize,
    events_processed: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    pid: u32,
    socket_fd: i32,
    family: u32,
    socket_type: u32,
    remote_addr: Option<String>,
    remote_port: Option<u16>,
    local_addr: Option<String>,
    local_port: Option<u16>,
    first_seen: u64,
    last_seen: u64,
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
    
    /// Record socket create event
    pub fn record_socket_create(&self, pid: u32, uid: u32, gid: u32, 
                               family: u32, socket_type: u32, socket_fd: i32) -> Result<NetworkEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::NetworkMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Track connection
        {
            let mut connections = self.connections.write();
            
            if connections.len() >= self.max_connections {
                self.evict_oldest(&mut connections);
            }
            
            let conn_id = (pid as u64) << 32 | (socket_fd as u32 as u64);
            connections.insert(conn_id, ConnectionInfo {
                pid,
                socket_fd,
                family,
                socket_type,
                remote_addr: None,
                remote_port: None,
                local_addr: None,
                local_port: None,
                first_seen: timestamp,
                last_seen: timestamp,
            });
        }
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Network socket create: pid={}, family={}, type={}", pid, family, socket_type);
        
        Ok(NetworkEvent {
            event_type: NetworkEventType::SocketCreate,
            pid,
            uid,
            gid,
            socket_family: family,
            socket_type,
            remote_addr: None,
            remote_port: None,
            local_addr: None,
            local_port: None,
            bytes_transferred: None,
            timestamp,
        })
    }
    
    /// Record socket connect event
    pub fn record_socket_connect(&self, pid: u32, uid: u32, gid: u32, socket_fd: i32,
                                remote_addr: String, remote_port: u16) -> Result<NetworkEvent, AgentError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::NetworkMonitoringFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        // Update connection info
        {
            let mut connections = self.connections.write();
            let conn_id = (pid as u64) << 32 | (socket_fd as u32 as u64);
            if let Some(conn) = connections.get_mut(&conn_id) {
                conn.remote_addr = Some(remote_addr.clone());
                conn.remote_port = Some(remote_port);
                conn.last_seen = timestamp;
            }
        }
        
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        debug!("Network socket connect: pid={}, {}:{}", pid, remote_addr, remote_port);
        
        Ok(NetworkEvent {
            event_type: NetworkEventType::SocketConnect,
            pid,
            uid,
            gid,
            socket_family: 0,
            socket_type: 0,
            remote_addr: Some(remote_addr),
            remote_port: Some(remote_port),
            local_addr: None,
            local_port: None,
            bytes_transferred: None,
            timestamp,
        })
    }
    
    /// Evict oldest connections (bounded memory)
    fn evict_oldest(&self, connections: &mut std::collections::HashMap<u64, ConnectionInfo>) {
        let target_size = (self.max_connections as f64 * 0.8) as usize;
        
        if connections.len() <= target_size {
            return;
        }
        
        let mut conn_vec: Vec<(u64, u64)> = connections.iter()
            .map(|(id, info)| (*id, info.last_seen))
            .collect();
        
        conn_vec.sort_by_key(|(_, ts)| *ts);
        
        let to_evict = connections.len() - target_size;
        for (id, _) in conn_vec.iter().take(to_evict) {
            connections.remove(id);
        }
    }
    
    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }
    
    /// Get events processed
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Relaxed)
    }
}

