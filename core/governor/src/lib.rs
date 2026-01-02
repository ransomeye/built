// Path and File Name : /home/ransomeye/rebuild/core/governor/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Resource governance orchestrator - coordinates CPU, memory, disk, network, and degradation governance across all RansomEye components

pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod degradation;

use cpu::{CpuGovernor, ComponentPriority as CpuPriority};
use memory::{MemoryGovernor, MemoryGovernanceError};
use disk::{DiskGovernor, DiskGovernanceError};
use network::{NetworkGovernor, TrafficPriority as NetPriority};
use degradation::{DegradationGovernor, FunctionCriticality};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum ResourceGovernanceError {
    #[error("CPU governance error: {0}")]
    Cpu(#[from] cpu::CpuGovernanceError),
    #[error("Memory governance error: {0}")]
    Memory(#[from] MemoryGovernanceError),
    #[error("Disk governance error: {0}")]
    Disk(#[from] DiskGovernanceError),
    #[error("Network governance error: {0}")]
    Network(#[from] network::NetworkGovernanceError),
    #[error("Degradation error: {0}")]
    Degradation(#[from] degradation::DegradationError),
    #[error("Unsafe system state: {0}")]
    UnsafeState(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGovernanceConfig {
    pub cpu_exhaustion_threshold: f32,
    pub memory_oom_threshold: f32,
    pub fd_warning_threshold: f32,
    pub disk_full_threshold: f32,
    pub network_overload_threshold: f32,
    pub degradation_threshold: f32,
}

impl Default for ResourceGovernanceConfig {
    fn default() -> Self {
        Self {
            cpu_exhaustion_threshold: 90.0,
            memory_oom_threshold: 85.0,
            fd_warning_threshold: 80.0,
            disk_full_threshold: 90.0,
            network_overload_threshold: 80.0,
            degradation_threshold: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentResourceLimits {
    pub cpu_max_percent: f32,
    pub cpu_window_seconds: u64,
    pub cpu_backpressure_threshold: f32,
    pub memory_max_mb: u64,
    pub memory_oom_threshold: f32,
    pub disk_max_mb: u64,
    pub disk_path: String,
    pub is_audit: bool,
    pub connection_max: u64,
    pub rate_limit_per_second: u64,
    pub rate_limit_window_seconds: u64,
    pub is_critical: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceGovernanceMetrics {
    pub cpu_metrics: cpu::CpuMetrics,
    pub memory_metrics: memory::MemoryMetrics,
    pub disk_metrics: disk::DiskMetrics,
    pub network_metrics: network::NetworkMetrics,
    pub degradation_metrics: degradation::DegradationMetrics,
    // Note: timestamp is not serialized (Instant doesn't implement Serialize)
    pub timestamp: Instant,
}

/// Main resource governance orchestrator
pub struct ResourceGovernor {
    cpu: Arc<CpuGovernor>,
    memory: Arc<MemoryGovernor>,
    disk: Arc<DiskGovernor>,
    network: Arc<NetworkGovernor>,
    degradation: Arc<DegradationGovernor>,
    config: ResourceGovernanceConfig,
    registered_components: Arc<RwLock<HashMap<String, ComponentResourceLimits>>>,
}

impl ResourceGovernor {
    /// Create new resource governor with configuration
    pub fn new(config: ResourceGovernanceConfig) -> Self {
        let cpu = Arc::new(CpuGovernor::new(config.cpu_exhaustion_threshold));
        let memory = Arc::new(MemoryGovernor::new(config.memory_oom_threshold));
        let disk = Arc::new(DiskGovernor::new(
            config.fd_warning_threshold,
            config.disk_full_threshold,
        ));
        let network = Arc::new(NetworkGovernor::new(config.network_overload_threshold));
        let degradation = Arc::new(DegradationGovernor::new(config.degradation_threshold));
        
        info!("Resource governor initialized with thresholds: CPU={}%, Memory={}%, Disk={}%, Network={}%, Degradation={}%",
              config.cpu_exhaustion_threshold,
              config.memory_oom_threshold,
              config.disk_full_threshold,
              config.network_overload_threshold,
              config.degradation_threshold);
        
        Self {
            cpu,
            memory,
            disk,
            network,
            degradation,
            config,
            registered_components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component with resource limits
    pub fn register_component(
        &self,
        component: String,
        limits: ComponentResourceLimits,
    ) -> Result<(), ResourceGovernanceError> {
        // Register with CPU governor
        let cpu_priority = if limits.is_critical {
            CpuPriority::Critical
        } else {
            CpuPriority::Normal
        };
        self.cpu.register_component(
            component.clone(),
            cpu_priority,
            limits.cpu_max_percent,
            limits.cpu_window_seconds,
            limits.cpu_backpressure_threshold,
        );
        
        // Register with memory governor
        self.memory.register_component(
            component.clone(),
            limits.memory_max_mb,
            limits.memory_oom_threshold,
            limits.is_critical,
        );
        
        // Register with disk governor
        self.disk.register_component(
            component.clone(),
            limits.disk_max_mb,
            std::path::PathBuf::from(&limits.disk_path),
            limits.is_audit,
        );
        
        // Register with network governor
        let net_priority = if limits.is_critical {
            NetPriority::Critical
        } else {
            NetPriority::Normal
        };
        self.network.register_connection_quota(
            component.clone(),
            net_priority,
            limits.connection_max,
        );
        self.network.register_rate_limit(
            component.clone(),
            net_priority,
            limits.rate_limit_per_second,
            limits.rate_limit_window_seconds,
        );
        
        // Register with degradation governor
        self.degradation.register_component(component.clone());
        
        // Store limits
        let mut registered = self.registered_components.write();
        registered.insert(component.clone(), limits);
        
        info!("Registered component with resource limits: {}", component);
        Ok(())
    }

    /// Register a critical function for a component
    pub fn register_critical_function(
        &self,
        component: String,
        function: String,
    ) {
        self.degradation.register_critical_function(
            component,
            function,
            FunctionCriticality::Critical,
        );
    }

    /// Check if component can use resources
    pub fn check_resources(
        &self,
        component: &str,
        cpu_percent: Option<f32>,
        memory_mb: Option<u64>,
        disk_mb: Option<u64>,
    ) -> Result<bool, ResourceGovernanceError> {
        // Check CPU
        if let Some(cpu) = cpu_percent {
            self.cpu.check_quota(component)?;
            self.cpu.record_usage(component, cpu);
        }
        
        // Check memory
        if let Some(mem) = memory_mb {
            self.memory.check_quota(component, mem)?;
            self.memory.record_usage(component, mem);
        }
        
        // Check disk
        if let Some(disk) = disk_mb {
            self.disk.check_quota(component, disk)?;
            self.disk.record_usage(component, disk);
        }
        
        // Check network (connection and rate limit)
        self.network.check_connection(component)?;
        self.network.check_rate_limit(component)?;
        
        // Check for unsafe states
        self.degradation.check_unsafe_states()?;
        
        Ok(true)
    }

    /// Get comprehensive resource metrics
    pub fn get_metrics(&self) -> Result<ResourceGovernanceMetrics, ResourceGovernanceError> {
        let cpu_metrics = self.cpu.get_metrics();
        let memory_metrics = self.memory.get_metrics()?;
        let disk_metrics = self.disk.get_metrics()?;
        let network_metrics = self.network.get_metrics();
        let degradation_metrics = self.degradation.get_metrics();
        
        Ok(ResourceGovernanceMetrics {
            cpu_metrics,
            memory_metrics,
            disk_metrics,
            network_metrics,
            degradation_metrics,
            timestamp: Instant::now(),
        })
    }

    /// Verify system safety (fail-closed check)
    pub fn verify_system_safety(&self) -> Result<bool, ResourceGovernanceError> {
        // Verify SWAP configuration (NO 64GB cap)
        self.memory.verify_swap_configuration()
            .map_err(|e| ResourceGovernanceError::Memory(e))?;
        
        // Check for unsafe states
        self.degradation.check_unsafe_states()
            .map_err(|e| ResourceGovernanceError::Degradation(e))?;
        
        // Verify audit partitions are writable
        let disk_metrics = self.disk.get_metrics()?;
        if !disk_metrics.audit_protected {
            return Err(ResourceGovernanceError::UnsafeState(
                "Audit partitions are not writable".to_string()
            ));
        }
        
        // Check critical functions are operational
        let degradation_metrics = self.degradation.get_metrics();
        if !degradation_metrics.critical_functions_operational {
            return Err(ResourceGovernanceError::UnsafeState(
                "Critical security functions are not operational".to_string()
            ));
        }
        
        Ok(true)
    }

    /// Report degradation for a component
    pub fn report_degradation(
        &self,
        component: &str,
        reason: String,
        severity: &str,
    ) -> Result<(), ResourceGovernanceError> {
        self.degradation.report_degradation(component, reason, severity)
            .map_err(|e| ResourceGovernanceError::Degradation(e))?;
        Ok(())
    }

    /// Get CPU governor (for direct access if needed)
    pub fn cpu(&self) -> &Arc<CpuGovernor> {
        &self.cpu
    }

    /// Get memory governor (for direct access if needed)
    pub fn memory(&self) -> &Arc<MemoryGovernor> {
        &self.memory
    }

    /// Get disk governor (for direct access if needed)
    pub fn disk(&self) -> &Arc<DiskGovernor> {
        &self.disk
    }

    /// Get network governor (for direct access if needed)
    pub fn network(&self) -> &Arc<NetworkGovernor> {
        &self.network
    }

    /// Get degradation governor (for direct access if needed)
    pub fn degradation(&self) -> &Arc<DegradationGovernor> {
        &self.degradation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_governor_creation() {
        let config = ResourceGovernanceConfig::default();
        let governor = ResourceGovernor::new(config);
        
        // Verify all sub-governors are initialized
        let _ = governor.cpu();
        let _ = governor.memory();
        let _ = governor.disk();
        let _ = governor.network();
        let _ = governor.degradation();
    }

    #[test]
    fn test_component_registration() {
        let config = ResourceGovernanceConfig::default();
        let governor = ResourceGovernor::new(config);
        
        let limits = ComponentResourceLimits {
            cpu_max_percent: 50.0,
            cpu_window_seconds: 60,
            cpu_backpressure_threshold: 40.0,
            memory_max_mb: 1024,
            memory_oom_threshold: 80.0,
            disk_max_mb: 2048,
            disk_path: "/tmp".to_string(),
            is_audit: false,
            connection_max: 100,
            rate_limit_per_second: 1000,
            rate_limit_window_seconds: 1,
            is_critical: false,
        };
        
        let result = governor.register_component("test_component".to_string(), limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_critical_component_always_passes() {
        let config = ResourceGovernanceConfig::default();
        let governor = ResourceGovernor::new(config);
        
        let limits = ComponentResourceLimits {
            cpu_max_percent: 100.0,
            cpu_window_seconds: 60,
            cpu_backpressure_threshold: 90.0,
            memory_max_mb: 4096,
            memory_oom_threshold: 90.0,
            disk_max_mb: 8192,
            disk_path: "/tmp".to_string(),
            is_audit: true,
            connection_max: 1000,
            rate_limit_per_second: 10000,
            rate_limit_window_seconds: 1,
            is_critical: true,
        };
        
        let result = governor.register_component("critical_component".to_string(), limits);
        assert!(result.is_ok());
        
        // Critical components should pass resource checks
        let result = governor.check_resources("critical_component", Some(100.0), Some(4096), Some(8192));
        assert!(result.is_ok());
    }
}
