// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/flow.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Flow tracking with bounded memory and eviction

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{warn, debug};

use super::errors::ProbeError;
use super::parser::ParsedPacket;

/// Flow identifier (5-tuple)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlowKey {
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
}

impl FlowKey {
    pub fn from_packet(packet: &ParsedPacket) -> Option<Self> {
        let src_ip = packet.src_ip.as_ref()?;
        let dst_ip = packet.dst_ip.as_ref()?;
        let src_port = packet.src_port?;
        let dst_port = packet.dst_port?;
        
        let protocol = match packet.protocol {
            super::parser::Protocol::TCP => 6,
            super::parser::Protocol::UDP => 17,
            super::parser::Protocol::ICMP => 1,
            _ => return None,
        };
        
        Some(Self {
            src_ip: src_ip.clone(),
            dst_ip: dst_ip.clone(),
            src_port,
            dst_port,
            protocol,
        })
    }
}

/// Flow state (bounded memory)
#[derive(Debug, Clone)]
pub struct Flow {
    pub key: FlowKey,
    pub first_seen: u64,
    pub last_seen: u64,
    pub packet_count: u64,
    pub byte_count: u64,
    pub flags: u8,
}

/// Flow tracker with bounded memory
/// 
/// Maximum flows: 1,000,000 (configurable)
/// Eviction: LRU (Least Recently Used)
/// Lock-free reads, bounded lock for writes
pub struct FlowTracker {
    flows: Arc<RwLock<HashMap<FlowKey, Flow>>>,
    max_flows: usize,
    eviction_threshold: usize,
}

impl FlowTracker {
    /// Create new flow tracker
    pub fn new(max_flows: usize) -> Self {
        let eviction_threshold = (max_flows as f64 * 0.9) as usize; // Evict at 90% capacity
        
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            max_flows,
            eviction_threshold,
        }
    }
    
    /// Update or create flow (bounded memory)
    pub fn update_flow(&self, packet: &ParsedPacket) -> Result<(), ProbeError> {
        let key = FlowKey::from_packet(packet)
            .ok_or_else(|| ProbeError::FlowTrackingFailed("Invalid flow key".to_string()))?;
        
        let mut flows = self.flows.write();
        
        // Check if we need to evict
        if flows.len() >= self.eviction_threshold {
            self.evict_lru(&mut flows);
        }
        
        // Update or create flow
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ProbeError::FlowTrackingFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        match flows.get_mut(&key) {
            Some(flow) => {
                // Update existing flow
                flow.last_seen = packet.timestamp;
                flow.packet_count += 1;
                flow.byte_count += packet.payload_len as u64;
            }
            None => {
                // Create new flow
                if flows.len() >= self.max_flows {
                    return Err(ProbeError::MemoryLimitExceeded(
                        format!("Flow table full: {} flows", self.max_flows)
                    ));
                }
                
                flows.insert(key.clone(), Flow {
                    key,
                    first_seen: packet.timestamp,
                    last_seen: packet.timestamp,
                    packet_count: 1,
                    byte_count: packet.payload_len as u64,
                    flags: 0,
                });
            }
        }
        
        Ok(())
    }
    
    /// Evict least recently used flows
    fn evict_lru(&self, flows: &mut HashMap<FlowKey, Flow>) {
        let target_size = (self.max_flows as f64 * 0.8) as usize; // Evict to 80% capacity
        
        if flows.len() <= target_size {
            return;
        }
        
        // Collect flows sorted by last_seen
        let mut flow_vec: Vec<(FlowKey, u64)> = flows.iter()
            .map(|(k, v)| (k.clone(), v.last_seen))
            .collect();
        
        flow_vec.sort_by_key(|(_, ts)| *ts);
        
        // Evict oldest flows
        let to_evict = flows.len() - target_size;
        for (key, _) in flow_vec.iter().take(to_evict) {
            flows.remove(key);
        }
        
        debug!("Evicted {} flows, current size: {}", to_evict, flows.len());
    }
    
    /// Get flow count
    pub fn flow_count(&self) -> usize {
        self.flows.read().len()
    }
    
    /// Get flow by key (read-only, lock-free compatible)
    pub fn get_flow(&self, key: &FlowKey) -> Option<Flow> {
        self.flows.read().get(key).cloned()
    }
    
    /// Clear all flows
    pub fn clear(&self) {
        self.flows.write().clear();
    }
}

