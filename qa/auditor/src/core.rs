// Path and File Name : /home/ransomeye/rebuild/ransomeye_validation/src/core.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Core validation domain types - Severity, Finding, and ValidationResult for fail-closed validation semantics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub suite: String,
    pub description: String,
    pub severity: Severity,
}

#[derive(Debug)]
pub enum ValidationResult {
    Pass(Vec<Finding>),
    Hold(Vec<Finding>),
    Fail(Vec<Finding>),
}

impl ValidationResult {
    /// Determine ValidationResult from findings according to fail-closed semantics:
    /// - Any High or Critical → Fail
    /// - Any Medium → Hold
    /// - Only Low/Info → Pass
    pub fn from_findings(findings: Vec<Finding>) -> Self {
        let has_critical_or_high = findings.iter().any(|f| {
            matches!(f.severity, Severity::Critical | Severity::High)
        });
        
        let has_medium = findings.iter().any(|f| {
            matches!(f.severity, Severity::Medium)
        });
        
        if has_critical_or_high {
            ValidationResult::Fail(findings)
        } else if has_medium {
            ValidationResult::Hold(findings)
        } else {
            ValidationResult::Pass(findings)
        }
    }
    
    /// Check if this result represents a failure
    pub fn is_fail(&self) -> bool {
        matches!(self, ValidationResult::Fail(_))
    }
    
    /// Get all findings
    pub fn findings(&self) -> &[Finding] {
        match self {
            ValidationResult::Pass(f) => f,
            ValidationResult::Hold(f) => f,
            ValidationResult::Fail(f) => f,
        }
    }
    
    /// Get all findings with High or Critical severity
    pub fn critical_findings(&self) -> Vec<&Finding> {
        self.findings().iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .collect()
    }
}

