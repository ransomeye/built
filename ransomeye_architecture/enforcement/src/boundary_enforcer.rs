// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/boundary_enforcer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime enforcement of trust boundaries - blocks forbidden flows and terminates violating processes

use crate::plane_classifier::{Plane, PlaneClassifier};
use crate::audit_logger::AuditLogger;
use crate::identity_enforcer::IdentityEnforcer;
use crate::fail_closed::FailClosedGuard;
use std::process;

pub struct BoundaryEnforcer {
    plane_classifier: PlaneClassifier,
    audit_logger: AuditLogger,
    identity_enforcer: IdentityEnforcer,
    fail_closed_guard: FailClosedGuard,
}

impl BoundaryEnforcer {
    pub fn new(audit_log_path: std::path::PathBuf) -> Result<Self, std::io::Error> {
        let audit_logger = AuditLogger::new(audit_log_path)?;
        let fail_closed_guard = FailClosedGuard::new(audit_logger.clone());
        let identity_enforcer = IdentityEnforcer::new(fail_closed_guard.clone());
        
        Ok(BoundaryEnforcer {
            plane_classifier: PlaneClassifier::new(),
            audit_logger: audit_logger.clone(),
            identity_enforcer,
            fail_closed_guard,
        })
    }
    
    /// Enforce boundary crossing - this is the main enforcement function
    /// Returns Ok(()) if allowed, aborts process if forbidden
    pub fn enforce_boundary_crossing(
        &self,
        source_component: &str,
        target_component: &str,
        source_identity: Option<&str>,
        operation: &str,
    ) -> Result<(), String> {
        // Verify identity if provided
        if let Some(identity) = source_identity {
            if let Err(e) = self.identity_enforcer.verify_identity(
                source_component,
                identity,
                Some(source_component),
            ) {
                return Err(e);
            }
        }
        
        // Classify planes
        let source_plane = self.plane_classifier.classify(source_component);
        let target_plane = self.plane_classifier.classify(target_component);
        
        // If either component is unknown, abort (fail-closed)
        if source_plane.is_none() {
            self.fail_closed_guard.abort_on_ambiguity(
                source_component,
                &format!("Unknown component: {}", source_component),
            );
        }
        
        if target_plane.is_none() {
            self.fail_closed_guard.abort_on_undefined_flow(
                source_component,
                target_component,
            );
        }
        
        let source_plane = source_plane.unwrap();
        let target_plane = target_plane.unwrap();
        
        // Check if flow is forbidden
        if self.is_forbidden_flow(source_plane, target_plane, operation) {
            self.handle_forbidden_flow(
                source_component,
                target_component,
                source_plane,
                target_plane,
                operation,
                source_identity,
            );
        }
        
        Ok(())
    }
    
    /// Check if a flow is forbidden
    fn is_forbidden_flow(&self, source: Plane, target: Plane, operation: &str) -> bool {
        match (source, target) {
            // Forbidden: Intelligence → Control
            (Plane::IntelligencePlane, Plane::ControlPlane) => true,
            
            // Forbidden: Data Plane → Policy Engine (Control Plane)
            (Plane::DataPlane, Plane::ControlPlane) if operation.contains("policy") => true,
            
            // Forbidden: Data Plane → Enforcement
            (Plane::DataPlane, Plane::ControlPlane) if operation.contains("enforcement") => true,
            
            // Forbidden: Intelligence → Enforcement
            (Plane::IntelligencePlane, Plane::ControlPlane) if operation.contains("enforcement") => true,
            
            // Forbidden: Management → Data Plane (direct)
            (Plane::ManagementPlane, Plane::DataPlane) => true,
            
            // Allowed flows
            (Plane::DataPlane, Plane::ControlPlane) => false, // Data → Core (allowed)
            (Plane::ControlPlane, Plane::IntelligencePlane) => false, // Control → AI (read-only, allowed)
            (Plane::IntelligencePlane, Plane::ManagementPlane) => false, // AI → Human (allowed)
            (Plane::ControlPlane, Plane::ManagementPlane) => false, // Control → Reporting (allowed)
            (Plane::ControlPlane, Plane::ControlPlane) => false, // Control → Control (allowed)
            
            // Everything else is undefined - fail-closed
            (_, _) => true,
        }
    }
    
    /// Handle forbidden flow violation - terminate process and audit log
    fn handle_forbidden_flow(
        &self,
        source_component: &str,
        target_component: &str,
        source_plane: Plane,
        target_plane: Plane,
        operation: &str,
        source_identity: Option<&str>,
    ) -> ! {
        let violation_type = format!("FORBIDDEN_FLOW_{:?}_TO_{:?}", source_plane, target_plane);
        let violation_details = format!(
            "Component {} (plane: {:?}) attempted {} to {} (plane: {:?})",
            source_component, source_plane, operation, target_component, target_plane
        );
        
        // Log violation
        let _ = self.audit_logger.log_violation(
            &violation_type,
            source_component,
            Some(target_component),
            &violation_details,
            "PROCESS_TERMINATED",
            source_identity,
        );
        
        // Revoke identity if provided
        if let Some(identity) = source_identity {
            self.identity_enforcer.revoke_identity(identity);
        }
        
        // Terminate process
        eprintln!("BOUNDARY_VIOLATION: {} -> {} ({})", source_component, target_component, violation_details);
        process::abort();
    }
    
    /// Check if AI component can access control plane (always returns false - forbidden)
    pub fn check_ai_to_control(&self, source: &str, target: &str) -> bool {
        if self.plane_classifier.is_intelligence_plane(source) 
            && self.plane_classifier.is_control_plane(target) {
            self.enforce_boundary_crossing(source, target, None, "api_access").unwrap_err();
            false
        } else {
            true
        }
    }
    
    /// Check if data plane can access policy engine (always returns false - forbidden)
    pub fn check_data_to_policy(&self, source: &str, target: &str) -> bool {
        if self.plane_classifier.is_data_plane(source) 
            && self.plane_classifier.is_control_plane(target) {
            // Check if target is policy engine
            if target.contains("alert_engine") || target.contains("policy") {
                self.enforce_boundary_crossing(source, target, None, "policy_access").unwrap_err();
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    #[should_panic(expected = "abort")]
    fn test_ai_to_control_forbidden() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let enforcer = BoundaryEnforcer::new(log_path).unwrap();
        
        // This should abort
        let _ = enforcer.enforce_boundary_crossing(
            "ransomeye_ai_core",
            "ransomeye_alert_engine",
            None,
            "api_call",
        );
    }
    
    #[test]
    fn test_data_to_core_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let enforcer = BoundaryEnforcer::new(log_path).unwrap();
        
        // This should be allowed
        let result = enforcer.enforce_boundary_crossing(
            "ransomeye_dpi_probe",
            "ransomeye_master_core",
            None,
            "telemetry",
        );
        assert!(result.is_ok());
    }
}

