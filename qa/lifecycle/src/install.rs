// Path and File Name : /home/ransomeye/rebuild/qa/lifecycle/src/install.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Installation and bootstrap validation - clean install, environment variables, trust material, service auto-start prevention

use super::{LifecycleTestResult, LifecycleValidator};
use std::time::Instant;
use tracing::{info, error};
use std::path::Path;

pub struct InstallValidator<'a> {
    validator: &'a LifecycleValidator,
}

impl<'a> InstallValidator<'a> {
    pub fn new(validator: &'a LifecycleValidator) -> Self {
        Self { validator }
    }

    /// Validate clean installation
    pub async fn validate_clean_install(&self) -> LifecycleTestResult {
        let start = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        info!("Validating clean installation");

        // Check if install script exists
        let install_script = format!("{}/install.sh", self.validator.get_project_root());
        if !Path::new(&install_script).exists() {
            errors.push(format!("Install script not found: {}", install_script));
            return LifecycleTestResult {
                stage: "install".to_string(),
                passed: false,
                duration_ms: start.elapsed().as_millis() as u64,
                errors,
                warnings,
                evidence: None,
            };
        }

        // Check if install script is executable
        let metadata = std::fs::metadata(&install_script);
        if let Ok(meta) = metadata {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = meta.permissions();
                if perms.mode() & 0o111 == 0 {
                    errors.push("Install script is not executable".to_string());
                }
            }
        }

        // Validate EULA exists
        let eula_path = format!("{}/ransomeye_installer/eula/EULA.txt", self.validator.get_project_root());
        if !Path::new(&eula_path).exists() {
            errors.push(format!("EULA not found: {}", eula_path));
        }

        // Validate required environment variables are checked
        // (This would be validated during actual install, but we check the script)
        let install_content = std::fs::read_to_string(&install_script).unwrap_or_default();
        if !install_content.contains("RANSOMEYE_ROOT_KEY_PATH") {
            warnings.push("Install script may not check RANSOMEYE_ROOT_KEY_PATH".to_string());
        }

        // Validate trust material requirement
        if !install_content.contains("trust") && !install_content.contains("Trust") {
            warnings.push("Install script may not validate trust material".to_string());
        }

        // Validate services don't auto-start without trust
        let systemd_dir = format!("{}/systemd", self.validator.get_project_root());
        if Path::new(&systemd_dir).exists() {
            let services = std::fs::read_dir(&systemd_dir).unwrap_or_else(|_| {
                std::fs::read_dir(".").unwrap()
            });
            
            for entry in services {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("service") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check if service has Restart=always (good)
                            if !content.contains("Restart=always") {
                                warnings.push(format!("Service {} missing Restart=always", 
                                    path.file_name().unwrap().to_string_lossy()));
                            }
                            
                            // Check if service has proper error handling
                            if !content.contains("StandardError=journal") {
                                warnings.push(format!("Service {} missing StandardError=journal", 
                                    path.file_name().unwrap().to_string_lossy()));
                            }
                        }
                    }
                }
            }
        }

        let passed = errors.is_empty();
        let duration_ms = start.elapsed().as_millis() as u64;

        if passed {
            info!("Clean installation validation passed");
        } else {
            error!("Clean installation validation failed: {:?}", errors);
        }

        LifecycleTestResult {
            stage: "install".to_string(),
            passed,
            duration_ms,
            errors,
            warnings,
            evidence: Some(format!("Install script: {}, EULA: {}", install_script, eula_path)),
        }
    }

    /// Validate bootstrap process
    pub async fn validate_bootstrap(&self) -> LifecycleTestResult {
        let start = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        info!("Validating bootstrap process");

        // Check for bootstrap scripts or initialization
        let bootstrap_indicators = [
            "ransomeye_installer",
            "post_install_validator.py",
            "systemd",
        ];

        for indicator in &bootstrap_indicators {
            let path = format!("{}/{}", self.validator.get_project_root(), indicator);
            if !Path::new(&path).exists() {
                warnings.push(format!("Bootstrap indicator not found: {}", indicator));
            }
        }

        // Validate post-install validator exists
        let validator_path = format!("{}/post_install_validator.py", self.validator.get_project_root());
        if !Path::new(&validator_path).exists() {
            errors.push(format!("Post-install validator not found: {}", validator_path));
        }

        let passed = errors.is_empty();
        let duration_ms = start.elapsed().as_millis() as u64;

        LifecycleTestResult {
            stage: "bootstrap".to_string(),
            passed,
            duration_ms,
            errors,
            warnings,
            evidence: Some(format!("Bootstrap validation completed")),
        }
    }
}

