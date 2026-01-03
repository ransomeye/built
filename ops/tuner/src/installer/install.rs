// Path and File Name : /home/ransomeye/rebuild/ransomeye_operations/src/installer/install.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Main installer - orchestrates the complete installation flow

#![cfg(feature = "future-tuner")]

use std::path::Path;
use tracing::{info, error, debug};

use crate::errors::OperationsError;
use crate::installer::preflight::PreflightChecker;
use crate::installer::retention::{RetentionConfigurator, RetentionPolicy};
use crate::installer::crypto::CryptoIdentityManager;
use crate::installer::state::{InstallStateManager, InstallState};
use crate::installer::summary::InstallSummary;

/// Main installer - orchestrates installation flow
pub struct Installer {
    project_root: String,
    eula_path: String,
    state_path: String,
    config_path: String,
    keys_dir: String,
    systemd_dir: String,
}

impl Installer {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            eula_path: format!("{}/ransomeye_operations/eula/EULA.txt", project_root),
            state_path: format!("{}/ransomeye_installer/config/install_state.json", project_root),
            config_path: format!("{}/config/retention.txt", project_root),
            keys_dir: format!("{}/ransomeye_installer/keys", project_root),
            systemd_dir: format!("{}/systemd", project_root),
        }
    }
    
    /// Run complete installation flow
    pub fn install(
        &self,
        eula_accepted: bool,
        retention_policy: Option<RetentionPolicy>,
        engine_version: &str,
    ) -> Result<InstallState, OperationsError> {
        info!("Starting RansomEye installation");
        
        // Step 1: Preflight checks
        info!("[1/7] Running preflight checks...");
        let preflight = PreflightChecker::new(&self.project_root, 10); // 10 GB minimum
        let preflight_result = preflight.check_all()?;
        debug!("Preflight checks passed: {:?}", preflight_result);
        
        // Step 2: Display and verify EULA acceptance
        info!("[2/7] Verifying EULA acceptance...");
        if !eula_accepted {
            return Err(OperationsError::EulaNotAccepted);
        }
        self.verify_eula()?;
        
        // Step 3: Configure retention
        info!("[3/7] Configuring retention policy...");
        let retention_config = RetentionConfigurator::new(&self.config_path);
        let retention_policy = retention_config.configure(retention_policy)?;
        
        // Step 4: Generate cryptographic identity
        info!("[4/7] Generating cryptographic identity...");
        let crypto_manager = CryptoIdentityManager::new(&self.keys_dir);
        let identity = crypto_manager.generate()?;
        
        // Step 5: Create and sign install state
        info!("[5/7] Creating signed install state...");
        let state_manager = InstallStateManager::new(&self.state_path, &self.keys_dir);
        let state = state_manager.create(
            eula_accepted,
            Some(chrono::Utc::now()),
            retention_policy,
            identity,
            engine_version,
        )?;
        
        // Step 6: Generate systemd units (DISABLED)
        info!("[6/7] Generating systemd service units...");
        self.generate_systemd_units()?;
        
        // Step 7: Print summary
        info!("[7/7] Installation complete");
        InstallSummary::print(&state);
        
        Ok(state)
    }
    
    /// Verify EULA file exists
    fn verify_eula(&self) -> Result<(), OperationsError> {
        if !Path::new(&self.eula_path).exists() {
            return Err(OperationsError::PreflightFailed(
                format!("EULA file not found: {}", self.eula_path)
            ));
        }
        Ok(())
    }
    
    /// Generate systemd service units (all DISABLED by default)
    fn generate_systemd_units(&self) -> Result<(), OperationsError> {
        use std::fs;
        
        fs::create_dir_all(&self.systemd_dir)
            .map_err(|e| OperationsError::IoError(e))?;
        
        // Core services that need systemd units
        let services = vec![
            ("ransomeye-core", "RansomEye Core Engine"),
            ("ransomeye-ingestion", "RansomEye Event Ingestion"),
            ("ransomeye-correlation", "RansomEye Threat Correlation"),
            ("ransomeye-policy", "RansomEye Policy Engine"),
            ("ransomeye-enforcement", "RansomEye Enforcement Engine"),
            ("ransomeye-intelligence", "RansomEye Threat Intelligence"),
            ("ransomeye-reporting", "RansomEye Reporting & Forensics"),
        ];
        
        for (service_name, description) in services {
            let unit_content = self.generate_service_unit(service_name, description);
            let unit_path = format!("{}/{}.service", self.systemd_dir, service_name);
            
            fs::write(&unit_path, unit_content)
                .map_err(|e| OperationsError::IoError(e))?;
            
            debug!("Generated systemd unit: {}", unit_path);
        }
        
        Ok(())
    }
    
    /// Generate systemd service unit content
    fn generate_service_unit(&self, service_name: &str, description: &str) -> String {
        // Extract directory name from service name (e.g., "ransomeye-core" -> "core")
        let dir_name = service_name.strip_prefix("ransomeye-").unwrap_or(service_name);
        
        format!(
            "# Path and File Name : {}/{}.service\n\
            # Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU\n\
            # Details of functionality of this file: Systemd service unit for {}\n\n\
            [Unit]\n\
            Description={}\n\
            After=network.target\n\
            Requires=network.target\n\
            ConditionPathExists={}\n\n\
            [Service]\n\
            Type=simple\n\
            # Rootless runtime enforcement - MUST NOT run as root (UID 0)\n\
            User=ransomeye\n\
            Group=ransomeye\n\
            WorkingDirectory={}\n\
            RuntimeDirectory=ransomeye/{}\n\
            StateDirectory=ransomeye/{}\n\
            ExecStart=/usr/bin/ransomeye_operations start {}\n\
            Restart=always\n\
            RestartSec=10\n\
            StandardOutput=journal\n\
            StandardError=journal\n\n\
            # Security hardening - Rootless runtime enforcement\n\
            NoNewPrivileges=true\n\
            PrivateTmp=true\n\
            ProtectSystem=strict\n\
            ProtectHome=true\n\
            ReadWritePaths={} /var/lib/ransomeye/{} /run/ransomeye/{}\n\n\
            # Capability-based privileges (no root required)\n\
            CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_RAW CAP_SYS_NICE\n\
            AmbientCapabilities=\n\
            PrivateUsers=false\n\n\
            [Install]\n\
            WantedBy=multi-user.target\n",
            self.systemd_dir,
            service_name,
            description,
            description,
            self.state_path,
            self.project_root,
            dir_name,
            dir_name,
            service_name,
            self.project_root,
            dir_name,
            dir_name
        )
    }
}

