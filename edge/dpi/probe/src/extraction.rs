// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/extraction.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Bounded feature extraction from parsed packets

use tracing::debug;

use super::errors::ProbeError;
use super::parser::ParsedPacket;
use super::flow::Flow;

/// Extracted features (bounded)
#[derive(Debug, Clone)]
pub struct Features {
    pub packet_size: u16,
    pub protocol: u8,
    pub is_fragment: bool,
    pub tcp_flags: Option<u8>,
    pub flow_duration: Option<u64>,
    pub flow_packet_count: Option<u64>,
    pub flow_byte_count: Option<u64>,
}

pub struct FeatureExtractor {
    max_features: usize,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            max_features: 100, // Bounded feature count
        }
    }
    
    /// Extract features from packet and flow (bounded)
    pub fn extract(&self, packet: &ParsedPacket, flow: Option<&Flow>) -> Result<Features, ProbeError> {
        // Validate bounds
        if packet.payload_len > 65535 {
            return Err(ProbeError::FeatureExtractionFailed(
                "Packet size exceeds maximum".to_string()
            ));
        }
        
        let protocol = match packet.protocol {
            super::parser::Protocol::TCP => 6,
            super::parser::Protocol::UDP => 17,
            super::parser::Protocol::ICMP => 1,
            super::parser::Protocol::IPv4 => 4,
            super::parser::Protocol::IPv6 => 41,
            _ => 0,
        };
        
        // Extract flow features if available
        let (flow_duration, flow_packet_count, flow_byte_count) = if let Some(f) = flow {
            let duration = packet.timestamp.saturating_sub(f.first_seen);
            (Some(duration), Some(f.packet_count), Some(f.byte_count))
        } else {
            (None, None, None)
        };
        
        debug!("Extracted features: size={}, protocol={}, fragment={}", 
            packet.payload_len, protocol, packet.is_fragment);
        
        Ok(Features {
            packet_size: packet.payload_len as u16,
            protocol,
            is_fragment: packet.is_fragment,
            tcp_flags: None, // Would be extracted from TCP header if available
            flow_duration,
            flow_packet_count,
            flow_byte_count,
        })
    }
    
    /// Get maximum feature count
    pub fn max_features(&self) -> usize {
        self.max_features
    }
}

