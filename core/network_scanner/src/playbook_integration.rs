// Path and File Name : /home/ransomeye/rebuild/core/network_scanner/src/playbook_integration.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Explicit playbook triggering - declarative triggers, explicit mapping, no auto-execution

use std::sync::Arc;
use tracing::{error, warn, info, debug};

use crate::errors::ScannerError;
use crate::result::ScanResult;
use crate::persistence::ScanPersistence;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PlaybookTrigger {
    pub trigger_id: String,
    pub condition: TriggerCondition,
    pub playbook_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum TriggerCondition {
    #[serde(rename = "new_external_rdp")]
    NewExternalRdp { port: u16 },
    
    #[serde(rename = "new_external_service")]
    NewExternalService { service_name: String },
    
    #[serde(rename = "risk_threshold")]
    RiskThreshold { threshold: f64 },
    
    #[serde(rename = "unexpected_exposure")]
    UnexpectedExposure { service_name: String },
}

pub struct PlaybookIntegration {
    persistence: Arc<ScanPersistence>,
    triggers: Vec<PlaybookTrigger>,
}

impl PlaybookIntegration {
    pub fn new(persistence: Arc<ScanPersistence>) -> Result<Self, ScannerError> {
        // Load triggers from configuration
        let triggers = Self::load_triggers()?;
        
        Ok(Self {
            persistence,
            triggers,
        })
    }
    
    /// Load playbook triggers from configuration
    fn load_triggers() -> Result<Vec<PlaybookTrigger>, ScannerError> {
        use std::fs;
        
        let trigger_file = std::env::var("RANSOMEYE_SCANNER_PLAYBOOK_TRIGGERS")
            .unwrap_or_else(|_| "/etc/ransomeye/config/scanner_playbook_triggers.yaml".to_string());
        
        if !std::path::Path::new(&trigger_file).exists() {
            warn!("Playbook trigger file does not exist: {}, using empty triggers", trigger_file);
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&trigger_file)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to read trigger file {}: {}", trigger_file, e)
            ))?;
        
        let triggers: Vec<PlaybookTrigger> = serde_yaml::from_str(&content)
            .map_err(|e| ScannerError::InvalidConfiguration(
                format!("Failed to parse trigger file: {}", e)
            ))?;
        
        Ok(triggers)
    }
    
    /// Evaluate scan result against playbook triggers
    /// Returns list of playbook IDs that should be triggered
    /// This is EXPLICIT - no auto-execution, just returns playbook IDs
    pub async fn evaluate_triggers(&self, result: &ScanResult) -> Result<Vec<String>, ScannerError> {
        let mut triggered_playbooks = Vec::new();
        
        for trigger in &self.triggers {
            if !trigger.enabled {
                continue;
            }
            
            if self.condition_matches(&trigger.condition, result).await? {
                info!("Playbook trigger matched: {} -> playbook {}", trigger.trigger_id, trigger.playbook_id);
                triggered_playbooks.push(trigger.playbook_id.clone());
            }
        }
        
        Ok(triggered_playbooks)
    }
    
    /// Check if trigger condition matches scan result
    async fn condition_matches(
        &self,
        condition: &TriggerCondition,
        result: &ScanResult,
    ) -> Result<bool, ScannerError> {
        match condition {
            TriggerCondition::NewExternalRdp { port } => {
                // Check if RDP port is newly exposed
                let deltas = self.persistence.get_asset_deltas(&result.asset.ip).await?;
                for delta in deltas {
                    if delta.delta_type == "new_ports" {
                        if let Some(ports) = delta.delta_data.get("ports").and_then(|v| v.as_array()) {
                            if ports.iter().any(|p| p.as_u64() == Some(*port as u64)) {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
            
            TriggerCondition::NewExternalService { service_name } => {
                // Check if service is newly exposed
                let deltas = self.persistence.get_asset_deltas(&result.asset.ip).await?;
                for delta in deltas {
                    if delta.delta_type == "new_ports" {
                        // Check if any new port matches the service
                        for service in &result.services {
                            if service.service_name == *service_name {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
            
            TriggerCondition::RiskThreshold { threshold } => {
                // Compute risk score and check threshold
                let risk_score = self.compute_risk_score(result);
                Ok(risk_score >= *threshold)
            }
            
            TriggerCondition::UnexpectedExposure { service_name } => {
                // Check if service is unexpectedly exposed
                for service in &result.services {
                    if service.service_name == *service_name {
                        // Check if it's a high-risk service
                        if self.is_high_risk_service(service) {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
        }
    }
    
    /// Compute risk score (simplified)
    fn compute_risk_score(&self, result: &ScanResult) -> f64 {
        let mut score = 0.0;
        
        for service in &result.services {
            if self.is_high_risk_service(service) {
                score += 0.3;
            }
        }
        
        score.min(1.0)
    }
    
    /// Check if service is high-risk
    fn is_high_risk_service(&self, service: &crate::result::Service) -> bool {
        matches!(service.port, 3389 | 23 | 5900 | 1433 | 3306 | 5432)
            || service.service_name.to_lowercase().contains("rdp")
            || service.service_name.to_lowercase().contains("telnet")
            || service.service_name.to_lowercase().contains("vnc")
    }
}

// CRITICAL: This module does NOT execute playbooks
// It only returns playbook IDs that should be triggered
// Actual execution is handled by Phase 6 (Playbook Engine)
// This ensures explicit, declarative triggering with no auto-execution

