// Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/agent/src/health.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Health monitoring for Linux Agent

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

use super::errors::AgentError;

/// Health monitor
/// 
/// Tracks component health and detects failures.
/// Lock-free statistics.
pub struct HealthMonitor {
    start_time: u64,
    last_event_time: AtomicU64,
    events_processed: AtomicU64,
    errors_count: AtomicU64,
    healthy: AtomicBool,
    max_idle_time: u64, // seconds
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(max_idle_time: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            start_time: now,
            last_event_time: AtomicU64::new(now),
            events_processed: AtomicU64::new(0),
            errors_count: AtomicU64::new(0),
            healthy: AtomicBool::new(true),
            max_idle_time,
        }
    }
    
    /// Record event processing
    pub fn record_event(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.last_event_time.store(now, Ordering::Relaxed);
        self.events_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record error
    pub fn record_error(&self) {
        self.errors_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Check health status
    pub fn check_health(&self) -> Result<bool, AgentError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AgentError::HealthCheckFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        let last_event = self.last_event_time.load(Ordering::Relaxed);
        let idle_time = now.saturating_sub(last_event);
        
        if idle_time > self.max_idle_time {
            self.healthy.store(false, Ordering::Release);
            warn!("Health check failed: idle for {} seconds (max: {})", 
                idle_time, self.max_idle_time);
            return Ok(false);
        }
        
        let errors = self.errors_count.load(Ordering::Relaxed);
        let processed = self.events_processed.load(Ordering::Relaxed);
        
        if processed > 0 {
            let error_rate = (errors as f64 / processed as f64) * 100.0;
            if error_rate > 10.0 {
                self.healthy.store(false, Ordering::Release);
                warn!("Health check failed: error rate {:.2}% (threshold: 10%)", error_rate);
                return Ok(false);
            }
        }
        
        self.healthy.store(true, Ordering::Release);
        Ok(true)
    }
    
    /// Get health statistics
    pub fn stats(&self) -> HealthStats {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        HealthStats {
            uptime: now.saturating_sub(self.start_time),
            events_processed: self.events_processed.load(Ordering::Relaxed),
            errors_count: self.errors_count.load(Ordering::Relaxed),
            healthy: self.healthy.load(Ordering::Relaxed),
            last_event_time: self.last_event_time.load(Ordering::Relaxed),
        }
    }
    
    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct HealthStats {
    pub uptime: u64,
    pub events_processed: u64,
    pub errors_count: u64,
    pub healthy: bool,
    pub last_event_time: u64,
}

