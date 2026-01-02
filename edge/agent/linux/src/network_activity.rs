// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/src/network_activity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network activity monitoring - observes network connections (NO enforcement, NO blocking)

use std::fs;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Utc;
use tracing::{warn, debug, error};
use crossbeam_channel::Sender;
use crate::event::NetworkEvent;

/// Monitors network connections by reading /proc/net files
/// OBSERVATION ONLY - never blocks, never enforces, never modifies network state
pub struct NetworkActivityMonitor;

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
    
    /// Scan /proc/net/tcp and /proc/net/udp for active connections
    fn scan_connections() -> std::collections::HashSet<ConnectionKey> {
        let mut connections = std::collections::HashSet::new();
        
        // Scan TCP connections
        if let Ok(file) = fs::File::open("/proc/net/tcp") {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1) {
                if let Ok(line) = line {
                    if let Some(conn) = Self::parse_net_line(&line, "TCP") {
                        connections.insert(conn);
                    }
                }
            }
        }
        
        // Scan UDP connections
        if let Ok(file) = fs::File::open("/proc/net/udp") {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1) {
                if let Ok(line) = line {
                    if let Some(conn) = Self::parse_net_line(&line, "UDP") {
                        connections.insert(conn);
                    }
                }
            }
        }
        
        connections
    }
    
    /// Parse a line from /proc/net/tcp or /proc/net/udp
    fn parse_net_line(line: &str, protocol: &str) -> Option<ConnectionKey> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 2 {
            return None;
        }
        
        // Parse local address (format: IPv4:PORT in hex)
        let local_addr = fields[1];
        let (src_ip, src_port) = Self::parse_address(local_addr)?;
        
        // Parse remote address
        let remote_addr = fields[2];
        let (dst_ip, dst_port) = Self::parse_address(remote_addr)?;
        
        // Extract process ID if available (from inode lookup - simplified)
        let process_id = 0; // Would need /proc/net lookup for actual PID
        
        Some(ConnectionKey {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol: protocol.to_string(),
            process_id,
        })
    }
    
    /// Parse address from /proc/net format (hex IPv4:PORT)
    fn parse_address(addr: &str) -> Option<(String, u16)> {
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        // Parse IP (4 hex bytes)
        let ip_hex = parts[0];
        if ip_hex.len() != 8 {
            return None;
        }
        
        let ip_bytes: Vec<u8> = (0..4)
            .filter_map(|i| {
                let start = i * 2;
                u8::from_str_radix(&ip_hex[start..start + 2], 16).ok()
            })
            .collect();
        
        if ip_bytes.len() != 4 {
            return None;
        }
        
        let ip = format!("{}.{}.{}.{}", ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]);
        
        // Parse port (hex)
        let port = u16::from_str_radix(parts[1], 16).ok()?;
        
        Some((ip, port))
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ConnectionKey {
    src_ip: String,
    dst_ip: String,
    src_port: u16,
    dst_port: u16,
    protocol: String,
    process_id: i32,
}
