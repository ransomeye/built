// Path and File Name : /home/ransomeye/rebuild/core/governor/src/disk.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Disk & FD limits - quota monitoring, audit partition protection, FD exhaustion detection, fail-closed on audit write failure

use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, Disks};
use thiserror::Error;
use tracing::{error, warn, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum DiskGovernanceError {
    #[error("Disk quota exceeded for component: {0}")]
    QuotaExceeded(String),
    #[error("File descriptor exhaustion: {0} used of {1} limit")]
    FdExhaustion(u64, u64),
    #[error("Audit partition write failed: {0}")]
    AuditWriteFailed(String),
    #[error("Disk full: {0}% used on {1}")]
    DiskFull(f32, String),
    #[error("Unsafe disk state: {0}")]
    UnsafeState(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskQuota {
    pub component: String,
    pub max_disk_mb: u64,
    pub current_usage_mb: u64,
    pub path: PathBuf,
    pub is_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f32,
    pub is_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdInfo {
    pub current: u64,
    pub limit: u64,
    pub utilization_percent: f32,
    pub exhaustion_warning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub disk_info: Vec<DiskInfo>,
    pub component_usage: HashMap<String, u64>,
    pub fd_info: FdInfo,
    pub audit_protected: bool,
    pub disk_full_detected: bool,
}

pub struct DiskGovernor {
    quotas: Arc<RwLock<HashMap<String, DiskQuota>>>,
    system: Arc<RwLock<System>>,
    audit_paths: Arc<RwLock<Vec<PathBuf>>>,
    fd_warning_threshold: f32,
    disk_full_threshold: f32,
    last_update: Arc<RwLock<Instant>>,
}

impl DiskGovernor {
    pub fn new(fd_warning_threshold: f32, disk_full_threshold: f32) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
            system: Arc::new(RwLock::new(system)),
            audit_paths: Arc::new(RwLock::new(Vec::new())),
            fd_warning_threshold,
            disk_full_threshold,
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Register a component with disk quota
    pub fn register_component(
        &self,
        component: String,
        max_disk_mb: u64,
        path: PathBuf,
        is_audit: bool,
    ) {
        let mut quotas = self.quotas.write();
        quotas.insert(
            component.clone(),
            DiskQuota {
                component: component.clone(),
                max_disk_mb,
                current_usage_mb: 0,
                path: path.clone(),
                is_audit,
            },
        );
        
        let path_display = path.to_string_lossy().to_string();
        if is_audit {
            let mut audit_paths = self.audit_paths.write();
            audit_paths.push(path);
            info!("Registered audit-protected disk quota for component: {} (max: {}MB, path: {})", 
                  component, max_disk_mb, path_display);
        } else {
            info!("Registered disk quota for component: {} (max: {}MB, path: {})", 
                  component, max_disk_mb, path_display);
        }
    }

    /// Check if component can use disk resources
    pub fn check_quota(&self, component: &str, requested_mb: u64) -> Result<bool, DiskGovernanceError> {
        let quotas = self.quotas.read();
        
        if let Some(quota) = quotas.get(component) {
            let new_usage = quota.current_usage_mb + requested_mb;
            
            if new_usage > quota.max_disk_mb {
                warn!("Disk quota exceeded for component: {} ({}MB > {}MB)", 
                      component, new_usage, quota.max_disk_mb);
                return Err(DiskGovernanceError::QuotaExceeded(component.to_string()));
            }

            // Check disk space availability
            if quota.is_audit {
                // CRITICAL: Audit partitions must always be writable
                let path_clone = quota.path.clone();
                if let Err(e) = self.verify_audit_write(&path_clone) {
                    error!("Audit partition write verification failed: {}", e);
                    return Err(DiskGovernanceError::AuditWriteFailed(
                        format!("Cannot write to audit partition: {:?}", quota.path)
                    ));
                }
            } else {
                // Check disk space for non-audit partitions
                let path_clone = quota.path.clone();
                if let Err(e) = self.check_disk_space(&path_clone, requested_mb) {
                    return Err(e);
                }
            }
        }

        Ok(true)
    }

    /// Verify audit partition is writable (FAIL-CLOSED)
    fn verify_audit_write(&self, path: &Path) -> Result<(), DiskGovernanceError> {
        // Try to create a test file to verify write capability
        let test_file = path.to_path_buf().join(".ransomeye_audit_write_test");
        
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                // Clean up test file
                let _ = fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => {
                let path_display = path.to_string_lossy().to_string();
                error!("Audit partition write test failed: {} (path: {})", e, path_display);
                Err(DiskGovernanceError::AuditWriteFailed(
                    format!("Cannot write to audit partition: {} - {}", path_display, e)
                ))
            }
        }
    }

    /// Check disk space availability
    fn check_disk_space(&self, path: &Path, requested_mb: u64) -> Result<(), DiskGovernanceError> {
        self.update_system_metrics();
        let requested_gb = requested_mb as f64 / 1024.0;
        let path_buf = path.to_path_buf();
        
        // Find the disk for this path
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let mount_point = disk.mount_point();
            
            if path_buf.starts_with(mount_point) {
                let total_space = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
                let available_space = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
                let usage_percent = ((total_space - available_space) / total_space) * 100.0;
                
                // Check if disk is full
                if usage_percent > self.disk_full_threshold as f64 {
                    return Err(DiskGovernanceError::DiskFull(
                        usage_percent as f32,
                        mount_point.to_string_lossy().to_string()
                    ));
                }
                
                // Check if requested space is available
                if available_space < requested_gb {
                    return Err(DiskGovernanceError::UnsafeState(
                        format!("Insufficient disk space: {:.2}GB available, {:.2}GB requested", 
                                available_space, requested_gb)
                    ));
                }
                
                return Ok(());
            }
        }
        
        // If we can't find the disk, assume it's OK (might be a subdirectory)
        Ok(())
    }

    /// Check file descriptor limits
    pub fn check_fd_limits(&self) -> Result<FdInfo, DiskGovernanceError> {
        #[cfg(unix)]
        {
            use libc::{getrlimit, rlimit, RLIMIT_NOFILE};
            
            let mut rlim = rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            
            unsafe {
                if getrlimit(RLIMIT_NOFILE, &mut rlim) != 0 {
                    return Err(DiskGovernanceError::UnsafeState(
                        "Failed to get file descriptor limits".to_string()
                    ));
                }
            }
            
            // Get current FD usage (approximate)
            // On Linux, we can check /proc/self/fd
            let current_fd_count = self.get_current_fd_count();
            let limit = rlim.rlim_max as u64;
            let utilization = (current_fd_count as f32 / limit as f32) * 100.0;
            let exhaustion_warning = utilization > self.fd_warning_threshold;
            
            if exhaustion_warning {
                warn!("FD exhaustion warning: {} used of {} limit ({:.2}%)", 
                      current_fd_count, limit, utilization);
            }
            
            if current_fd_count >= limit {
                return Err(DiskGovernanceError::FdExhaustion(current_fd_count, limit));
            }
            
            Ok(FdInfo {
                current: current_fd_count,
                limit,
                utilization_percent: utilization,
                exhaustion_warning,
            })
        }
        
        #[cfg(not(unix))]
        {
            // Non-Unix systems - return dummy info
            Ok(FdInfo {
                current: 0,
                limit: 1024,
                utilization_percent: 0.0,
                exhaustion_warning: false,
            })
        }
    }

    /// Get current file descriptor count
    #[cfg(unix)]
    fn get_current_fd_count(&self) -> u64 {
        let fd_dir = Path::new("/proc/self/fd");
        if fd_dir.exists() {
            match fs::read_dir(fd_dir) {
                Ok(entries) => entries.count() as u64,
                Err(_) => 0,
            }
        } else {
            0
        }
    }

    /// Update system disk metrics
    fn update_system_metrics(&self) {
        let mut last_update = self.last_update.write();
        let now = Instant::now();
        if now.duration_since(*last_update) >= Duration::from_secs(5) {
            // Disks are refreshed on-demand via Disks::new_with_refreshed_list()
            *last_update = now;
        }
    }

    /// Record disk usage for a component
    pub fn record_usage(&self, component: &str, disk_mb: u64) {
        let mut quotas = self.quotas.write();
        if let Some(quota) = quotas.get_mut(component) {
            quota.current_usage_mb = disk_mb;
        }
    }

    /// Get current disk metrics
    pub fn get_metrics(&self) -> Result<DiskMetrics, DiskGovernanceError> {
        self.update_system_metrics();
        let quotas = self.quotas.read();
        let audit_paths = self.audit_paths.read();
        
        // Collect disk information
        let mut disk_info = Vec::new();
        let mut disk_full_detected = false;
        
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let total = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let available = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let used = total - available;
            let usage_percent = (used / total) * 100.0;
            
            // Check if this is an audit partition
            let is_audit = audit_paths.iter().any(|p| p.starts_with(disk.mount_point()));
            
            if usage_percent > self.disk_full_threshold as f64 {
                disk_full_detected = true;
            }
            
            disk_info.push(DiskInfo {
                path: disk.mount_point().to_string_lossy().to_string(),
                total_gb: total,
                available_gb: available,
                used_gb: used,
                usage_percent: usage_percent as f32,
                is_audit,
            });
        }
        
        // Component usage
        let component_usage: HashMap<String, u64> = quotas
            .iter()
            .map(|(k, v)| (k.clone(), v.current_usage_mb))
            .collect();
        
        // FD information
        let fd_info = self.check_fd_limits()?;
        
        // Verify audit protection
        let audit_protected = audit_paths.iter().all(|p| {
            self.verify_audit_write(p).is_ok()
        });
        
        Ok(DiskMetrics {
            disk_info,
            component_usage,
            fd_info,
            audit_protected,
            disk_full_detected,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disk_governor_creation() {
        let governor = DiskGovernor::new(80.0, 90.0);
        assert_eq!(governor.fd_warning_threshold, 80.0);
        assert_eq!(governor.disk_full_threshold, 90.0);
    }

    #[test]
    fn test_component_registration() {
        let governor = DiskGovernor::new(80.0, 90.0);
        let temp_dir = TempDir::new().unwrap();
        
        governor.register_component(
            "test_component".to_string(),
            1024,
            temp_dir.path().to_path_buf(),
            false,
        );
        
        let quotas = governor.quotas.read();
        assert!(quotas.contains_key("test_component"));
    }

    #[test]
    fn test_audit_partition_protection() {
        let governor = DiskGovernor::new(80.0, 90.0);
        let temp_dir = TempDir::new().unwrap();
        
        governor.register_component(
            "audit_component".to_string(),
            2048,
            temp_dir.path().to_path_buf(),
            true,
        );
        
        // Audit partition should be writable
        let result = governor.check_quota("audit_component", 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fd_limits_check() {
        let governor = DiskGovernor::new(80.0, 90.0);
        
        // Should not fail on valid system
        let result = governor.check_fd_limits();
        assert!(result.is_ok());
    }
}

