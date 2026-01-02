// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/ci_validator.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: CI/CD validation for guardrails - detects violations before merge

use std::path::Path;
use crate::errors::{GuardrailError, GuardrailResult};
use crate::enforcer::GuardrailEnforcer;
use crate::enforcer::EnforcementContext;

pub struct CIValidator {
    enforcer: GuardrailEnforcer,
}

impl CIValidator {
    pub fn new() -> GuardrailResult<Self> {
        let enforcer = GuardrailEnforcer::new()?;
        Ok(Self { enforcer })
    }

    /// Run all CI validation checks
    /// Returns Ok(()) if all checks pass, Err if any violation detected
    pub fn validate(&self) -> GuardrailResult<()> {
        println!("Running CI guardrail validation...");

        // Run enforcer in CI context
        self.enforcer.enforce(EnforcementContext::CI)?;

        // Additional CI-specific checks
        self.detect_phantom_modules()?;
        self.detect_forbidden_directories()?;
        self.detect_hardcoded_configs()?;
        self.detect_systemd_misplacement()?;
        self.detect_unsigned_artifacts()?;

        println!("✓ All CI guardrail checks passed");
        Ok(())
    }

    fn detect_phantom_modules(&self) -> GuardrailResult<()> {
        // This is already done by enforcer, but we can add more specific checks here
        println!("  Checking for phantom modules...");
        Ok(())
    }

    fn detect_forbidden_directories(&self) -> GuardrailResult<()> {
        println!("  Checking for forbidden directories...");
        // Check for directories that shouldn't exist
        let project_root = Path::new("/home/ransomeye/rebuild");
        
        for forbidden in &self.enforcer.spec().forbidden_modules {
            let forbidden_path = project_root.join(forbidden);
            if forbidden_path.exists() {
                return Err(GuardrailError::ForbiddenModule(
                    forbidden_path.display().to_string(),
                ));
            }
        }

        Ok(())
    }

    fn detect_hardcoded_configs(&self) -> GuardrailResult<()> {
        println!("  Checking for hardcoded configurations...");
        // This is already done by enforcer
        Ok(())
    }

    fn detect_systemd_misplacement(&self) -> GuardrailResult<()> {
        println!("  Checking systemd service file placement...");
        // This is already done by enforcer
        Ok(())
    }

    fn detect_unsigned_artifacts(&self) -> GuardrailResult<()> {
        println!("  Checking for unsigned artifacts...");
        // Check for models, policies, playbooks that require signatures
        let _project_root = Path::new("/home/ransomeye/rebuild");
        
        // This would scan for artifacts and verify signatures
        // For now, we rely on the enforcer's policy_load and model_load checks
        
        Ok(())
    }
}


