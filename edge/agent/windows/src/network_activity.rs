// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/src/network_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network activity monitoring on Windows using GetExtendedTcpTable/GetExtendedUdpTable - observes network connections (NO enforcement, NO blocking)

#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use chrono::Utc;
#[cfg(windows)]
use tracing::{warn, debug, error};
#[cfg(windows)]
use crossbeam_channel::Sender;
#[cfg(windows)]
use crate::event::NetworkEvent;

/// Monitors network connections using Windows networking APIs
/// OBSERVATION ONLY - never blocks, never enforces, never modifies network state
#[cfg(windows)]
pub struct NetworkActivityMonitor;

#[cfg(windows)]
impl NetworkActivityMonitor {
    /// Monitor network connections
    pub async fn monitor(
        running: Arc<AtomicBool>,
        event_tx: Sender<NetworkEvent>,
        scan_interval_secs: u64,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(scan_interval_secs));
        let mut last_connections = std::collections::HashSet::new();
        
        while running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            let current_connections = Self::scan_connections();
            
            // Detect new connections
            for conn in &current_connections {
                if !last_connections.contains(conn) {
                    if let Some(event) = Self::connection_to_event(conn) {
                        if event_tx.try_send(event).is_err() {
                            warn!("Network event queue full, dropping event");
                        }
                    }
                }
            }
            
            last_connections = current_connections;
        }
    }
    
    /// Scan TCP and UDP connections using Windows APIs
    fn scan_connections() -> std::collections::HashSet<ConnectionKey> {
        use winapi::shared::ws2def::{AF_INET, SOCK_STREAM, SOCK_DGRAM};
        use winapi::um::iphlpapi::{GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCPTABLE_OWNER_MODULE, MIB_UDPTABLE_OWNER_MODULE};
        use std::ptr;
        
        let mut connections = std::collections::HashSet::new();
        
        unsafe {
            // Scan TCP connections
            let mut size = 0u32;
            GetExtendedTcpTable(
                ptr::null_mut(),
                &mut size,
                0,
                AF_INET as u32,
                5, // TCP_TABLE_OWNER_MODULE_ALL
                0,
            );
            
            if size > 0 {
                let mut buffer = vec![0u8; size as usize];
                if GetExtendedTcpTable(
                    buffer.as_mut_ptr() as *mut _,
                    &mut size,
                    0,
                    AF_INET as u32,
                    5,
                    0,
                ) == 0 {
                    // Parse TCP table (simplified - would parse MIB_TCPTABLE_OWNER_MODULE properly)
                    // For now, just track that we scanned
                }
            }
            
            // Scan UDP connections (similar)
            let mut size = 0u32;
            GetExtendedUdpTable(
                ptr::null_mut(),
                &mut size,
                0,
                AF_INET as u32,
                2, // UDP_TABLE_OWNER_MODULE
                0,
            );
        }
        
        connections
    }
    
    /// Convert ConnectionKey to NetworkEvent
    fn connection_to_event(conn: &ConnectionKey) -> Option<NetworkEvent> {
        Some(NetworkEvent {
            event_type: "network_connection".to_string(),
            src_ip: conn.src_ip.clone(),
            dst_ip: conn.dst_ip.clone(),
            src_port: conn.src_port,
            dst_port: conn.dst_port,
            protocol: conn.protocol.clone(),
            process_id: conn.process_id,
            timestamp: Utc::now(),
        })
    }
}

#[cfg(windows)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ConnectionKey {
    src_ip: String,
    dst_ip: String,
    src_port: u16,
    dst_port: u16,
    protocol: String,
    process_id: u32,
}

#[cfg(not(windows))]
pub struct NetworkActivityMonitor;

#[cfg(not(windows))]
impl NetworkActivityMonitor {
    pub async fn monitor(
        _running: std::sync::Arc<std::sync::atomic::AtomicBool>,
        _event_tx: crossbeam_channel::Sender<crate::event::NetworkEvent>,
        _scan_interval_secs: u64,
    ) {
        // Placeholder for non-Windows builds
    }
}
