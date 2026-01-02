// Path and File Name : /home/ransomeye/rebuild/core/governor/src/cpu.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: CPU governance - per-component quotas, priority scheduling, exhaustion detection, back-pressure signaling

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use thiserror::Error;
use tracing::{error, warn, info, debug};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum CpuGovernanceError {
    #[error("CPU quota exceeded for component: {0}")]
    QuotaExceeded(String),
    #[error("CPU exhaustion detected: {0}% utilization")]
    CpuExhaustion(f32),
    #[error("Back-pressure signal failed: {0}")]
    BackPressureFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ComponentPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone)]
pub struct CpuQuota {
    pub component: String,
    pub priority: ComponentPriority,
    pub max_cpu_percent: f32,
    pub window_seconds: u64,
    pub current_usage: f32,
    // Note: window_start is not serialized (Instant doesn't implement Serialize)
    pub window_start: Instant,
    pub backpressure_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub system_cpu_percent: f32,
    pub component_usage: HashMap<String, f32>,
    pub exhaustion_detected: bool,
    pub backpressure_active: bool,
}

pub struct CpuGovernor {
    quotas: Arc<RwLock<HashMap<String, CpuQuota>>>,
    system: Arc<RwLock<System>>,
    exhaustion_threshold: f32,
    backpressure_signals: Arc<RwLock<HashMap<String, bool>>>,
    last_update: Arc<RwLock<Instant>>,
}

impl CpuGovernor {
    pub fn new(exhaustion_threshold: f32) -> Self {
        let mut system = System::new_all();
        system.refresh_cpu();
        
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
            system: Arc::new(RwLock::new(system)),
            exhaustion_threshold,
            backpressure_signals: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Register a component with CPU quota
    pub fn register_component(
        &self,
        component: String,
        priority: ComponentPriority,
        max_cpu_percent: f32,
        window_seconds: u64,
        backpressure_threshold: f32,
    ) {
        let mut quotas = self.quotas.write();
        quotas.insert(
            component.clone(),
            CpuQuota {
                component: component.clone(),
                priority,
                max_cpu_percent,
                window_seconds,
                current_usage: 0.0,
                window_start: Instant::now(),
                backpressure_threshold,
            },
        );
        info!("Registered CPU quota for component: {} (priority: {:?}, max: {}%)", 
              component, priority, max_cpu_percent);
    }

    /// Check if component can use CPU resources
    pub fn check_quota(&self, component: &str) -> Result<bool, CpuGovernanceError> {
        let quotas = self.quotas.read();
        
        // CRITICAL components always pass
        if let Some(quota) = quotas.get(component) {
            if quota.priority == ComponentPriority::Critical {
                return Ok(true);
            }
        }

        // Update system metrics
        self.update_system_metrics();

        // Check system-wide CPU exhaustion
        let system = self.system.read();
        let cpu_usage = system.global_cpu_info().cpu_usage() as f32;
        
        if cpu_usage > self.exhaustion_threshold {
            warn!("CPU exhaustion detected: {:.2}% (threshold: {:.2}%)", 
                  cpu_usage, self.exhaustion_threshold);
            
            // Check if we can still allow CRITICAL components
            if let Some(quota) = quotas.get(component) {
                if quota.priority == ComponentPriority::Critical {
                    return Ok(true);
                }
            }
            
            return Err(CpuGovernanceError::CpuExhaustion(cpu_usage));
        }

        // Check component-specific quota
        if let Some(quota) = quotas.get(component) {
            let now = Instant::now();
            let mut quota_mut = quota.clone();
            
            // Reset window if expired
            if now.duration_since(quota_mut.window_start) >= Duration::from_secs(quota_mut.window_seconds) {
                quota_mut.current_usage = 0.0;
                quota_mut.window_start = now;
            }

            // Check quota
            if quota_mut.current_usage >= quota_mut.max_cpu_percent {
                warn!("CPU quota exceeded for component: {} ({:.2}% >= {:.2}%)", 
                      component, quota_mut.current_usage, quota_mut.max_cpu_percent);
                return Err(CpuGovernanceError::QuotaExceeded(component.to_string()));
            }

            // Check backpressure threshold
            if quota_mut.current_usage >= quota_mut.backpressure_threshold {
                self.trigger_backpressure(component)?;
            }
        }

        Ok(true)
    }

    /// Record CPU usage for a component
    pub fn record_usage(&self, component: &str, cpu_percent: f32) {
        let mut quotas = self.quotas.write();
        if let Some(quota) = quotas.get_mut(component) {
            quota.current_usage = cpu_percent;
        }
    }

    /// Update system CPU metrics
    fn update_system_metrics(&self) {
        let mut system = self.system.write();
        let mut last_update = self.last_update.write();
        
        let now = Instant::now();
        if now.duration_since(*last_update) >= Duration::from_secs(1) {
            system.refresh_cpu();
            *last_update = now;
        }
    }

    /// Trigger backpressure signal for a component
    fn trigger_backpressure(&self, component: &str) -> Result<(), CpuGovernanceError> {
        let mut signals = self.backpressure_signals.write();
        signals.insert(component.to_string(), true);
        warn!("Backpressure triggered for component: {}", component);
        Ok(())
    }

    /// Check if backpressure is active for a component
    pub fn is_backpressure_active(&self, component: &str) -> bool {
        let signals = self.backpressure_signals.read();
        signals.get(component).copied().unwrap_or(false)
    }

    /// Clear backpressure signal
    pub fn clear_backpressure(&self, component: &str) {
        let mut signals = self.backpressure_signals.write();
        signals.remove(component);
        debug!("Backpressure cleared for component: {}", component);
    }

    /// Get current CPU metrics
    pub fn get_metrics(&self) -> CpuMetrics {
        self.update_system_metrics();
        
        let system = self.system.read();
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        let quotas = self.quotas.read();
        let component_usage: HashMap<String, f32> = quotas
            .iter()
            .map(|(k, v)| (k.clone(), v.current_usage))
            .collect();
        
        let signals = self.backpressure_signals.read();
        let backpressure_active = !signals.is_empty();
        
        CpuMetrics {
            system_cpu_percent: cpu_usage,
            component_usage,
            exhaustion_detected: cpu_usage > self.exhaustion_threshold,
            backpressure_active,
        }
    }

    /// Get priority-ordered component list (for scheduling)
    pub fn get_priority_ordered_components(&self) -> Vec<String> {
        let quotas = self.quotas.read();
        let mut components: Vec<(ComponentPriority, String)> = quotas
            .iter()
            .map(|(k, v)| (v.priority, k.clone()))
            .collect();
        
        components.sort_by(|a, b| b.0.cmp(&a.0)); // Higher priority first
        components.into_iter().map(|(_, k)| k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_governor_creation() {
        let governor = CpuGovernor::new(90.0);
        assert_eq!(governor.exhaustion_threshold, 90.0);
    }

    #[test]
    fn test_component_registration() {
        let governor = CpuGovernor::new(90.0);
        governor.register_component(
            "test_component".to_string(),
            ComponentPriority::High,
            50.0,
            60,
            40.0,
        );
        
        let quotas = governor.quotas.read();
        assert!(quotas.contains_key("test_component"));
    }

    #[test]
    fn test_critical_always_passes() {
        let governor = CpuGovernor::new(90.0);
        governor.register_component(
            "critical_component".to_string(),
            ComponentPriority::Critical,
            100.0,
            60,
            90.0,
        );
        
        // Critical components should always pass
        let result = governor.check_quota("critical_component");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_backpressure_triggering() {
        let governor = CpuGovernor::new(90.0);
        governor.register_component(
            "test_component".to_string(),
            ComponentPriority::Normal,
            50.0,
            60,
            40.0,
        );
        
        // Record usage above backpressure threshold
        governor.record_usage("test_component", 45.0);
        
        // Check quota (should trigger backpressure)
        let _ = governor.check_quota("test_component");
        
        assert!(governor.is_backpressure_active("test_component"));
    }

    #[test]
    fn test_priority_ordering() {
        let governor = CpuGovernor::new(90.0);
        governor.register_component("low".to_string(), ComponentPriority::Low, 10.0, 60, 8.0);
        governor.register_component("critical".to_string(), ComponentPriority::Critical, 100.0, 60, 90.0);
        governor.register_component("normal".to_string(), ComponentPriority::Normal, 50.0, 60, 40.0);
        governor.register_component("high".to_string(), ComponentPriority::High, 75.0, 60, 60.0);
        
        let ordered = governor.get_priority_ordered_components();
        assert_eq!(ordered[0], "critical");
        assert_eq!(ordered[1], "high");
        assert_eq!(ordered[2], "normal");
        assert_eq!(ordered[3], "low");
    }
}

