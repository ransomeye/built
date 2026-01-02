// Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/probe/src/health.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Health monitoring for DPI Probe

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tracing::{error, warn, info};

use super::errors::ProbeError;

/// Health monitor
/// 
/// Tracks component health and detects failures.
/// Lock-free statistics.
pub struct HealthMonitor {
    start_time: u64,
    last_packet_time: AtomicU64,
    packets_processed: AtomicU64,
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
            last_packet_time: AtomicU64::new(now),
            packets_processed: AtomicU64::new(0),
            errors_count: AtomicU64::new(0),
            healthy: AtomicBool::new(true),
            max_idle_time,
        }
    }
    
    /// Record packet processing
    pub fn record_packet(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.last_packet_time.store(now, Ordering::Relaxed);
        self.packets_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record error
    pub fn record_error(&self) {
        self.errors_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Check health status
    pub fn check_health(&self) -> Result<bool, ProbeError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ProbeError::HealthCheckFailed(format!("Time error: {}", e)))?
            .as_secs();
        
        let last_packet = self.last_packet_time.load(Ordering::Relaxed);
        let idle_time = now.saturating_sub(last_packet);
        
        // Check if idle too long
        if idle_time > self.max_idle_time {
            self.healthy.store(false, Ordering::Release);
            warn!("Health check failed: idle for {} seconds (max: {})", 
                idle_time, self.max_idle_time);
            return Ok(false);
        }
        
        // Check error rate
        let errors = self.errors_count.load(Ordering::Relaxed);
        let processed = self.packets_processed.load(Ordering::Relaxed);
        
        if processed > 0 {
            let error_rate = (errors as f64 / processed as f64) * 100.0;
            if error_rate > 10.0 { // More than 10% error rate
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
            packets_processed: self.packets_processed.load(Ordering::Relaxed),
            errors_count: self.errors_count.load(Ordering::Relaxed),
            healthy: self.healthy.load(Ordering::Relaxed),
            last_packet_time: self.last_packet_time.load(Ordering::Relaxed),
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
    pub packets_processed: u64,
    pub errors_count: u64,
    pub healthy: bool,
    pub last_packet_time: u64,
}

