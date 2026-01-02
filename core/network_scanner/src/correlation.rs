// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/correlation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Integration with Phase 5 correlation engine - asset risk changes, newly exposed services, unexpected exposure

use std::sync::Arc;
use tracing::{error, warn, info, debug};

use crate::errors::ScannerError;
use crate::result::ScanResult;
use crate::persistence::ScanPersistence;

pub struct CorrelationIntegration {
    persistence: Arc<ScanPersistence>,
}

impl CorrelationIntegration {
    pub fn new(persistence: Arc<ScanPersistence>) -> Self {
        Self { persistence }
    }
    
    /// Submit scan result to correlation engine
    /// This exposes results to Phase 5 for correlation
    pub async fn submit_result(&self, result: &ScanResult) -> Result<(), ScannerError> {
        info!("Submitting scan result {} to correlation engine", result.scan_id);
        
        // Compute risk changes
        let risk_changes = self.compute_risk_changes(result).await?;
        
        // Detect newly exposed services
        let new_exposures = self.detect_new_exposures(result).await?;
        
        // Detect unexpected exposure
        let unexpected = self.detect_unexpected_exposure(result).await?;
        
        // Format correlation event (for Phase 5)
        let correlation_event = serde_json::json!({
            "event_type": "network_scan_result",
            "scan_id": result.scan_id,
            "asset_ip": result.asset.ip,
            "timestamp": result.timestamp,
            "risk_changes": risk_changes,
            "new_exposures": new_exposures,
            "unexpected_exposure": unexpected,
            "open_ports": result.open_ports.len(),
            "services": result.services.len(),
        });
        
        // In production, this would send to Phase 5 correlation engine
        // For now, log the event
        debug!("Correlation event: {}", serde_json::to_string_pretty(&correlation_event)?);
        
        Ok(())
    }
    
    /// Compute risk changes for asset
    async fn compute_risk_changes(&self, result: &ScanResult) -> Result<serde_json::Value, ScannerError> {
        // Get previous scan result
        let previous_results = self.persistence.get_asset_results(&result.asset.ip).await?;
        
        if previous_results.is_empty() {
            // First scan - compute initial risk
            let risk_score = self.compute_risk_score(result);
            return Ok(serde_json::json!({
                "type": "initial_risk",
                "risk_score": risk_score,
                "risk_level": self.risk_level(risk_score),
            }));
        }
        
        let previous = &previous_results[0];
        let previous_risk = self.compute_risk_score(previous);
        let current_risk = self.compute_risk_score(result);
        
        let risk_delta = current_risk - previous_risk;
        
        Ok(serde_json::json!({
            "type": "risk_change",
            "previous_risk": previous_risk,
            "current_risk": current_risk,
            "risk_delta": risk_delta,
            "risk_level": self.risk_level(current_risk),
        }))
    }
    
    /// Detect newly exposed services
    async fn detect_new_exposures(&self, result: &ScanResult) -> Result<serde_json::Value, ScannerError> {
        // Get previous scan result
        let previous_results = self.persistence.get_asset_results(&result.asset.ip).await?;
        
        if previous_results.is_empty() {
            // All services are new
            return Ok(serde_json::json!({
                "new_services": result.services.iter().map(|s| serde_json::json!({
                    "port": s.port,
                    "service": s.service_name,
                })).collect::<Vec<_>>(),
            }));
        }
        
        let previous = &previous_results[0];
        let previous_ports: std::collections::HashSet<u16> = previous.services.iter()
            .map(|s| s.port)
            .collect();
        
        let new_services: Vec<_> = result.services.iter()
            .filter(|s| !previous_ports.contains(&s.port))
            .map(|s| serde_json::json!({
                "port": s.port,
                "service": s.service_name,
            }))
            .collect();
        
        Ok(serde_json::json!({
            "new_services": new_services,
        }))
    }
    
    /// Detect unexpected exposure (e.g., RDP exposed externally)
    async fn detect_unexpected_exposure(&self, result: &ScanResult) -> Result<serde_json::Value, ScannerError> {
        let mut unexpected = Vec::new();
        
        for service in &result.services {
            // Check for high-risk services exposed
            if self.is_high_risk_service(service) {
                unexpected.push(serde_json::json!({
                    "port": service.port,
                    "service": service.service_name,
                    "risk": "high",
                    "reason": "High-risk service exposed",
                }));
            }
        }
        
        Ok(serde_json::json!({
            "unexpected_exposures": unexpected,
        }))
    }
    
    /// Compute risk score for scan result
    fn compute_risk_score(&self, result: &ScanResult) -> f64 {
        let mut score = 0.0;
        
        // Base score from number of open ports
        score += (result.open_ports.len() as f64) * 0.1;
        
        // Add risk from high-risk services
        for service in &result.services {
            if self.is_high_risk_service(service) {
                score += 0.5;
            }
        }
        
        // Normalize to 0-1 range
        score.min(1.0)
    }
    
    /// Get risk level from score
    fn risk_level(&self, score: f64) -> &str {
        if score >= 0.7 {
            "high"
        } else if score >= 0.4 {
            "medium"
        } else {
            "low"
        }
    }
    
    /// Check if service is high-risk
    fn is_high_risk_service(&self, service: &crate::result::Service) -> bool {
        // High-risk services: RDP, Telnet, VNC, etc.
        matches!(service.port, 3389 | 23 | 5900 | 1433 | 3306 | 5432)
            || service.service_name.to_lowercase().contains("rdp")
            || service.service_name.to_lowercase().contains("telnet")
            || service.service_name.to_lowercase().contains("vnc")
    }
}

// Note: This integration does NOT trigger policy actions
// It only exposes data to Phase 5 correlation engine
// Policy decisions are made by Phase 3, not Phase 9

