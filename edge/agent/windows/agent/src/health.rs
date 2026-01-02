// Path and File Name : /home/ransomeye/rebuild/ransomeye_windows_agent/agent/src/health.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Health monitoring for Windows Agent

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
use tracing::{error, warn, info, debug};

use super::errors::AgentError;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health monitor
/// 
/// Monitors agent health: ETW session status, event processing, memory usage.
pub struct HealthMonitor {
    status: Arc<AtomicU64>, // 0=Healthy, 1=Degraded, 2=Unhealthy
    etw_running: Arc<AtomicBool>,
    events_processed: Arc<AtomicU64>,
    events_dropped: Arc<AtomicU64>,
    last_health_check: Arc<std::sync::atomic::AtomicU64>,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new() -> Self {
        Self {
            status: Arc::new(AtomicU64::new(0)), // Healthy
            etw_running: Arc::new(AtomicBool::new(false)),
            events_processed: Arc::new(AtomicU64::new(0)),
            events_dropped: Arc::new(AtomicU64::new(0)),
            last_health_check: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Update ETW running status
    pub fn set_etw_running(&self, running: bool) {
        self.etw_running.store(running, Ordering::Release);
        self.update_health();
    }
    
    /// Increment events processed
    pub fn increment_events_processed(&self) {
        self.events_processed.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Increment events dropped
    pub fn increment_events_dropped(&self) {
        self.events_dropped.fetch_add(1, Ordering::AcqRel);
        self.update_health();
    }
    
    /// Update health status
    fn update_health(&self) {
        let etw_running = self.etw_running.load(Ordering::Acquire);
        let events_processed = self.events_processed.load(Ordering::Acquire);
        let events_dropped = self.events_dropped.load(Ordering::Acquire);
        
        let status = if !etw_running {
            // ETW not running - unhealthy
            HealthStatus::Unhealthy
        } else if events_dropped > 0 && events_dropped as f64 / (events_processed + events_dropped) as f64 > 0.1 {
            // More than 10% events dropped - degraded
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        self.status.store(status as u64, Ordering::Release);
    }
    
    /// Get health status
    pub fn get_status(&self) -> HealthStatus {
        self.update_health();
        
        match self.status.load(Ordering::Acquire) {
            0 => HealthStatus::Healthy,
            1 => HealthStatus::Degraded,
            2 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unhealthy,
        }
    }
    
    /// Get health report
    pub fn get_health_report(&self) -> HealthReport {
        HealthReport {
            status: self.get_status(),
            etw_running: self.etw_running.load(Ordering::Acquire),
            events_processed: self.events_processed.load(Ordering::Acquire),
            events_dropped: self.events_dropped.load(Ordering::Acquire),
        }
    }
    
    /// Check health (returns error if unhealthy)
    pub fn check_health(&self) -> Result<(), AgentError> {
        let status = self.get_status();
        
        match status {
            HealthStatus::Healthy => Ok(()),
            HealthStatus::Degraded => {
                warn!("Health check: Degraded");
                Ok(())
            }
            HealthStatus::Unhealthy => {
                Err(AgentError::HealthCheckFailed(
                    format!("Agent is unhealthy: ETW running={}, events_processed={}, events_dropped={}",
                        self.etw_running.load(Ordering::Acquire),
                        self.events_processed.load(Ordering::Acquire),
                        self.events_dropped.load(Ordering::Acquire)
                    )
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub etw_running: bool,
    pub events_processed: u64,
    pub events_dropped: u64,
}

