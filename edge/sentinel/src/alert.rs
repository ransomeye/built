// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/alert.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Alert emission for Sentinel violations

use tracing::{error, warn};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AlertEmitter {
    core_api_url: Option<String>,
}

impl AlertEmitter {
    pub fn new(core_api_url: Option<String>) -> Self {
        Self {
            core_api_url,
        }
    }
    
    /// Emit alert for Agent unhealthy
    pub async fn emit_agent_unhealthy(&self) {
        warn!("ALERT: Agent health check failed");
        self.emit_alert("agent_unhealthy", "Agent health check failed").await;
    }
    
    /// Emit alert for Agent terminated
    pub async fn emit_agent_terminated(&self) {
        error!("ALERT: Agent terminated unexpectedly");
        self.emit_alert("agent_terminated", "Agent service terminated unexpectedly").await;
    }
    
    /// Emit alert for Agent error
    pub async fn emit_agent_error(&self, error: String) {
        error!("ALERT: Agent error: {}", error);
        self.emit_alert("agent_error", &error).await;
    }
    
    /// Emit alert for Agent tamper
    pub async fn emit_agent_tamper(&self, error: String) {
        error!("ALERT: Agent binary tamper detected: {}", error);
        self.emit_alert("agent_tamper", &error).await;
    }
    
    /// Emit alert for DPI unhealthy
    pub async fn emit_dpi_unhealthy(&self) {
        warn!("ALERT: DPI health check failed");
        self.emit_alert("dpi_unhealthy", "DPI health check failed").await;
    }
    
    /// Emit alert for DPI terminated
    pub async fn emit_dpi_terminated(&self) {
        error!("ALERT: DPI terminated unexpectedly");
        self.emit_alert("dpi_terminated", "DPI service terminated unexpectedly").await;
    }
    
    /// Emit alert for DPI error
    pub async fn emit_dpi_error(&self, error: String) {
        error!("ALERT: DPI error: {}", error);
        self.emit_alert("dpi_error", &error).await;
    }
    
    /// Emit alert for DPI tamper
    pub async fn emit_dpi_tamper(&self, error: String) {
        error!("ALERT: DPI binary tamper detected: {}", error);
        self.emit_alert("dpi_tamper", &error).await;
    }
    
    /// Emit generic alert
    async fn emit_alert(&self, alert_type: &str, message: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Log alert
        error!("SENTINEL ALERT [{}] {}: {}", timestamp, alert_type, message);
        
        // In production, would send to Core API
        if let Some(ref url) = self.core_api_url {
            // Would use HTTP client to send alert
            warn!("Alert would be sent to Core API: {}/alerts", url);
        }
    }
}

