// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/preflight.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Preflight checks - validates OS, disk space, time synchronization, and permissions before installation

use sysinfo::{System, Disks};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn, error};

use crate::errors::OperationsError;

/// Preflight checker - validates system requirements before installation
pub struct PreflightChecker {
    project_root: String,
    min_disk_gb: u64,
}

#[derive(Debug, Clone)]
pub struct PreflightResult {
    pub os_supported: bool,
    pub disk_space_sufficient: bool,
    pub time_synchronized: bool,
    pub permissions_ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl PreflightChecker {
    pub fn new(project_root: &str, min_disk_gb: u64) -> Self {
        Self {
            project_root: project_root.to_string(),
            min_disk_gb,
        }
    }
    
    /// Run all preflight checks
    pub fn check_all(&self) -> Result<PreflightResult, OperationsError> {
        let mut result = PreflightResult {
            os_supported: false,
            disk_space_sufficient: false,
            time_synchronized: false,
            permissions_ok: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Check OS
        match self.check_os() {
            Ok(true) => {
                result.os_supported = true;
                debug!("OS check passed");
            }
            Ok(false) => {
                result.errors.push("OS not supported".to_string());
            }
            Err(e) => {
                result.errors.push(format!("OS check failed: {}", e));
            }
        }
        
        // Check disk space
        match self.check_disk_space() {
            Ok(true) => {
                result.disk_space_sufficient = true;
                debug!("Disk space check passed");
            }
            Ok(false) => {
                result.errors.push(format!("Insufficient disk space (minimum {} GB required)", self.min_disk_gb));
            }
            Err(e) => {
                result.errors.push(format!("Disk space check failed: {}", e));
            }
        }
        
        // Check time synchronization
        match self.check_time_sync() {
            Ok(true) => {
                result.time_synchronized = true;
                debug!("Time synchronization check passed");
            }
            Ok(false) => {
                result.warnings.push("Time may not be synchronized".to_string());
            }
            Err(e) => {
                result.warnings.push(format!("Time sync check failed: {}", e));
            }
        }
        
        // Check permissions
        match self.check_permissions() {
            Ok(true) => {
                result.permissions_ok = true;
                debug!("Permissions check passed");
            }
            Ok(false) => {
                result.errors.push("Insufficient permissions".to_string());
            }
            Err(e) => {
                result.errors.push(format!("Permissions check failed: {}", e));
            }
        }
        
        if !result.errors.is_empty() {
            error!("Preflight checks failed: {:?}", result.errors);
            return Err(OperationsError::PreflightFailed(
                result.errors.join("; ")
            ));
        }
        
        Ok(result)
    }
    
    /// Check OS compatibility
    fn check_os(&self) -> Result<bool, OperationsError> {
        // Check if running on Linux
        #[cfg(target_os = "linux")]
        {
            // Check for systemd
            let systemd_exists = Path::new("/usr/lib/systemd/systemd").exists() ||
                                Path::new("/lib/systemd/systemd").exists();
            
            if !systemd_exists {
                return Ok(false);
            }
            
            // Check kernel version (should be >= 4.0)
            let uname_output = std::process::Command::new("uname")
                .arg("-r")
                .output()
                .map_err(|e| OperationsError::IoError(e))?;
            
            let kernel_version = String::from_utf8_lossy(&uname_output.stdout);
            // Simple check - in production, parse version properly
            if kernel_version.contains("linux") {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(false)
        }
    }
    
    /// Check available disk space
    fn check_disk_space(&self) -> Result<bool, OperationsError> {
        let path = Path::new(&self.project_root);
        
        let disks = Disks::new_with_refreshed_list();
        
        for disk in disks.list() {
            let mount_point = disk.mount_point();
            if path.starts_with(mount_point) {
                let available_gb = disk.available_space() / (1024 * 1024 * 1024);
                debug!("Available disk space: {} GB", available_gb);
                return Ok(available_gb >= self.min_disk_gb);
            }
        }
        
        // If we can't find the disk, assume insufficient
        warn!("Could not determine disk space for {}", self.project_root);
        Ok(false)
    }
    
    /// Check time synchronization
    fn check_time_sync(&self) -> Result<bool, OperationsError> {
        // Check if systemd-timesyncd or chronyd is running
        let systemd_timesync = std::process::Command::new("systemctl")
            .arg("is-active")
            .arg("systemd-timesyncd")
            .output();
        
        let chronyd = std::process::Command::new("systemctl")
            .arg("is-active")
            .arg("chronyd")
            .output();
        
        // If either is active, time is likely synchronized
        if let Ok(output) = systemd_timesync {
            if output.status.success() {
                return Ok(true);
            }
        }
        
        if let Ok(output) = chronyd {
            if output.status.success() {
                return Ok(true);
            }
        }
        
        // Check if NTP is configured
        let ntp_check = std::process::Command::new("timedatectl")
            .arg("status")
            .output();
        
        if let Ok(output) = ntp_check {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.contains("synchronized") || status.contains("NTP service: active") {
                return Ok(true);
            }
        }
        
        // If we can't determine, warn but don't fail
        warn!("Could not verify time synchronization");
        Ok(false)
    }
    
    /// Check permissions
    fn check_permissions(&self) -> Result<bool, OperationsError> {
        let path = Path::new(&self.project_root);
        
        // Check if we can write to the project root
        if !path.exists() {
            // Try to create it
            std::fs::create_dir_all(path)
                .map_err(|e| OperationsError::IoError(e))?;
        }
        
        // Check write permission
        let test_file = path.join(".write_test");
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                std::fs::remove_file(&test_file).ok();
                Ok(true)
            }
            Err(e) => {
                Err(OperationsError::PermissionDenied(format!("Cannot write to {}: {}", self.project_root, e)))
            }
        }
    }
}

