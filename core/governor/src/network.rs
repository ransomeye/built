// Path and File Name : /home/ransomeye/rebuild/core/governor/src/network.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Network governance - connection caps, rate limiting under load, drop non-critical traffic first, never drop CRITICAL telemetry or alerts

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{error, warn, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum NetworkGovernanceError {
    #[error("Connection cap exceeded for component: {0}")]
    ConnectionCapExceeded(String),
    #[error("Rate limit exceeded for component: {0}")]
    RateLimitExceeded(String),
    #[error("Network overload: {0}")]
    NetworkOverload(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TrafficPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3, // Telemetry and alerts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuota {
    pub component: String,
    pub priority: TrafficPriority,
    pub max_connections: u64,
    pub current_connections: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub component: String,
    pub priority: TrafficPriority,
    pub max_requests_per_second: u64,
    pub window_seconds: u64,
    pub current_count: u64,
    // Note: window_start is not serialized (Instant doesn't implement Serialize)
    pub window_start: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub connection_usage: HashMap<String, u64>,
    pub rate_limit_usage: HashMap<String, u64>,
    pub overload_detected: bool,
    pub dropped_connections: u64,
    pub dropped_requests: u64,
}

pub struct NetworkGovernor {
    connection_quotas: Arc<RwLock<HashMap<String, ConnectionQuota>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    dropped_connections: Arc<RwLock<u64>>,
    dropped_requests: Arc<RwLock<u64>>,
    overload_threshold: f32,
}

impl NetworkGovernor {
    pub fn new(overload_threshold: f32) -> Self {
        Self {
            connection_quotas: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            dropped_connections: Arc::new(RwLock::new(0)),
            dropped_requests: Arc::new(RwLock::new(0)),
            overload_threshold,
        }
    }

    /// Register a component with connection quota
    pub fn register_connection_quota(
        &self,
        component: String,
        priority: TrafficPriority,
        max_connections: u64,
    ) {
        let mut quotas = self.connection_quotas.write();
        quotas.insert(
            component.clone(),
            ConnectionQuota {
                component: component.clone(),
                priority,
                max_connections,
                current_connections: 0,
            },
        );
        info!("Registered connection quota for component: {} (priority: {:?}, max: {})", 
              component, priority, max_connections);
    }

    /// Register a component with rate limit
    pub fn register_rate_limit(
        &self,
        component: String,
        priority: TrafficPriority,
        max_requests_per_second: u64,
        window_seconds: u64,
    ) {
        let mut limits = self.rate_limits.write();
        limits.insert(
            component.clone(),
            RateLimit {
                component: component.clone(),
                priority,
                max_requests_per_second,
                window_seconds,
                current_count: 0,
                window_start: Instant::now(),
            },
        );
        info!("Registered rate limit for component: {} (priority: {:?}, max: {}/s)", 
              component, priority, max_requests_per_second);
    }

    /// Check if component can accept a new connection
    pub fn check_connection(&self, component: &str) -> Result<bool, NetworkGovernanceError> {
        let quotas = self.connection_quotas.read();
        
        if let Some(quota) = quotas.get(component) {
            // CRITICAL priority (telemetry/alerts) always passes
            if quota.priority == TrafficPriority::Critical {
                return Ok(true);
            }
            
            // Check connection cap
            if quota.current_connections >= quota.max_connections {
                // Drop non-critical connections
                if quota.priority != TrafficPriority::Critical {
                    let mut dropped = self.dropped_connections.write();
                    *dropped += 1;
                    warn!("Connection cap exceeded for component: {} (dropping non-critical connection)", 
                          component);
                    return Err(NetworkGovernanceError::ConnectionCapExceeded(component.to_string()));
                }
            }
        }
        
        Ok(true)
    }

    /// Record a new connection
    pub fn record_connection(&self, component: &str) {
        let mut quotas = self.connection_quotas.write();
        if let Some(quota) = quotas.get_mut(component) {
            quota.current_connections += 1;
        }
    }

    /// Release a connection
    pub fn release_connection(&self, component: &str) {
        let mut quotas = self.connection_quotas.write();
        if let Some(quota) = quotas.get_mut(component) {
            if quota.current_connections > 0 {
                quota.current_connections -= 1;
            }
        }
    }

    /// Check if component can make a request (rate limiting)
    pub fn check_rate_limit(&self, component: &str) -> Result<bool, NetworkGovernanceError> {
        let mut limits = self.rate_limits.write();
        
        if let Some(limit) = limits.get_mut(component) {
            // CRITICAL priority (telemetry/alerts) always passes
            if limit.priority == TrafficPriority::Critical {
                return Ok(true);
            }
            
            let now = Instant::now();
            
            // Reset window if expired
            if now.duration_since(limit.window_start) >= Duration::from_secs(limit.window_seconds) {
                limit.current_count = 0;
                limit.window_start = now;
            }
            
            // Check rate limit
            if limit.current_count >= limit.max_requests_per_second {
                // Drop non-critical requests
                if limit.priority != TrafficPriority::Critical {
                    let mut dropped = self.dropped_requests.write();
                    *dropped += 1;
                    warn!("Rate limit exceeded for component: {} (dropping non-critical request)", 
                          component);
                    return Err(NetworkGovernanceError::RateLimitExceeded(component.to_string()));
                }
            }
            
            limit.current_count += 1;
        }
        
        Ok(true)
    }

    /// Check network overload condition
    pub fn check_overload(&self) -> Result<bool, NetworkGovernanceError> {
        let quotas = self.connection_quotas.read();
        let limits = self.rate_limits.read();
        
        // Calculate total utilization
        let mut total_connections = 0u64;
        let mut max_connections = 0u64;
        
        for quota in quotas.values() {
            total_connections += quota.current_connections;
            max_connections += quota.max_connections;
        }
        
        let connection_utilization = if max_connections > 0 {
            (total_connections as f32 / max_connections as f32) * 100.0
        } else {
            0.0
        };
        
        // Check if overloaded
        if connection_utilization > self.overload_threshold {
            warn!("Network overload detected: {:.2}% connection utilization (threshold: {:.2}%)", 
                  connection_utilization, self.overload_threshold);
            
            // In overload, drop non-critical traffic
            // CRITICAL traffic (telemetry/alerts) is never dropped
            return Err(NetworkGovernanceError::NetworkOverload(
                format!("Connection utilization: {:.2}%", connection_utilization)
            ));
        }
        
        Ok(true)
    }

    /// Get current network metrics
    pub fn get_metrics(&self) -> NetworkMetrics {
        let quotas = self.connection_quotas.read();
        let limits = self.rate_limits.read();
        
        let connection_usage: HashMap<String, u64> = quotas
            .iter()
            .map(|(k, v)| (k.clone(), v.current_connections))
            .collect();
        
        let rate_limit_usage: HashMap<String, u64> = limits
            .iter()
            .map(|(k, v)| (k.clone(), v.current_count))
            .collect();
        
        let overload_detected = self.check_overload().is_err();
        
        let dropped_connections = *self.dropped_connections.read();
        let dropped_requests = *self.dropped_requests.read();
        
        NetworkMetrics {
            connection_usage,
            rate_limit_usage,
            overload_detected,
            dropped_connections,
            dropped_requests,
        }
    }

    /// Reset dropped counters (for metrics)
    pub fn reset_dropped_counters(&self) {
        let mut dropped_conn = self.dropped_connections.write();
        let mut dropped_req = self.dropped_requests.write();
        *dropped_conn = 0;
        *dropped_req = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_governor_creation() {
        let governor = NetworkGovernor::new(80.0);
        assert_eq!(governor.overload_threshold, 80.0);
    }

    #[test]
    fn test_connection_quota_registration() {
        let governor = NetworkGovernor::new(80.0);
        governor.register_connection_quota(
            "test_component".to_string(),
            TrafficPriority::Normal,
            100,
        );
        
        let quotas = governor.connection_quotas.read();
        assert!(quotas.contains_key("test_component"));
    }

    #[test]
    fn test_critical_always_passes() {
        let governor = NetworkGovernor::new(80.0);
        governor.register_connection_quota(
            "critical_telemetry".to_string(),
            TrafficPriority::Critical,
            100,
        );
        
        // Critical components should always pass
        let result = governor.check_connection("critical_telemetry");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_rate_limiting() {
        let governor = NetworkGovernor::new(80.0);
        governor.register_rate_limit(
            "test_component".to_string(),
            TrafficPriority::Normal,
            10,
            1,
        );
        
        // Make 10 requests (should all pass)
        for _ in 0..10 {
            let result = governor.check_rate_limit("test_component");
            assert!(result.is_ok());
        }
        
        // 11th request should fail (non-critical)
        let result = governor.check_rate_limit("test_component");
        assert!(result.is_err());
    }

    #[test]
    fn test_critical_rate_limit_always_passes() {
        let governor = NetworkGovernor::new(80.0);
        governor.register_rate_limit(
            "critical_alerts".to_string(),
            TrafficPriority::Critical,
            10,
            1,
        );
        
        // Even if we exceed the limit, critical should pass
        for _ in 0..20 {
            let result = governor.check_rate_limit("critical_alerts");
            assert!(result.is_ok());
        }
    }
}

