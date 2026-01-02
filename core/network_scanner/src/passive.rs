// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/passive.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Passive scanner - flow metadata ingestion only, no packet capture, no payload inspection

use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use tracing::{error, warn, info, debug};

use crate::errors::ScannerError;
use crate::result::{ScanResult, ScannerMode, Asset, PortInfo, PortState, Service, ScanMetadata};
use crate::security::ScanResultSigner;

/// Flow metadata structure (from Phase 4 ingestion)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FlowMetadata {
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

pub struct PassiveScanner {
    signer: Arc<ScanResultSigner>,
}

impl PassiveScanner {
    /// Create a new passive scanner
    pub fn new() -> Result<Self, ScannerError> {
        // Get signing key path
        let signing_key_path = std::env::var("RANSOMEYE_SCANNER_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "/etc/ransomeye/keys/scanner_private_key.pem".to_string());
        
        let signer = Arc::new(ScanResultSigner::new(&signing_key_path)?);
        
        Ok(Self { signer })
    }
    
    /// Process flow metadata (from Phase 4 ingestion)
    /// This function does NOT perform packet capture or payload inspection
    pub async fn process_flow_metadata(
        &self,
        flows: Vec<FlowMetadata>,
    ) -> Result<Vec<ScanResult>, ScannerError> {
        info!("Processing {} flow metadata records (passive scan)", flows.len());
        
        // Group flows by destination IP (asset discovery)
        let mut assets: std::collections::HashMap<String, Vec<FlowMetadata>> = 
            std::collections::HashMap::new();
        
        for flow in flows {
            // Only process destination IPs (assets being accessed)
            assets.entry(flow.dst_ip.clone())
                .or_insert_with(Vec::new)
                .push(flow);
        }
        
        let mut results = Vec::new();
        
        // Generate scan result for each discovered asset
        for (ip, asset_flows) in assets {
            match self.build_result_from_flows(&ip, asset_flows).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    warn!("Failed to build result for asset {}: {}", ip, e);
                    // Continue with next asset
                }
            }
        }
        
        info!("Passive scan produced {} asset results", results.len());
        Ok(results)
    }
    
    /// Build scan result from flow metadata
    async fn build_result_from_flows(
        &self,
        ip: &str,
        flows: Vec<FlowMetadata>,
    ) -> Result<ScanResult, ScannerError> {
        debug!("Building result for asset {} from {} flows", ip, flows.len());
        
        // Extract unique ports from flows
        let mut ports: std::collections::HashSet<u16> = std::collections::HashSet::new();
        let mut services = Vec::new();
        
        for flow in &flows {
            ports.insert(flow.dst_port);
            
            // Infer service from port and protocol
            let service_name = self.infer_service_from_port(flow.dst_port, &flow.protocol);
            
            services.push(Service {
                port: flow.dst_port,
                protocol: flow.protocol.clone(),
                service_name,
                version: None, // No version info from flow metadata
                banner: None, // No banner in passive scan
                confidence: 0.6, // Lower confidence for passive detection
            });
        }
        
        // Build scan result
        let mut result = ScanResult {
            scan_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            scanner_mode: ScannerMode::Passive,
            asset: Asset {
                ip: ip.to_string(),
                hostname: None, // Hostname not available from flow metadata
                mac: None, // MAC not available from flow metadata
                vendor: None,
            },
            open_ports: ports.iter().map(|p| PortInfo {
                port: *p,
                protocol: "tcp".to_string(), // Default to TCP
                state: PortState::Open, // Assumed open if traffic observed
                discovered_at: Utc::now(),
            }).collect(),
            services,
            confidence_score: 0.6, // Lower confidence for passive scan
            hash: String::new(),
            signature: String::new(),
            metadata: Some(ScanMetadata {
                scan_duration_ms: 0,
                ports_scanned: ports.len(),
                hosts_scanned: 1,
                rate_limit_applied: false, // No rate limiting in passive mode
                cidr: None,
            }),
        };
        
        // Sign result
        result = self.signer.sign_result(result)?;
        
        Ok(result)
    }
    
    /// Infer service from port and protocol
    fn infer_service_from_port(&self, port: u16, protocol: &str) -> String {
        match (port, protocol.to_lowercase().as_str()) {
            (22, _) => "ssh".to_string(),
            (23, _) => "telnet".to_string(),
            (25, _) => "smtp".to_string(),
            (53, _) => "dns".to_string(),
            (80, _) | (8080, _) => "http".to_string(),
            (443, _) | (8443, _) => "https".to_string(),
            (110, _) => "pop3".to_string(),
            (143, _) => "imap".to_string(),
            (445, _) => "smb".to_string(),
            (993, _) => "imaps".to_string(),
            (995, _) => "pop3s".to_string(),
            (1433, _) => "mssql".to_string(),
            (3306, _) => "mysql".to_string(),
            (3389, _) => "rdp".to_string(),
            (5432, _) => "postgresql".to_string(),
            (5900, _) => "vnc".to_string(),
            _ => format!("unknown-{}", port),
        }
    }
}

// CRITICAL: Passive scanner MUST NEVER:
// - Perform packet capture
// - Inspect payloads
// - Access raw network traffic
// - Use libpcap or similar
// - Store packet contents
//
// It ONLY processes flow metadata from Phase 4 ingestion

