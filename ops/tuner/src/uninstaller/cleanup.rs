// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/uninstaller/cleanup.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Cleanup manager - securely removes services, configs, and optionally evidence

use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::errors::OperationsError;
use crate::installer::state::InstallState;

/// Cleanup options
#[derive(Debug, Clone)]
pub struct CleanupOptions {
    pub remove_services: bool,
    pub remove_configs: bool,
    pub remove_evidence: bool,
    pub secure_delete: bool,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            remove_services: true,
            remove_configs: true,
            remove_evidence: false,  // Default: preserve evidence
            secure_delete: false,
        }
    }
}

/// Cleanup manager - handles secure removal of components
pub struct CleanupManager {
    project_root: String,
    systemd_dir: String,
    state_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub removed_services: Vec<String>,
    pub removed_configs: Vec<String>,
    pub evidence_destroyed: bool,
    pub secure_delete_used: bool,
}

impl CleanupManager {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            systemd_dir: format!("{}/systemd", project_root),
            state_path: format!("{}/ransomeye_installer/config/install_state.json", project_root),
        }
    }
    
    /// Perform cleanup based on options
    pub fn cleanup(&self, state: &InstallState, options: &CleanupOptions) -> Result<CleanupLog, OperationsError> {
        info!("Starting cleanup with options: {:?}", options);
        
        let mut log = CleanupLog {
            timestamp: Utc::now(),
            removed_services: Vec::new(),
            removed_configs: Vec::new(),
            evidence_destroyed: options.remove_evidence,
            secure_delete_used: options.secure_delete,
        };
        
        // Remove systemd services
        if options.remove_services {
            self.remove_systemd_services(&mut log)?;
        }
        
        // Remove configs
        if options.remove_configs {
            self.remove_configs(&mut log)?;
        }
        
        // Remove evidence (if requested)
        if options.remove_evidence {
            self.remove_evidence(options.secure_delete)?;
        }
        
        // Remove install state (last, after everything else)
        if Path::new(&self.state_path).exists() {
            fs::remove_file(&self.state_path)
                .map_err(|e| OperationsError::IoError(e))?;
            debug!("Removed install state file");
        }
        
        info!("Cleanup complete: {:?}", log);
        Ok(log)
    }
    
    /// Remove systemd service units
    fn remove_systemd_services(&self, log: &mut CleanupLog) -> Result<(), OperationsError> {
        if !Path::new(&self.systemd_dir).exists() {
            return Ok(());
        }
        
        let services = vec![
            "ransomeye-core.service",
            "ransomeye-ingestion.service",
            "ransomeye-correlation.service",
            "ransomeye-policy.service",
            "ransomeye-enforcement.service",
            "ransomeye-intelligence.service",
            "ransomeye-reporting.service",
        ];
        
        for service in services {
            let service_path = format!("{}/{}", self.systemd_dir, service);
            if Path::new(&service_path).exists() {
                fs::remove_file(&service_path)
                    .map_err(|e| OperationsError::IoError(e))?;
                log.removed_services.push(service.to_string());
                debug!("Removed systemd service: {}", service);
            }
        }
        
        Ok(())
    }
    
    /// Remove configuration files
    fn remove_configs(&self, log: &mut CleanupLog) -> Result<(), OperationsError> {
        let configs = vec![
            format!("{}/config/retention.txt", self.project_root),
            format!("{}/ransomeye_installer/config/install_state.json", self.project_root),
        ];
        
        for config_path in configs {
            if Path::new(&config_path).exists() {
                fs::remove_file(&config_path)
                    .map_err(|e| OperationsError::IoError(e))?;
                log.removed_configs.push(config_path.clone());
                debug!("Removed config: {}", config_path);
            }
        }
        
        Ok(())
    }
    
    /// Remove evidence (with optional secure deletion)
    fn remove_evidence(&self, secure_delete: bool) -> Result<(), OperationsError> {
        // CRITICAL: Only reference modules that EXIST on disk
        // ransomeye_forensic does not exist - forensic functionality is in ransomeye_reporting
        let evidence_dirs = vec![
            format!("{}/ransomeye_reporting", self.project_root),
        ];
        
        for evidence_dir in evidence_dirs {
            if Path::new(&evidence_dir).exists() {
                if secure_delete {
                    self.secure_delete_directory(&evidence_dir)?;
                } else {
                    fs::remove_dir_all(&evidence_dir)
                        .map_err(|e| OperationsError::IoError(e))?;
                }
                debug!("Removed evidence directory: {}", evidence_dir);
            }
        }
        
        Ok(())
    }
    
    /// Secure delete directory (3-pass overwrite)
    fn secure_delete_directory(&self, dir_path: &str) -> Result<(), OperationsError> {
        use walkdir::WalkDir;
        
        // First pass: overwrite with random data
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .open(entry.path()) {
                    use std::io::Write;
                    let size = file.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let random_data: Vec<u8> = (0..size).map(|_| rand::random()).collect();
                    file.write_all(&random_data).ok();
                }
            }
        }
        
        // Second pass: overwrite with zeros
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .open(entry.path()) {
                    use std::io::Write;
                    let size = file.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    file.write_all(&vec![0u8; size as usize]).ok();
                }
            }
        }
        
        // Third pass: overwrite with random data again
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .open(entry.path()) {
                    use std::io::Write;
                    let size = file.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let random_data: Vec<u8> = (0..size).map(|_| rand::random()).collect();
                    file.write_all(&random_data).ok();
                }
            }
        }
        
        // Finally, remove directory
        fs::remove_dir_all(dir_path)
            .map_err(|e| OperationsError::IoError(e))?;
        
        Ok(())
    }
}

