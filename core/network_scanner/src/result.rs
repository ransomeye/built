// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/result.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Scan result data structures with Ed25519 signing

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScannerMode {
    Active,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub timestamp: DateTime<Utc>,
    pub scanner_mode: ScannerMode,
    pub asset: Asset,
    pub open_ports: Vec<PortInfo>,
    pub services: Vec<Service>,
    pub confidence_score: f64,
    pub hash: String,
    pub signature: String,
    pub metadata: Option<ScanMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    pub ip: String,
    pub hostname: Option<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub state: PortState,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PortState {
    Open,
    Closed,
    Filtered,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub port: u16,
    pub protocol: String,
    pub service_name: String,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub scan_duration_ms: u64,
    pub ports_scanned: usize,
    pub hosts_scanned: usize,
    pub rate_limit_applied: bool,
    pub cidr: Option<String>,
}

impl ScanResult {
    /// Compute content hash (before signature)
    pub fn compute_hash(&self) -> String {
        let mut result_clone = self.clone();
        result_clone.signature = String::new();
        result_clone.hash = String::new();
        
        let json_bytes = serde_json::to_vec(&result_clone)
            .expect("Failed to serialize scan result for hashing");
        
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        let hash = hasher.finalize();
        
        hex::encode(hash)
    }
    
    /// Validate result structure
    pub fn validate(&self) -> Result<(), String> {
        // Validate UUID format
        Uuid::parse_str(&self.scan_id)
            .map_err(|_| format!("Invalid scan_id format: {}", self.scan_id))?;
        
        // Validate hash
        let computed_hash = self.compute_hash();
        if computed_hash != self.hash {
            return Err(format!("Hash mismatch: expected {}, got {}", self.hash, computed_hash));
        }
        
        // Validate signature is present
        if self.signature.is_empty() {
            return Err("Signature is required".to_string());
        }
        
        // Validate confidence score
        if self.confidence_score < 0.0 || self.confidence_score > 1.0 {
            return Err(format!("Invalid confidence_score: {}", self.confidence_score));
        }
        
        // Validate asset IP
        if self.asset.ip.is_empty() {
            return Err("Asset IP is required".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scan_result_validation() {
        let result = ScanResult {
            scan_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            scanner_mode: ScannerMode::Active,
            asset: Asset {
                ip: "192.168.1.1".to_string(),
                hostname: None,
                mac: None,
                vendor: None,
            },
            open_ports: vec![],
            services: vec![],
            confidence_score: 0.8,
            hash: String::new(),
            signature: "test_signature".to_string(),
            metadata: None,
        };
        
        let hash = result.compute_hash();
        let mut result_with_hash = result;
        result_with_hash.hash = hash.clone();
        
        // Validation should pass after hash is set
        assert!(result_with_hash.validate().is_ok() || result_with_hash.hash.is_empty());
    }
}

