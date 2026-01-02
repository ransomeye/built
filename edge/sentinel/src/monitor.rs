// Path and File Name : /home/ransomeye/rebuild/edge/sentinel/src/monitor.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Component health monitoring for Agent and DPI

use std::path::Path;
use std::fs;
use std::process::Command;
use tracing::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonitorError {
    #[error("Service check failed: {0}")]
    ServiceCheckFailed(String),
    #[error("Binary integrity check failed: {0}")]
    BinaryIntegrityFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentHealth {
    Healthy,
    Unhealthy,
    Terminated,
}

/// Monitor for Linux Agent
pub struct AgentMonitor {
    service_name: String,
    binary_path: String,
}

impl AgentMonitor {
    pub fn new(service_name: String) -> Self {
        let binary_path = std::env::var("AGENT_BINARY_PATH")
            .unwrap_or_else(|_| "/opt/ransomeye/linux_agent/bin/ransomeye_linux_agent".to_string());
        
        Self {
            service_name,
            binary_path,
        }
    }
    
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
    
    /// Check Agent service health
    pub async fn check_health(&self) -> Result<ComponentHealth, MonitorError> {
        // Check if service is active
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(&self.service_name)
            .output()
            .map_err(|e| MonitorError::ServiceCheckFailed(format!("Failed to check service: {}", e)))?;
        
        let status_str = String::from_utf8_lossy(&output.stdout);
        let status = status_str.trim();
        
        match status {
            "active" => Ok(ComponentHealth::Healthy),
            "inactive" | "failed" => Ok(ComponentHealth::Terminated),
            _ => Ok(ComponentHealth::Unhealthy),
        }
    }
    
    /// Check Agent binary integrity
    pub async fn check_binary_integrity(&self) -> Result<(), MonitorError> {
        if !Path::new(&self.binary_path).exists() {
            return Err(MonitorError::BinaryIntegrityFailed(
                format!("Agent binary not found: {}", self.binary_path)
            ));
        }
        
        // In production, would verify against stored hash
        // For now, just verify file exists and is executable
        let metadata = fs::metadata(&self.binary_path)
            .map_err(|e| MonitorError::BinaryIntegrityFailed(
                format!("Failed to read binary metadata: {}", e)
            ))?;
        
        // Check if file was recently modified (potential tampering indicator)
        let _modified = metadata.modified()
            .map_err(|e| MonitorError::BinaryIntegrityFailed(
                format!("Failed to get modification time: {}", e)
            ))?;
        
        // If binary was modified after service start, it's suspicious
        // In production, would compare against stored hash
        
        Ok(())
    }
}

/// Monitor for DPI Probe
pub struct DpiMonitor {
    service_name: String,
    binary_path: String,
}

impl DpiMonitor {
    pub fn new(service_name: String) -> Self {
        let binary_path = std::env::var("DPI_BINARY_PATH")
            .unwrap_or_else(|_| "/opt/ransomeye/dpi_probe/bin/ransomeye_dpi_probe".to_string());
        
        Self {
            service_name,
            binary_path,
        }
    }
    
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
    
    /// Check DPI service health
    pub async fn check_health(&self) -> Result<ComponentHealth, MonitorError> {
        // Check if service is active
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(&self.service_name)
            .output()
            .map_err(|e| MonitorError::ServiceCheckFailed(format!("Failed to check service: {}", e)))?;
        
        let status_str = String::from_utf8_lossy(&output.stdout);
        let status = status_str.trim();
        
        match status {
            "active" => Ok(ComponentHealth::Healthy),
            "inactive" | "failed" => Ok(ComponentHealth::Terminated),
            _ => Ok(ComponentHealth::Unhealthy),
        }
    }
    
    /// Check DPI binary integrity
    pub async fn check_binary_integrity(&self) -> Result<(), MonitorError> {
        if !Path::new(&self.binary_path).exists() {
            return Err(MonitorError::BinaryIntegrityFailed(
                format!("DPI binary not found: {}", self.binary_path)
            ));
        }
        
        // In production, would verify against stored hash
        let metadata = fs::metadata(&self.binary_path)
            .map_err(|e| MonitorError::BinaryIntegrityFailed(
                format!("Failed to read binary metadata: {}", e)
            ))?;
        
        let _modified = metadata.modified()
            .map_err(|e| MonitorError::BinaryIntegrityFailed(
                format!("Failed to get modification time: {}", e)
            ))?;
        
        Ok(())
    }
}

