// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/flow.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Flow assembler - tracks network flows and assembles packets into flows

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use dashmap::DashMap;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlowKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub flow_id: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub packet_count: u64,
    pub byte_count: u64,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub metadata: serde_json::Value,
}

pub struct FlowAssembler {
    flows: DashMap<FlowKey, Flow>,
    timeout: Duration,
}

impl FlowAssembler {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            flows: DashMap::new(),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }
    
    pub fn process_packet(
        &self,
        src_ip: IpAddr,
        dst_ip: IpAddr,
        src_port: u16,
        dst_port: u16,
        protocol: u8,
        packet_size: usize,
    ) -> Option<Flow> {
        let key = FlowKey {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
        };
        
        let now = SystemTime::now();
        
        // Get or create flow
        let mut flow = self.flows.entry(key.clone())
            .or_insert_with(|| {
                let flow_id = uuid::Uuid::new_v4().to_string();
                Flow {
                    flow_id,
                    src_ip: src_ip.to_string(),
                    dst_ip: dst_ip.to_string(),
                    src_port,
                    dst_port,
                    protocol: Self::protocol_name(protocol),
                    packet_count: 0,
                    byte_count: 0,
                    first_seen: now,
                    last_seen: now,
                    metadata: serde_json::json!({}),
                }
            })
            .clone();
        
        // Update flow statistics
        flow.packet_count += 1;
        flow.byte_count += packet_size as u64;
        flow.last_seen = now;
        
        // Check if flow should be exported (timeout or size threshold)
        let age = now.duration_since(flow.first_seen).unwrap_or(Duration::ZERO);
        if age >= self.timeout || flow.packet_count >= 10000 {
            self.flows.remove(&key);
            return Some(flow);
        }
        
        // Update stored flow
        if let Some(mut stored) = self.flows.get_mut(&key) {
            stored.packet_count = flow.packet_count;
            stored.byte_count = flow.byte_count;
            stored.last_seen = flow.last_seen;
        }
        
        None
    }
    
    pub fn cleanup_stale_flows(&self) {
        let now = SystemTime::now();
        self.flows.retain(|_, flow| {
            let age = now.duration_since(flow.last_seen).unwrap_or(Duration::ZERO);
            age < self.timeout
        });
    }
    
    fn protocol_name(protocol: u8) -> String {
        match protocol {
            1 => "ICMP".to_string(),
            6 => "TCP".to_string(),
            17 => "UDP".to_string(),
            _ => format!("PROTO_{}", protocol),
        }
    }
}


