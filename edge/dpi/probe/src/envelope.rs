// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/envelope.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Phase-4 compliant event envelope creation

use serde::{Serialize, Deserialize};
use chrono::Utc;
use tracing::debug;

use super::errors::ProbeError;
use super::parser::ParsedPacket;
use super::extraction::Features;

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
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: String,
    pub packet_size: u16,
    pub is_fragment: bool,
    pub features: FeaturesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesData {
    pub flow_duration: Option<u64>,
    pub flow_packet_count: Option<u64>,
    pub flow_byte_count: Option<u64>,
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
    
    /// Create Phase-4 event envelope
    /// 
    /// No enrichment, no inference, no policy logic.
    /// Raw telemetry only.
    pub fn build(&mut self, packet: &ParsedPacket, features: &Features, signature: String) -> Result<EventEnvelope, ProbeError> {
        self.sequence += 1;
        
        let event_id = format!("dpi-{}-{}", self.component_id, self.sequence);
        let timestamp = Utc::now().to_rfc3339();
        
        let protocol_str = match packet.protocol {
            super::parser::Protocol::TCP => "TCP",
            super::parser::Protocol::UDP => "UDP",
            super::parser::Protocol::ICMP => "ICMP",
            super::parser::Protocol::IPv4 => "IPv4",
            super::parser::Protocol::IPv6 => "IPv6",
            super::parser::Protocol::Ethernet => "Ethernet",
            super::parser::Protocol::Unknown => "Unknown",
        };
        
        let envelope = EventEnvelope {
            event_id,
            timestamp,
            component: self.component.clone(),
            component_id: self.component_id.clone(),
            event_type: "network_telemetry".to_string(),
            sequence: self.sequence,
            signature,
            data: EventData {
                src_ip: packet.src_ip.clone(),
                dst_ip: packet.dst_ip.clone(),
                src_port: packet.src_port,
                dst_port: packet.dst_port,
                protocol: protocol_str.to_string(),
                packet_size: features.packet_size,
                is_fragment: features.is_fragment,
                features: FeaturesData {
                    flow_duration: features.flow_duration,
                    flow_packet_count: features.flow_packet_count,
                    flow_byte_count: features.flow_byte_count,
                },
            },
        };
        
        debug!("Created event envelope: {}", envelope.event_id);
        Ok(envelope)
    }
    
    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

