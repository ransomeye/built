// Path and File Name : /home/ransomeye/rebuild/qa/validation/src/contract_integrity.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Contract Integrity validation - envelope schema validation, version compatibility, fail-closed enforcement

use serde::{Deserialize, Serialize};
use tracing::{info, error, warn, debug};
use chrono::{DateTime, Utc};
use crate::errors::{ValidationError, ValidationResult};

/// Event envelope structure (Phase 4 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub producer_id: String,
    pub component_type: String,
    pub schema_version: u32,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: u64,
    pub signature: String,
    pub integrity_hash: String,
    pub nonce: String,
    pub event_data: String,
}

/// Directive envelope structure (Phase 6 → Phase 7 format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectiveEnvelope {
    pub directive_id: String,
    pub policy_id: String,
    pub policy_version: String,
    pub signature: String,
    pub signature_hash: String,
    pub issued_at: DateTime<Utc>,
    pub ttl_seconds: u64,
    pub nonce: String,
    pub target_scope: serde_json::Value,
    pub action: String,
    pub preconditions_hash: String,
    pub audit_receipt: serde_json::Value,
    pub allowed_actions: Vec<String>,
    pub required_approvals: Vec<String>,
    pub evidence_reference: String,
    pub kill_chain_stage: String,
    pub severity: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractIntegrityResult {
    pub envelope_schema_valid: bool,
    pub version_compatibility_valid: bool,
    pub fail_closed_verified: bool,
    pub violations: Vec<String>,
    pub test_cases: Vec<TestCaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub details: String,
    pub evidence: Option<String>,
}

pub struct ContractIntegrityValidator {
    current_schema_version: u32,
    supported_versions: Vec<u32>,
}

impl ContractIntegrityValidator {
    pub fn new() -> Self {
        Self {
            current_schema_version: 1,
            supported_versions: vec![1],
        }
    }
    
    /// Validate event envelope schema at Phase 4 boundary
    pub fn validate_event_envelope(&self, envelope: &EventEnvelope) -> ValidationResult<()> {
        debug!("Validating event envelope from producer {}", envelope.producer_id);
        
        // Check required fields
        if envelope.producer_id.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Producer ID is required".to_string()
            ));
        }
        
        if envelope.component_type.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Component type is required".to_string()
            ));
        }
        
        if envelope.signature.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Signature is required".to_string()
            ));
        }
        
        if envelope.integrity_hash.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Integrity hash is required".to_string()
            ));
        }
        
        if envelope.nonce.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Nonce is required".to_string()
            ));
        }
        
        if envelope.event_data.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Event data is required".to_string()
            ));
        }
        
        // Validate component type
        let valid_component_types = vec!["dpi_probe", "linux_agent", "windows_agent"];
        if !valid_component_types.contains(&envelope.component_type.as_str()) {
            return Err(ValidationError::ContractIntegrity(
                format!("Invalid component type: {}", envelope.component_type)
            ));
        }
        
        debug!("Event envelope structure validation passed");
        Ok(())
    }
    
    /// Validate version compatibility (fail-closed on mismatch)
    pub fn validate_version_compatibility(&self, schema_version: u32) -> ValidationResult<()> {
        if !self.supported_versions.contains(&schema_version) {
            error!("Incompatible schema version: {} (supported: {:?})", 
                   schema_version, self.supported_versions);
            return Err(ValidationError::ContractIntegrity(
                format!("Incompatible schema version: {} (supported: {:?})",
                       schema_version, self.supported_versions)
            ));
        }
        
        debug!("Version compatibility check passed for version {}", schema_version);
        Ok(())
    }
    
    /// Validate directive envelope schema at Phase 6 → Phase 7 boundary
    pub fn validate_directive_envelope(&self, directive: &DirectiveEnvelope) -> ValidationResult<()> {
        debug!("Validating directive envelope {}", directive.directive_id);
        
        // Check required fields
        if directive.directive_id.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Directive ID is required".to_string()
            ));
        }
        
        if directive.policy_id.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Policy ID is required".to_string()
            ));
        }
        
        if directive.policy_version.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Policy version is required".to_string()
            ));
        }
        
        if directive.signature.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Signature is required".to_string()
            ));
        }
        
        if directive.signature_hash.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Signature hash is required".to_string()
            ));
        }
        
        if directive.nonce.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Nonce is required".to_string()
            ));
        }
        
        if directive.action.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Action is required".to_string()
            ));
        }
        
        if directive.preconditions_hash.is_empty() {
            return Err(ValidationError::ContractIntegrity(
                "Preconditions hash is required".to_string()
            ));
        }
        
        // Validate TTL
        if directive.ttl_seconds == 0 {
            return Err(ValidationError::ContractIntegrity(
                "TTL must be greater than zero".to_string()
            ));
        }
        
        // Check expiry (fail-closed on expired directives)
        if directive.is_expired() {
            warn!("Directive {} has expired", directive.directive_id);
            return Err(ValidationError::ContractIntegrity(
                format!("Directive {} has expired", directive.directive_id)
            ));
        }
        
        debug!("Directive envelope structure validation passed");
        Ok(())
    }
    
    /// Run comprehensive contract integrity tests
    pub async fn run_validation_suite(&self) -> ValidationResult<ContractIntegrityResult> {
        info!("Starting contract integrity validation suite");
        
        let mut result = ContractIntegrityResult {
            envelope_schema_valid: true,
            version_compatibility_valid: true,
            fail_closed_verified: true,
            violations: Vec::new(),
            test_cases: Vec::new(),
        };
        
        // Test 1: Valid event envelope should pass
        let valid_envelope = EventEnvelope {
            producer_id: "test_producer".to_string(),
            component_type: "dpi_probe".to_string(),
            schema_version: 1,
            timestamp: Utc::now(),
            sequence_number: 1,
            signature: "test_sig".to_string(),
            integrity_hash: "test_hash".to_string(),
            nonce: "test_nonce".to_string(),
            event_data: r#"{"test": "data"}"#.to_string(),
        };
        
        match self.validate_event_envelope(&valid_envelope) {
            Ok(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Valid event envelope passes".to_string(),
                    passed: true,
                    details: "Valid envelope structure accepted".to_string(),
                    evidence: None,
                });
            }
            Err(e) => {
                result.violations.push(format!("Valid envelope rejected: {}", e));
                result.envelope_schema_valid = false;
                result.test_cases.push(TestCaseResult {
                    name: "Valid event envelope passes".to_string(),
                    passed: false,
                    details: format!("Failed: {}", e),
                    evidence: None,
                });
            }
        }
        
        // Test 2: Invalid event envelope (missing fields) should fail
        let mut invalid_envelope = valid_envelope.clone();
        invalid_envelope.producer_id = String::new();
        
        match self.validate_event_envelope(&invalid_envelope) {
            Ok(_) => {
                result.violations.push("Invalid envelope (missing producer_id) was accepted".to_string());
                result.envelope_schema_valid = false;
                result.fail_closed_verified = false;
                result.test_cases.push(TestCaseResult {
                    name: "Invalid event envelope fails (fail-closed)".to_string(),
                    passed: false,
                    details: "Missing producer_id should have been rejected".to_string(),
                    evidence: None,
                });
            }
            Err(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Invalid event envelope fails (fail-closed)".to_string(),
                    passed: true,
                    details: "Missing producer_id correctly rejected".to_string(),
                    evidence: None,
                });
            }
        }
        
        // Test 3: Version compatibility - supported version should pass
        match self.validate_version_compatibility(1) {
            Ok(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Supported version compatibility".to_string(),
                    passed: true,
                    details: "Version 1 correctly accepted".to_string(),
                    evidence: None,
                });
            }
            Err(e) => {
                result.violations.push(format!("Supported version rejected: {}", e));
                result.version_compatibility_valid = false;
                result.fail_closed_verified = false;
                result.test_cases.push(TestCaseResult {
                    name: "Supported version compatibility".to_string(),
                    passed: false,
                    details: format!("Failed: {}", e),
                    evidence: None,
                });
            }
        }
        
        // Test 4: Version compatibility - unsupported version should fail
        match self.validate_version_compatibility(999) {
            Ok(_) => {
                result.violations.push("Unsupported version 999 was accepted".to_string());
                result.version_compatibility_valid = false;
                result.fail_closed_verified = false;
                result.test_cases.push(TestCaseResult {
                    name: "Unsupported version compatibility (fail-closed)".to_string(),
                    passed: false,
                    details: "Unsupported version should have been rejected".to_string(),
                    evidence: None,
                });
            }
            Err(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Unsupported version compatibility (fail-closed)".to_string(),
                    passed: true,
                    details: "Unsupported version correctly rejected".to_string(),
                    evidence: None,
                });
            }
        }
        
        // Test 5: Valid directive envelope should pass
        let valid_directive = DirectiveEnvelope {
            directive_id: "test_directive_123".to_string(),
            policy_id: "test_policy".to_string(),
            policy_version: "1.0.0".to_string(),
            signature: "test_sig".to_string(),
            signature_hash: "test_hash".to_string(),
            issued_at: Utc::now(),
            ttl_seconds: 3600,
            nonce: "test_nonce".to_string(),
            target_scope: serde_json::json!({}),
            action: "test_action".to_string(),
            preconditions_hash: "test_precond".to_string(),
            audit_receipt: serde_json::json!({}),
            allowed_actions: vec![],
            required_approvals: vec![],
            evidence_reference: "test_ref".to_string(),
            kill_chain_stage: "test_stage".to_string(),
            severity: "high".to_string(),
            reasoning: "test reasoning".to_string(),
        };
        
        match self.validate_directive_envelope(&valid_directive) {
            Ok(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Valid directive envelope passes".to_string(),
                    passed: true,
                    details: "Valid directive structure accepted".to_string(),
                    evidence: None,
                });
            }
            Err(e) => {
                result.violations.push(format!("Valid directive rejected: {}", e));
                result.envelope_schema_valid = false;
                result.test_cases.push(TestCaseResult {
                    name: "Valid directive envelope passes".to_string(),
                    passed: false,
                    details: format!("Failed: {}", e),
                    evidence: None,
                });
            }
        }
        
        // Test 6: Expired directive should fail
        let mut expired_directive = valid_directive.clone();
        expired_directive.issued_at = Utc::now() - chrono::Duration::hours(2);
        expired_directive.ttl_seconds = 3600; // Expired 1 hour ago
        
        match self.validate_directive_envelope(&expired_directive) {
            Ok(_) => {
                result.violations.push("Expired directive was accepted".to_string());
                result.fail_closed_verified = false;
                result.test_cases.push(TestCaseResult {
                    name: "Expired directive fails (fail-closed)".to_string(),
                    passed: false,
                    details: "Expired directive should have been rejected".to_string(),
                    evidence: None,
                });
            }
            Err(_) => {
                result.test_cases.push(TestCaseResult {
                    name: "Expired directive fails (fail-closed)".to_string(),
                    passed: true,
                    details: "Expired directive correctly rejected".to_string(),
                    evidence: None,
                });
            }
        }
        
        info!("Contract integrity validation suite completed: {} violations", result.violations.len());
        Ok(result)
    }
}

impl DirectiveEnvelope {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiry = self.issued_at + chrono::Duration::seconds(self.ttl_seconds as i64);
        now > expiry
    }
}

