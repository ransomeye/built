// Path and File Name : /home/ransomeye/rebuild/core/governor/src/degradation.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Fail-safe degradation - explicit degradation logging, critical security functions remain operational, unsafe states trigger controlled shutdown or isolation

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{error, warn, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum DegradationError {
    #[error("Unsafe state detected: {0}")]
    UnsafeState(String),
    #[error("Critical function failure: {0}")]
    CriticalFunctionFailure(String),
    #[error("Degradation threshold exceeded: {0}")]
    DegradationThresholdExceeded(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentState {
    Normal,
    Degraded,
    Isolated,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionCriticality {
    NonCritical,
    Important,
    Critical, // Security functions
}

#[derive(Debug, Clone)]
pub struct ComponentStatus {
    pub component: String,
    pub state: ComponentState,
    pub degradation_reason: Option<String>,
    pub degraded_at: Option<Instant>,
    pub critical_functions_operational: bool,
}

#[derive(Debug, Clone)]
pub struct DegradationEvent {
    pub timestamp: Instant,
    pub component: String,
    pub reason: String,
    pub severity: String,
    pub critical_functions_affected: bool,
}

#[derive(Debug, Clone)]
pub struct DegradationMetrics {
    pub component_states: HashMap<String, ComponentState>,
    pub degradation_events: Vec<DegradationEvent>,
    pub unsafe_state_detected: bool,
    pub critical_functions_operational: bool,
    pub isolated_components: Vec<String>,
    pub shutdown_components: Vec<String>,
}

pub struct DegradationGovernor {
    component_statuses: Arc<RwLock<HashMap<String, ComponentStatus>>>,
    critical_functions: Arc<RwLock<HashMap<String, FunctionCriticality>>>,
    degradation_events: Arc<RwLock<Vec<DegradationEvent>>>,
    degradation_threshold: f32,
    last_update: Arc<RwLock<Instant>>,
}

impl DegradationGovernor {
    pub fn new(degradation_threshold: f32) -> Self {
        Self {
            component_statuses: Arc::new(RwLock::new(HashMap::new())),
            critical_functions: Arc::new(RwLock::new(HashMap::new())),
            degradation_events: Arc::new(RwLock::new(Vec::new())),
            degradation_threshold,
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Register a component
    pub fn register_component(&self, component: String) {
        let mut statuses = self.component_statuses.write();
        statuses.insert(
            component.clone(),
            ComponentStatus {
                component: component.clone(),
                state: ComponentState::Normal,
                degradation_reason: None,
                degraded_at: None,
                critical_functions_operational: true,
            },
        );
        info!("Registered component for degradation monitoring: {}", component);
    }

    /// Register a critical function
    pub fn register_critical_function(
        &self,
        component: String,
        function: String,
        criticality: FunctionCriticality,
    ) {
        let mut functions = self.critical_functions.write();
        let key = format!("{}::{}", component, function);
        functions.insert(key, criticality);
        
        if criticality == FunctionCriticality::Critical {
            info!("Registered critical function: {}::{}", component, function);
        }
    }

    /// Report degradation for a component
    pub fn report_degradation(
        &self,
        component: &str,
        reason: String,
        severity: &str,
    ) -> Result<(), DegradationError> {
        let mut statuses = self.component_statuses.write();
        let mut events = self.degradation_events.write();
        
        // Check if component exists
        if let Some(status) = statuses.get_mut(component) {
            // Verify critical functions are still operational
            let critical_ok = self.verify_critical_functions(component);
            
            if !critical_ok {
                error!("Critical function failure detected for component: {}", component);
                status.state = ComponentState::Isolated;
                status.critical_functions_operational = false;
                
                // Log degradation event
                events.push(DegradationEvent {
                    timestamp: Instant::now(),
                    component: component.to_string(),
                    reason: reason.clone(),
                    severity: severity.to_string(),
                    critical_functions_affected: true,
                });
                
                return Err(DegradationError::CriticalFunctionFailure(
                    format!("Component: {}, Reason: {}", component, reason)
                ));
            }
            
            // Update component state
            status.state = ComponentState::Degraded;
            status.degradation_reason = Some(reason.clone());
            status.degraded_at = Some(Instant::now());
            status.critical_functions_operational = true;
            
            // Log degradation event
            events.push(DegradationEvent {
                timestamp: Instant::now(),
                component: component.to_string(),
                reason: reason.clone(),
                severity: severity.to_string(),
                critical_functions_affected: false,
            });
            
            warn!("Component degraded: {} (reason: {}, severity: {})", 
                  component, reason, severity);
        } else {
            warn!("Degradation reported for unregistered component: {}", component);
        }
        
        Ok(())
    }

    /// Verify critical functions are operational
    fn verify_critical_functions(&self, component: &str) -> bool {
        let functions = self.critical_functions.read();
        
        // Check if component has any critical functions
        let has_critical = functions.iter().any(|(k, v)| {
            k.starts_with(&format!("{}::", component)) && 
            *v == FunctionCriticality::Critical
        });
        
        if !has_critical {
            // Component has no critical functions, so it's OK to degrade
            return true;
        }
        
        // For now, assume critical functions are operational
        // In a real implementation, this would check actual function health
        true
    }

    /// Isolate a component (stop non-critical operations)
    pub fn isolate_component(&self, component: &str, reason: String) -> Result<(), DegradationError> {
        let mut statuses = self.component_statuses.write();
        let mut events = self.degradation_events.write();
        
        if let Some(status) = statuses.get_mut(component) {
            // Verify critical functions before isolation
            if !self.verify_critical_functions(component) {
                return Err(DegradationError::CriticalFunctionFailure(
                    format!("Cannot isolate component with failed critical functions: {}", component)
                ));
            }
            
            status.state = ComponentState::Isolated;
            status.degradation_reason = Some(reason.clone());
            status.degraded_at = Some(Instant::now());
            
            events.push(DegradationEvent {
                timestamp: Instant::now(),
                component: component.to_string(),
                reason,
                severity: "ISOLATION".to_string(),
                critical_functions_affected: false,
            });
            
            warn!("Component isolated: {}", component);
        }
        
        Ok(())
    }

    /// Shutdown a component (controlled shutdown)
    pub fn shutdown_component(&self, component: &str, reason: String) -> Result<(), DegradationError> {
        let mut statuses = self.component_statuses.write();
        let mut events = self.degradation_events.write();
        
        if let Some(status) = statuses.get_mut(component) {
            // Check if component has critical functions
            let has_critical = self.has_critical_functions(component);
            
            if has_critical {
                // Critical functions must remain operational
                // Instead of shutdown, isolate the component
                warn!("Component {} has critical functions - isolating instead of shutting down", 
                      component);
                status.state = ComponentState::Isolated;
            } else {
                status.state = ComponentState::Shutdown;
            }
            
            status.degradation_reason = Some(reason.clone());
            status.degraded_at = Some(Instant::now());
            
            let reason_clone = reason.clone();
            events.push(DegradationEvent {
                timestamp: Instant::now(),
                component: component.to_string(),
                reason,
                severity: "SHUTDOWN".to_string(),
                critical_functions_affected: false,
            });
            
            error!("Component shutdown: {} (reason: {})", component, reason_clone);
        }
        
        Ok(())
    }

    /// Check if component has critical functions
    fn has_critical_functions(&self, component: &str) -> bool {
        let functions = self.critical_functions.read();
        functions.iter().any(|(k, v)| {
            k.starts_with(&format!("{}::", component)) && 
            *v == FunctionCriticality::Critical
        })
    }

    /// Restore component to normal state
    pub fn restore_component(&self, component: &str) {
        let mut statuses = self.component_statuses.write();
        
        if let Some(status) = statuses.get_mut(component) {
            status.state = ComponentState::Normal;
            status.degradation_reason = None;
            status.degraded_at = None;
            status.critical_functions_operational = true;
            
            info!("Component restored to normal: {}", component);
        }
    }

    /// Check for unsafe states
    pub fn check_unsafe_states(&self) -> Result<bool, DegradationError> {
        let statuses = self.component_statuses.read();
        
        // Check if any component is in unsafe state
        for (component, status) in statuses.iter() {
            if !status.critical_functions_operational {
                return Err(DegradationError::UnsafeState(
                    format!("Component {} has failed critical functions", component)
                ));
            }
        }
        
        // Check degradation threshold
        let degraded_count = statuses.values()
            .filter(|s| s.state == ComponentState::Degraded)
            .count();
        
        let total_count = statuses.len();
        let degradation_percent = if total_count > 0 {
            (degraded_count as f32 / total_count as f32) * 100.0
        } else {
            0.0
        };
        
        if degradation_percent > self.degradation_threshold {
            return Err(DegradationError::DegradationThresholdExceeded(
                format!("{:.2}% of components degraded (threshold: {:.2}%)", 
                        degradation_percent, self.degradation_threshold)
            ));
        }
        
        Ok(true)
    }

    /// Get current degradation metrics
    pub fn get_metrics(&self) -> DegradationMetrics {
        let statuses = self.component_statuses.read();
        let events = self.degradation_events.read();
        
        let component_states: HashMap<String, ComponentState> = statuses
            .iter()
            .map(|(k, v)| (k.clone(), v.state))
            .collect();
        
        let isolated_components: Vec<String> = statuses
            .iter()
            .filter(|(_, v)| v.state == ComponentState::Isolated)
            .map(|(k, _)| k.clone())
            .collect();
        
        let shutdown_components: Vec<String> = statuses
            .iter()
            .filter(|(_, v)| v.state == ComponentState::Shutdown)
            .map(|(k, _)| k.clone())
            .collect();
        
        let critical_functions_operational = statuses.values()
            .all(|s| s.critical_functions_operational);
        
        let unsafe_state_detected = self.check_unsafe_states().is_err();
        
        DegradationMetrics {
            component_states,
            degradation_events: events.clone(),
            unsafe_state_detected,
            critical_functions_operational,
            isolated_components,
            shutdown_components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degradation_governor_creation() {
        let governor = DegradationGovernor::new(50.0);
        assert_eq!(governor.degradation_threshold, 50.0);
    }

    #[test]
    fn test_component_registration() {
        let governor = DegradationGovernor::new(50.0);
        governor.register_component("test_component".to_string());
        
        let statuses = governor.component_statuses.read();
        assert!(statuses.contains_key("test_component"));
    }

    #[test]
    fn test_degradation_reporting() {
        let governor = DegradationGovernor::new(50.0);
        governor.register_component("test_component".to_string());
        
        let result = governor.report_degradation(
            "test_component",
            "High CPU usage".to_string(),
            "WARNING",
        );
        assert!(result.is_ok());
        
        let statuses = governor.component_statuses.read();
        let status = statuses.get("test_component").unwrap();
        assert_eq!(status.state, ComponentState::Degraded);
    }

    #[test]
    fn test_critical_function_protection() {
        let governor = DegradationGovernor::new(50.0);
        governor.register_component("secure_component".to_string());
        governor.register_critical_function(
            "secure_component".to_string(),
            "encrypt".to_string(),
            FunctionCriticality::Critical,
        );
        
        // Component with critical functions should not be shutdown
        let result = governor.shutdown_component(
            "secure_component",
            "Test".to_string(),
        );
        assert!(result.is_ok());
        
        let statuses = governor.component_statuses.read();
        let status = statuses.get("secure_component").unwrap();
        // Should be isolated, not shutdown
        assert_eq!(status.state, ComponentState::Isolated);
    }
}

