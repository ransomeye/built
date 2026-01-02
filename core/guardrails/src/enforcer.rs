// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/enforcer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime guardrail enforcer with fail-closed behavior

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use crate::errors::{GuardrailError, GuardrailResult};
use crate::loader::GuardrailLoader;
use crate::verifier::GuardrailVerifier;
use crate::spec::GuardrailSpec;
use crate::audit::AuditLogger;

// Make spec accessible for CI validator
impl GuardrailEnforcer {
    pub fn spec(&self) -> &GuardrailSpec {
        &self.spec
    }
}

#[derive(Debug, Clone)]
pub enum EnforcementContext {
    Installer,
    ServiceStart { service_name: String },
    ModelLoad { model_path: String },
    PolicyLoad { policy_path: String },
    Runtime,
    CI,
}

pub struct GuardrailEnforcer {
    spec: GuardrailSpec,
    audit_logger: AuditLogger,
}

impl GuardrailEnforcer {
    pub fn new() -> GuardrailResult<Self> {
        let loader = GuardrailLoader::default();
        let spec = GuardrailVerifier::verify_and_load(&loader)?;
        let audit_logger = AuditLogger::default();

        Ok(Self {
            spec,
            audit_logger,
        })
    }

    /// Enforce all guardrails based on context
    pub fn enforce(&self, context: EnforcementContext) -> GuardrailResult<()> {
        // Always verify spec signature first
        let verifier = GuardrailVerifier::from_spec(&self.spec);
        verifier.verify(&self.spec)?;

        // Context-specific enforcement
        match context {
            EnforcementContext::Installer => {
                self.enforce_installer()?;
            }
            EnforcementContext::ServiceStart { ref service_name } => {
                self.enforce_service_start(service_name)?;
            }
            EnforcementContext::ModelLoad { ref model_path } => {
                self.enforce_model_load(model_path)?;
            }
            EnforcementContext::PolicyLoad { ref policy_path } => {
                self.enforce_policy_load(policy_path)?;
            }
            EnforcementContext::Runtime => {
                self.enforce_runtime()?;
            }
            EnforcementContext::CI => {
                self.enforce_ci()?;
            }
        }

        // Common enforcement checks
        self.enforce_common()?;

        Ok(())
    }

    fn enforce_installer(&self) -> GuardrailResult<()> {
        // Check for phantom modules
        self.check_phantom_modules()?;
        
        // Check for hardcoded configs
        self.check_hardcoded_configs()?;
        
        // Check systemd placement
        self.check_systemd_placement()?;
        
        Ok(())
    }

    fn enforce_service_start(&self, service_name: &str) -> GuardrailResult<()> {
        // Verify service file is in correct location
        let service_path = PathBuf::from("/home/ransomeye/rebuild/systemd")
            .join(format!("{}.service", service_name));
        
        if !service_path.exists() {
            // Check if it's a standalone exception
            if !self.is_standalone_exception(&service_path) {
                return Err(GuardrailError::SystemdMisplacement(
                    service_path.display().to_string(),
                ));
            }
        }

        Ok(())
    }

    fn enforce_model_load(&self, model_path: &str) -> GuardrailResult<()> {
        let path = Path::new(model_path);
        
        // Check model format
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| GuardrailError::MissingModelMetadata(model_path.to_string()))?;
        
        if !self.spec.model_requirements.allowed_formats.iter()
            .any(|f| f.trim_start_matches('.') == ext) {
            return Err(GuardrailError::MissingModelMetadata(
                format!("Model format .{} not allowed", ext)
            ));
        }

        // Check for SHAP file
        if self.spec.model_requirements.required_shap {
            let shap_path = path.with_extension("shap.json");
            if !shap_path.exists() && self.spec.model_requirements.reject_if_missing_shap {
                self.log_violation(
                    "MissingShap",
                    None,
                    None,
                    Some(model_path),
                    "Model missing SHAP explainability file",
                )?;
                return Err(GuardrailError::MissingShap(model_path.to_string()));
            }
        }

        // Check for metadata
        let metadata_path = path.with_extension("metadata.json");
        if !metadata_path.exists() && self.spec.model_requirements.reject_if_missing_metadata {
            self.log_violation(
                "MissingModelMetadata",
                None,
                None,
                Some(model_path),
                "Model missing metadata.json",
            )?;
            return Err(GuardrailError::MissingModelMetadata(model_path.to_string()));
        }

        Ok(())
    }

    fn enforce_policy_load(&self, policy_path: &str) -> GuardrailResult<()> {
        // Check if policy is signed (if required)
        if self.spec.crypto_requirements.required_signing_for
            .contains(&"policies".to_string()) {
            let sig_path = format!("{}.sig", policy_path);
            if !Path::new(&sig_path).exists() {
                self.log_violation(
                    "UnsignedArtifact",
                    None,
                    None,
                    Some(policy_path),
                    "Policy file not signed",
                )?;
                return Err(GuardrailError::UnsignedArtifact(policy_path.to_string()));
            }
        }

        Ok(())
    }

    fn enforce_runtime(&self) -> GuardrailResult<()> {
        // Runtime checks: ENV vars, etc.
        for env_var in &self.spec.env_only_rules.required_env_vars {
            if std::env::var(env_var).is_err() {
                return Err(GuardrailError::MissingEnvVar(env_var.clone()));
            }
        }

        Ok(())
    }

    fn enforce_ci(&self) -> GuardrailResult<()> {
        // CI-specific checks (same as installer but more thorough)
        self.enforce_installer()?;
        self.check_file_headers()?;
        Ok(())
    }

    fn enforce_common(&self) -> GuardrailResult<()> {
        // Common checks that apply to all contexts
        self.check_phantom_modules()?;
        self.check_forbidden_modules()?;
        Ok(())
    }

    fn check_phantom_modules(&self) -> GuardrailResult<()> {
        let project_root = Path::new("/home/ransomeye/rebuild");
        
        for entry in WalkDir::new(project_root)
            .max_depth(2)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && 
                name != "target" && 
                name != "node_modules" &&
                name != "__pycache__"
            }) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let dir_name = entry.file_name().to_string_lossy();
                
                // Check if it's a forbidden module
                if self.spec.forbidden_modules.iter()
                    .any(|f| dir_name.contains(f)) {
                    self.log_violation(
                        "ForbiddenModule",
                        None,
                        Some(&dir_name),
                        Some(&entry.path().display().to_string()),
                        "Forbidden module detected",
                    )?;
                    return Err(GuardrailError::ForbiddenModule(dir_name.to_string()));
                }

                // Check if it looks like a module but isn't in allowed list
                if dir_name.starts_with("ransomeye_") || dir_name.starts_with("core_") {
                    if !self.spec.allowed_modules.iter()
                        .any(|m| dir_name == *m) {
                        // Might be a phantom module
                        self.log_violation(
                            "PhantomModule",
                            None,
                            Some(&dir_name),
                            Some(&entry.path().display().to_string()),
                            "Potential phantom module detected",
                        )?;
                        return Err(GuardrailError::PhantomModule(dir_name.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_forbidden_modules(&self) -> GuardrailResult<()> {
        // Additional check for explicitly forbidden patterns
        let project_root = Path::new("/home/ransomeye/rebuild");
        
        for forbidden in &self.spec.forbidden_modules {
            let _pattern = format!("**/{}", forbidden);
            // Simple check - could be enhanced with glob matching
            for entry in WalkDir::new(project_root)
                .max_depth(3)
                .into_iter() {
                let entry = entry?;
                if entry.file_type().is_dir() {
                    let dir_name = entry.file_name().to_string_lossy();
                    if dir_name.contains(forbidden) {
                        return Err(GuardrailError::ForbiddenModule(dir_name.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_hardcoded_configs(&self) -> GuardrailResult<()> {
        let project_root = Path::new("/home/ransomeye/rebuild");
        
        for pattern in &self.spec.env_only_rules.forbidden_patterns {
            let regex = Regex::new(&pattern.pattern)
                .map_err(|e| GuardrailError::Crypto(format!("Invalid regex: {}", e)))?;

            for entry in WalkDir::new(project_root)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && 
                    name != "target" && 
                    name != "node_modules" &&
                    name != "__pycache__" &&
                    name != ".git"
                }) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let ext = entry.path().extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    
                    // Only check source files
                    if matches!(ext, "py" | "rs" | "yaml" | "yml" | "json" | "sh" | "ts" | "tsx") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            for (line_num, line) in content.lines().enumerate() {
                                if regex.is_match(line) {
                                    // Check exceptions
                                    let mut is_exception = false;
                                    for exception in &pattern.exceptions {
                                        if line.contains(exception) {
                                            is_exception = true;
                                            break;
                                        }
                                    }
                                    
                                    if !is_exception {
                                        self.log_violation(
                                            "HardcodedConfig",
                                            None,
                                            None,
                                            Some(&entry.path().display().to_string()),
                                            &format!("Line {}: {}", line_num + 1, pattern.description),
                                        )?;
                                        return Err(GuardrailError::HardcodedConfig(
                                            entry.path().display().to_string(),
                                            format!("Line {}: {}", line_num + 1, pattern.description),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_systemd_placement(&self) -> GuardrailResult<()> {
        let systemd_dir = Path::new(&self.spec.systemd_requirements.unified_directory);
        let project_root = Path::new("/home/ransomeye/rebuild");

        for entry in WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && 
                name != "target" && 
                name != "node_modules" &&
                name != "__pycache__" &&
                name != ".git"
            }) {
            let entry = entry?;
            if entry.file_type().is_file() && entry.path().extension() == Some("service".as_ref()) {
                let path = entry.path();
                
                // Check if it's in the unified directory
                if !path.starts_with(systemd_dir) {
                    // Check if it's a standalone exception
                    if !self.is_standalone_exception(path) {
                        self.log_violation(
                            "SystemdMisplacement",
                            None,
                            None,
                            Some(&path.display().to_string()),
                            "Systemd service file not in unified directory",
                        )?;
                        return Err(GuardrailError::SystemdMisplacement(
                            path.display().to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_file_headers(&self) -> GuardrailResult<()> {
        let project_root = Path::new("/home/ransomeye/rebuild");
        
        for entry in WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && 
                name != "target" && 
                name != "node_modules" &&
                name != "__pycache__" &&
                name != ".git"
            }) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let ext = entry.path().extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                
                // Check if file type requires header
                if self.spec.header_requirements.required_files.iter()
                    .any(|f| f.trim_start_matches('.') == ext) {
                    
                    // Check exceptions
                    let path_str = entry.path().display().to_string();
                    let is_exception = self.spec.header_requirements.exceptions.iter()
                        .any(|exc| {
                            // Simple glob matching
                            if exc.contains("**") {
                                let pattern = exc.replace("**", "");
                                path_str.contains(&pattern)
                            } else {
                                path_str.contains(exc)
                            }
                        });
                    
                    if !is_exception {
                        // Check first few lines for header
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            let first_line = content.lines().next().unwrap_or("");
                            if !first_line.contains("Path and File Name") {
                                self.log_violation(
                                    "MissingHeader",
                                    None,
                                    None,
                                    Some(&path_str),
                                    "File missing mandatory header",
                                )?;
                                return Err(GuardrailError::MissingHeader(path_str));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn is_standalone_exception(&self, path: &Path) -> bool {
        for exception in &self.spec.systemd_requirements.standalone_exceptions {
            if path.starts_with(&exception.path) && exception.allowed_systemd {
                return true;
            }
        }
        false
    }

    fn log_violation(
        &self,
        violation_type: &str,
        phase: Option<&str>,
        module: Option<&str>,
        file_path: Option<&str>,
        details: &str,
    ) -> GuardrailResult<()> {
        let spec_hash = self.spec.spec_hash.clone();
        self.audit_logger.log_violation(
            violation_type,
            phase,
            module,
            file_path,
            details,
            &spec_hash,
        )?;
        Ok(())
    }
}

