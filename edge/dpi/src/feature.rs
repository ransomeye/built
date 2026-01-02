// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/src/feature.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Feature extractor - extracts features from flows for telemetry (NO AI, feature extraction only)

use serde::{Serialize, Deserialize};
use crate::flow::Flow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowFeatures {
    pub flow_id: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub packet_count: u64,
    pub byte_count: u64,
    pub duration_seconds: f64,
    pub bytes_per_second: f64,
    pub packets_per_second: f64,
    pub metadata: serde_json::Value,
}

pub struct FeatureExtractor;

impl FeatureExtractor {
    pub fn extract(flow: &Flow) -> FlowFeatures {
        let duration = flow.last_seen
            .duration_since(flow.first_seen)
            .unwrap_or(std::time::Duration::ZERO)
            .as_secs_f64();
        
        let bytes_per_second = if duration > 0.0 {
            flow.byte_count as f64 / duration
        } else {
            0.0
        };
        
        let packets_per_second = if duration > 0.0 {
            flow.packet_count as f64 / duration
        } else {
            0.0
        };
        
        FlowFeatures {
            flow_id: flow.flow_id.clone(),
            src_ip: flow.src_ip.clone(),
            dst_ip: flow.dst_ip.clone(),
            src_port: flow.src_port,
            dst_port: flow.dst_port,
            protocol: flow.protocol.clone(),
            packet_count: flow.packet_count,
            byte_count: flow.byte_count,
            duration_seconds: duration,
            bytes_per_second,
            packets_per_second,
            metadata: flow.metadata.clone(),
        }
    }
    
    pub fn to_telemetry_data(features: &FlowFeatures) -> serde_json::Value {
        serde_json::json!({
            "flow_id": features.flow_id,
            "src_ip": features.src_ip,
            "dst_ip": features.dst_ip,
            "src_port": features.src_port,
            "dst_port": features.dst_port,
            "protocol": features.protocol,
            "packet_count": features.packet_count,
            "byte_count": features.byte_count,
            "classification": "unknown", // No AI - classification done by Control Plane
            "metadata": features.metadata,
        })
    }
}

