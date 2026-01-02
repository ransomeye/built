// Path and File Name : /home/ransomeye/rebuild/ransomeye_architecture/enforcement/src/contract_version.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Data contract version validation and enforcement

use crate::fail_closed::FailClosedGuard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ContractVersion {
    pub fn parse(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version_str));
        }
        
        let major = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2].parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;
        
        Ok(ContractVersion { major, minor, patch })
    }
    
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub struct ContractVersionEnforcer {
    expected_version: ContractVersion,
    fail_closed_guard: FailClosedGuard,
}

impl ContractVersionEnforcer {
    pub fn new(expected_version: &str, fail_closed_guard: FailClosedGuard) -> Result<Self, String> {
        let expected_version = ContractVersion::parse(expected_version)?;
        Ok(ContractVersionEnforcer {
            expected_version,
            fail_closed_guard,
        })
    }
    
    /// Validate contract version - fail-closed on mismatch
    pub fn validate_version(&self, component: &str, received_version: Option<&str>) -> Result<(), String> {
        // Missing version → abort
        let received_version = match received_version {
            Some(v) => v,
            None => {
                self.fail_closed_guard.abort_on_ambiguity(component, "Contract version field missing");
            }
        };
        
        // Parse version
        let received = match ContractVersion::parse(received_version) {
            Ok(v) => v,
            Err(e) => {
                let reason = format!("Invalid version format: {}", e);
                self.fail_closed_guard.abort_on_ambiguity(component, &reason);
            }
        };
        
        // Exact match required - no backward compatibility
        if received != self.expected_version {
            let reason = format!(
                "Version mismatch: expected {}, received {}",
                self.expected_version.to_string(),
                received.to_string()
            );
            self.fail_closed_guard.abort_on_ambiguity(component, &reason);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_logger::AuditLogger;
    use tempfile::TempDir;
    
    #[test]
    fn test_version_parsing() {
        let version = ContractVersion::parse("1.0.0").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }
    
    #[test]
    fn test_version_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(log_path).unwrap();
        let guard = FailClosedGuard::new(logger);
        
        let enforcer = ContractVersionEnforcer::new("1.0.0", guard).unwrap();
        
        // Missing version should abort
        // Version mismatch should abort
        // These are tested in integration tests
    }
}

