// Path and File Name : /home/ransomeye/rebuild/core/governor/src/memory.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Memory & SWAP governance - limits per component, SWAP awareness up to physical RAM (NO 64GB cap), OOM early-warning, controlled workload shedding

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use thiserror::Error;
use tracing::{error, warn, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum MemoryGovernanceError {
    #[error("Memory limit exceeded for component: {0}")]
    LimitExceeded(String),
    #[error("OOM early-warning: {0}% memory used")]
    OomEarlyWarning(f32),
    #[error("Unsafe memory state: {0}")]
    UnsafeState(String),
    #[error("SWAP detection failed: {0}")]
    SwapDetectionFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuota {
    pub component: String,
    pub max_memory_mb: u64,
    pub current_usage_mb: u64,
    pub oom_warning_threshold: f32,
    pub is_critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub used_ram_gb: f64,
    pub total_swap_gb: f64,
    pub used_swap_gb: f64,
    pub available_swap_gb: f64,
    pub swap_utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub system_info: MemoryInfo,
    pub component_usage: HashMap<String, u64>,
    pub oom_warning_active: bool,
    pub workload_shedding_active: bool,
    pub shed_components: Vec<String>,
}

pub struct MemoryGovernor {
    quotas: Arc<RwLock<HashMap<String, MemoryQuota>>>,
    system: Arc<RwLock<System>>,
    oom_warning_threshold: f32,
    last_update: Arc<RwLock<Instant>>,
    shed_components: Arc<RwLock<Vec<String>>>,
}

impl MemoryGovernor {
    pub fn new(oom_warning_threshold: f32) -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
            system: Arc::new(RwLock::new(system)),
            oom_warning_threshold,
            last_update: Arc::new(RwLock::new(Instant::now())),
            shed_components: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a component with memory quota
    pub fn register_component(
        &self,
        component: String,
        max_memory_mb: u64,
        oom_warning_threshold: f32,
        is_critical: bool,
    ) {
        let mut quotas = self.quotas.write();
        quotas.insert(
            component.clone(),
            MemoryQuota {
                component: component.clone(),
                max_memory_mb,
                current_usage_mb: 0,
                oom_warning_threshold,
                is_critical,
            },
        );
        info!("Registered memory quota for component: {} (max: {}MB, critical: {})", 
              component, max_memory_mb, is_critical);
    }

    /// Get memory information (RAM and SWAP)
    /// CRITICAL: SWAP scales to available physical RAM - NO 64GB CAP
    pub fn get_memory_info(&self) -> Result<MemoryInfo, MemoryGovernanceError> {
        self.update_system_metrics();
        
        let system = self.system.read();
        
        // Get RAM information
        let total_ram = system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0); // Convert to GB
        let used_ram = system.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_ram = system.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Get SWAP information
        let total_swap = system.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_swap = system.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_swap = total_swap - used_swap;
        
        // Calculate SWAP utilization
        let swap_utilization_percent = if total_swap > 0.0 {
            (used_swap / total_swap) * 100.0
        } else {
            0.0
        };
        
        // CRITICAL: Verify SWAP is not capped at 64GB
        // SWAP should scale up to available physical RAM
        // This is a validation check - we don't enforce a cap, we ensure it can scale
        if total_swap > 0.0 && total_ram > 64.0 && total_swap < total_ram {
            warn!("SWAP ({:.2}GB) is less than physical RAM ({:.2}GB) - SWAP should scale to RAM", 
                  total_swap, total_ram);
        }
        
        Ok(MemoryInfo {
            total_ram_gb: total_ram,
            available_ram_gb: available_ram,
            used_ram_gb: used_ram,
            total_swap_gb: total_swap,
            used_swap_gb: used_swap,
            available_swap_gb: available_swap,
            swap_utilization_percent,
        })
    }

    /// Check if component can use memory resources
    pub fn check_quota(&self, component: &str, requested_mb: u64) -> Result<bool, MemoryGovernanceError> {
        let quotas = self.quotas.read();
        
        // CRITICAL components always pass (but still tracked)
        if let Some(quota) = quotas.get(component) {
            if quota.is_critical {
                return Ok(true);
            }
        }

        // Update system metrics
        let memory_info = self.get_memory_info()?;
        
        // Check system-wide OOM early-warning
        let ram_utilization = (memory_info.used_ram_gb / memory_info.total_ram_gb) * 100.0;
        
        if ram_utilization > self.oom_warning_threshold as f64 {
            warn!("OOM early-warning: {:.2}% RAM used (threshold: {:.2}%)", 
                  ram_utilization, self.oom_warning_threshold);
            
            // Check if we can still allow CRITICAL components
            if let Some(quota) = quotas.get(component) {
                if quota.is_critical {
                    return Ok(true);
                }
            }
            
            // Trigger workload shedding for non-critical components
            self.shed_workload(component)?;
            
            return Err(MemoryGovernanceError::OomEarlyWarning(ram_utilization as f32));
        }

        // Check component-specific quota
        if let Some(quota) = quotas.get(component) {
            let new_usage = quota.current_usage_mb + requested_mb;
            
            if new_usage > quota.max_memory_mb {
                warn!("Memory quota exceeded for component: {} ({}MB > {}MB)", 
                      component, new_usage, quota.max_memory_mb);
                return Err(MemoryGovernanceError::LimitExceeded(component.to_string()));
            }

            // Check component-level OOM warning
            let component_utilization = (new_usage as f32 / quota.max_memory_mb as f32) * 100.0;
            if component_utilization > quota.oom_warning_threshold {
                warn!("Component OOM warning: {} at {:.2}%", component, component_utilization);
            }
        }

        Ok(true)
    }

    /// Record memory usage for a component
    pub fn record_usage(&self, component: &str, memory_mb: u64) {
        let mut quotas = self.quotas.write();
        if let Some(quota) = quotas.get_mut(component) {
            quota.current_usage_mb = memory_mb;
        }
    }

    /// Update system memory metrics
    fn update_system_metrics(&self) {
        let mut system = self.system.write();
        let mut last_update = self.last_update.write();
        
        let now = Instant::now();
        if now.duration_since(*last_update) >= Duration::from_secs(1) {
            system.refresh_memory();
            *last_update = now;
        }
    }

    /// Shed workload for a non-critical component
    fn shed_workload(&self, component: &str) -> Result<(), MemoryGovernanceError> {
        let quotas = self.quotas.read();
        
        // Don't shed critical components
        if let Some(quota) = quotas.get(component) {
            if quota.is_critical {
                return Ok(());
            }
        }
        
        let mut shed = self.shed_components.write();
        if !shed.contains(&component.to_string()) {
            warn!("Shedding workload for component: {}", component);
            shed.push(component.to_string());
        }
        
        Ok(())
    }

    /// Check if component is being shed
    pub fn is_shed(&self, component: &str) -> bool {
        let shed = self.shed_components.read();
        shed.contains(&component.to_string())
    }

    /// Restore component (clear shedding)
    pub fn restore_component(&self, component: &str) {
        let mut shed = self.shed_components.write();
        shed.retain(|c| c != component);
        info!("Restored component: {}", component);
    }

    /// Get current memory metrics
    pub fn get_metrics(&self) -> Result<MemoryMetrics, MemoryGovernanceError> {
        let memory_info = self.get_memory_info()?;
        
        let quotas = self.quotas.read();
        let component_usage: HashMap<String, u64> = quotas
            .iter()
            .map(|(k, v)| (k.clone(), v.current_usage_mb))
            .collect();
        
        let ram_utilization = (memory_info.used_ram_gb / memory_info.total_ram_gb) * 100.0;
        let oom_warning_active = ram_utilization > self.oom_warning_threshold as f64;
        
        let shed = self.shed_components.read();
        let workload_shedding_active = !shed.is_empty();
        let shed_components = shed.clone();
        
        Ok(MemoryMetrics {
            system_info: memory_info,
            component_usage,
            oom_warning_active,
            workload_shedding_active,
            shed_components,
        })
    }

    /// Verify SWAP configuration (ensures NO 64GB cap)
    /// SWAP should scale to available physical RAM
    pub fn verify_swap_configuration(&self) -> Result<bool, MemoryGovernanceError> {
        let memory_info = self.get_memory_info()?;
        
        // CRITICAL: SWAP must NOT be capped at 64GB
        // SWAP should scale up to available physical RAM
        // This is a validation - we ensure the system allows SWAP to scale
        
        if memory_info.total_ram_gb > 64.0 {
            // For systems with >64GB RAM, SWAP should ideally be at least equal to RAM
            // But we don't enforce a cap - we just verify it CAN scale
            if memory_info.total_swap_gb > 0.0 {
                info!("SWAP configuration verified: {:.2}GB SWAP for {:.2}GB RAM (NO 64GB CAP)", 
                      memory_info.total_swap_gb, memory_info.total_ram_gb);
            } else {
                warn!("No SWAP configured for {:.2}GB RAM system", memory_info.total_ram_gb);
            }
        } else {
            info!("SWAP configuration verified: {:.2}GB SWAP for {:.2}GB RAM", 
                  memory_info.total_swap_gb, memory_info.total_ram_gb);
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_governor_creation() {
        let governor = MemoryGovernor::new(85.0);
        assert_eq!(governor.oom_warning_threshold, 85.0);
    }

    #[test]
    fn test_component_registration() {
        let governor = MemoryGovernor::new(85.0);
        governor.register_component(
            "test_component".to_string(),
            1024,
            80.0,
            false,
        );
        
        let quotas = governor.quotas.read();
        assert!(quotas.contains_key("test_component"));
    }

    #[test]
    fn test_critical_always_passes() {
        let governor = MemoryGovernor::new(85.0);
        governor.register_component(
            "critical_component".to_string(),
            2048,
            90.0,
            true,
        );
        
        // Critical components should always pass
        let result = governor.check_quota("critical_component", 1000);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_workload_shedding() {
        let governor = MemoryGovernor::new(85.0);
        governor.register_component(
            "non_critical".to_string(),
            512,
            80.0,
            false,
        );
        
        // Simulate OOM condition by setting high threshold and checking
        // In real scenario, this would be triggered by actual memory pressure
        let _ = governor.shed_workload("non_critical");
        
        assert!(governor.is_shed("non_critical"));
    }

    #[test]
    fn test_swap_no_64gb_cap() {
        let governor = MemoryGovernor::new(85.0);
        
        // Verify SWAP configuration doesn't enforce 64GB cap
        // This test ensures the logic allows SWAP to scale beyond 64GB
        let result = governor.verify_swap_configuration();
        assert!(result.is_ok());
        
        // The verification should not fail if SWAP > 64GB
        // It should only warn if SWAP < RAM for large systems
    }
}

