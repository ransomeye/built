// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/lib.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: RansomEye Guardrails Enforcement Engine library root - Phase 0 root-of-trust enforcement layer

pub mod errors;
pub mod loader;
pub mod verifier;
pub mod enforcer;
pub mod ci_validator;
pub mod audit;
pub mod spec;

pub use errors::{GuardrailError, GuardrailResult};
pub use loader::GuardrailLoader;
pub use verifier::GuardrailVerifier;
pub use enforcer::{GuardrailEnforcer, EnforcementContext};
pub use ci_validator::CIValidator;
pub use audit::AuditLogger;

/// Single entry point for guardrail enforcement
/// 
/// This function enforces all guardrails and exits immediately on violation.
/// It is called by:
/// - Installer (before installation steps)
/// - Systemd services (ExecStartPre)
/// - CI pipelines
/// - Runtime checks
pub fn enforce_or_exit(context: EnforcementContext) -> ! {
    use std::process;
    
    match GuardrailEnforcer::new() {
        Ok(enforcer) => {
            match enforcer.enforce(context) {
                Ok(_) => {
                    // Success - exit normally (but this function never returns normally)
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("GUARDRAIL VIOLATION: {}", e);
                    eprintln!("System will not start. This is fail-closed behavior.");
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("GUARDRAIL INITIALIZATION FAILED: {}", e);
            eprintln!("System will not start. This is fail-closed behavior.");
            process::exit(1);
        }
    }
}

