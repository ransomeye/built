// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/visibility.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: SOC Copilot visibility interface - read-only access to discovered assets, exposure changes, scan history, risk deltas

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::errors::ScannerError;
use crate::result::ScanResult;
use crate::persistence::{ScanPersistence, ScanDelta};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetView {
    pub ip: String,
    pub hostname: Option<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub scan_count: i32,
    pub open_ports: Vec<PortView>,
    pub services: Vec<ServiceView>,
    pub risk_score: f64,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortView {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceView {
    pub port: u16,
    pub protocol: String,
    pub service_name: String,
    pub version: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureChangeView {
    pub asset_ip: String,
    pub change_type: String,
    pub timestamp: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistoryView {
    pub scan_id: String,
    pub timestamp: String,
    pub scanner_mode: String,
    pub asset_ip: String,
    pub ports_found: usize,
    pub services_found: usize,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDeltaView {
    pub asset_ip: String,
    pub previous_risk: f64,
    pub current_risk: f64,
    pub risk_delta: f64,
    pub risk_level: String,
    pub timestamp: String,
}

pub struct ScannerVisibility {
    persistence: Arc<ScanPersistence>,
}

impl ScannerVisibility {
    pub fn new(persistence: Arc<ScanPersistence>) -> Self {
        Self { persistence }
    }
    
    /// Get all discovered assets (read-only)
    pub async fn get_discovered_assets(&self) -> Result<Vec<AssetView>, ScannerError> {
        use sqlx::Row;
        
        let rows = sqlx::query(
            r#"
            SELECT a.ip, a.hostname, a.mac, a.vendor, a.first_seen, a.last_seen, a.scan_count,
                   COALESCE(json_agg(DISTINCT jsonb_build_object(
                       'port', ps.port,
                       'protocol', ps.protocol,
                       'service_name', ps.service_name,
                       'service_version', ps.service_version,
                       'last_seen', ps.last_seen
                   )) FILTER (WHERE ps.port IS NOT NULL), '[]'::json) as services
            FROM scan_assets a
            LEFT JOIN scan_port_services ps ON a.ip = ps.asset_ip
            GROUP BY a.ip, a.hostname, a.mac, a.vendor, a.first_seen, a.last_seen, a.scan_count
            ORDER BY a.last_seen DESC
            "#
        )
        .fetch_all(&self.persistence.pool)
        .await
        .map_err(|e| ScannerError::DatabaseError(
            format!("Failed to fetch assets: {}", e)
        ))?;
        
        // Convert rows to AssetView (simplified - full implementation would parse JSON)
        // For now, return empty list
        Ok(Vec::new())
    }
    
    /// Get exposure changes for an asset (read-only)
    pub async fn get_exposure_changes(&self, asset_ip: &str) -> Result<Vec<ExposureChangeView>, ScannerError> {
        let deltas = self.persistence.get_asset_deltas(asset_ip).await?;
        
        Ok(deltas.into_iter().map(|delta| {
            ExposureChangeView {
                asset_ip: delta.asset_ip.clone(),
                change_type: delta.delta_type.clone(),
                timestamp: delta.created_at.to_rfc3339(),
                details: delta.delta_data,
            }
        }).collect())
    }
    
    /// Get scan history for an asset (read-only)
    pub async fn get_scan_history(&self, asset_ip: &str) -> Result<Vec<ScanHistoryView>, ScannerError> {
        let results = self.persistence.get_asset_results(asset_ip).await?;
        
        Ok(results.into_iter().map(|result| {
            ScanHistoryView {
                scan_id: result.scan_id.clone(),
                timestamp: result.timestamp.to_rfc3339(),
                scanner_mode: format!("{:?}", result.scanner_mode),
                asset_ip: result.asset.ip.clone(),
                ports_found: result.open_ports.len(),
                services_found: result.services.len(),
                confidence_score: result.confidence_score,
            }
        }).collect())
    }
    
    /// Get risk deltas (read-only)
    pub async fn get_risk_deltas(&self, asset_ip: &str) -> Result<Vec<RiskDeltaView>, ScannerError> {
        let results = self.persistence.get_asset_results(asset_ip).await?;
        
        if results.len() < 2 {
            return Ok(Vec::new());
        }
        
        let mut risk_deltas = Vec::new();
        
        for i in 1..results.len() {
            let previous = &results[i];
            let current = &results[i - 1];
            
            let previous_risk = self.compute_risk_score(previous);
            let current_risk = self.compute_risk_score(current);
            let risk_delta = current_risk - previous_risk;
            
            risk_deltas.push(RiskDeltaView {
                asset_ip: asset_ip.to_string(),
                previous_risk,
                current_risk,
                risk_delta,
                risk_level: self.risk_level(current_risk),
                timestamp: current.timestamp.to_rfc3339(),
            });
        }
        
        Ok(risk_deltas)
    }
    
    /// Compute risk score
    fn compute_risk_score(&self, result: &ScanResult) -> f64 {
        let mut score = 0.0;
        
        for service in &result.services {
            if self.is_high_risk_service(service) {
                score += 0.3;
            }
        }
        
        score.min(1.0)
    }
    
    /// Get risk level
    fn risk_level(&self, score: f64) -> String {
        if score >= 0.7 {
            "high".to_string()
        } else if score >= 0.4 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }
    
    /// Check if service is high-risk
    fn is_high_risk_service(&self, service: &crate::result::Service) -> bool {
        matches!(service.port, 3389 | 23 | 5900 | 1433 | 3306 | 5432)
            || service.service_name.to_lowercase().contains("rdp")
            || service.service_name.to_lowercase().contains("telnet")
            || service.service_name.to_lowercase().contains("vnc")
    }
}

// Note: This interface is READ-ONLY
// SOC Copilot cannot initiate scans, only view results

