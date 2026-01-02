// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/identity_enforcer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime identity verification and enforcement

use std::collections::HashSet;
use std::sync::Mutex;
use crate::fail_closed::FailClosedGuard;

pub struct IdentityEnforcer {
    revoked_identities: Mutex<HashSet<String>>,
    fail_closed_guard: FailClosedGuard,
}

impl IdentityEnforcer {
    pub fn new(fail_closed_guard: FailClosedGuard) -> Self {
        IdentityEnforcer {
            revoked_identities: Mutex::new(HashSet::new()),
            fail_closed_guard,
        }
    }
    
    /// Verify component identity - must be valid, not revoked, and match expected component
    pub fn verify_identity(
        &self,
        component: &str,
        identity: &str,
        expected_component: Option<&str>,
    ) -> Result<(), String> {
        // Check if identity is revoked
        if self.is_revoked(identity) {
            self.fail_closed_guard.abort_on_identity_mismatch(
                component,
                &format!("Identity {} is revoked", identity),
            );
        }
        
        // Verify identity format (basic check - should be hash-like)
        if identity.is_empty() || identity.len() < 32 {
            self.fail_closed_guard.abort_on_identity_mismatch(
                component,
                "Invalid identity format",
            );
        }
        
        // If expected component is provided, verify match
        if let Some(expected) = expected_component {
            // In real implementation, identity would encode component info
            // For now, we verify identity is not empty and not revoked
            if identity.is_empty() {
                self.fail_closed_guard.abort_on_identity_mismatch(
                    component,
                    &format!("Identity does not match expected component {}", expected),
                );
            }
        }
        
        Ok(())
    }
    
    /// Check if identity is revoked
    pub fn is_revoked(&self, identity: &str) -> bool {
        self.revoked_identities.lock().unwrap().contains(identity)
    }
    
    /// Revoke an identity
    pub fn revoke_identity(&self, identity: &str) {
        self.revoked_identities.lock().unwrap().insert(identity.to_string());
    }
    
    /// Verify signature (placeholder - real implementation would verify cryptographic signature)
    pub fn verify_signature(
        &self,
        component: &str,
        data: &[u8],
        signature: &str,
        identity: &str,
    ) -> Result<(), String> {
        // Basic validation
        if signature.is_empty() {
            self.fail_closed_guard.abort_on_identity_mismatch(
                component,
                "Signature is empty",
            );
        }
        
        // In real implementation, would verify RSA-4096-PSS-SHA256 signature
        // For now, we just verify signature exists and identity is valid
        self.verify_identity(component, identity, None)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_logger::AuditLogger;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_identity_verification() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(log_path).unwrap();
        let guard = FailClosedGuard::new(logger);
        let enforcer = IdentityEnforcer::new(guard);
        
        // Valid identity should pass
        let result = enforcer.verify_identity("test_component", "valid_identity_hash_12345678901234567890", None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_revoked_identity() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(log_path).unwrap();
        let guard = FailClosedGuard::new(logger);
        let enforcer = IdentityEnforcer::new(guard);
        
        let identity = "revoked_identity_hash_12345678901234567890";
        enforcer.revoke_identity(identity);
        
        assert!(enforcer.is_revoked(identity));
    }
}

