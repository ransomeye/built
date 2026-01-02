// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/fail_closed.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Fail-closed guard that aborts on any ambiguity or undefined condition

use crate::audit_logger::AuditLogger;
use std::process;

#[derive(Clone)]
pub struct FailClosedGuard {
    audit_logger: AuditLogger,
}

impl FailClosedGuard {
    pub fn new(audit_logger: AuditLogger) -> Self {
        FailClosedGuard { audit_logger }
    }
    
    /// Abort on ambiguity - any undefined or ambiguous condition must result in abort
    pub fn abort_on_ambiguity(&self, component: &str, reason: &str) -> ! {
        let _ = self.audit_logger.log_fail_closed(component, reason);
        eprintln!("FAIL-CLOSED: {} - {}", component, reason);
        process::abort();
    }
    
    /// Abort on undefined flow - any flow not explicitly allowed is forbidden
    pub fn abort_on_undefined_flow(&self, source: &str, target: &str) -> ! {
        let reason = format!("Undefined flow from {} to {}", source, target);
        self.abort_on_ambiguity(source, &reason);
    }
    
    /// Abort on identity mismatch - any identity verification failure must abort
    pub fn abort_on_identity_mismatch(&self, component: &str, reason: &str) -> ! {
        let reason = format!("Identity mismatch: {}", reason);
        self.abort_on_ambiguity(component, &reason);
    }
    
    /// Verify condition - if false, abort
    pub fn verify_or_abort(&self, component: &str, condition: bool, reason: &str) {
        if !condition {
            self.abort_on_ambiguity(component, reason);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    #[should_panic(expected = "abort")]
    fn test_fail_closed_abort() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(log_path).unwrap();
        let guard = FailClosedGuard::new(logger);
        
        guard.abort_on_ambiguity("test_component", "test reason");
    }
}

