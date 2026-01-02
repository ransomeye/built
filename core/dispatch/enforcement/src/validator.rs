// Path and File Name : /home/ransomeye/rebuild/ransomeye_enforcement/src/validator.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Decision validator - validates decision structure and completeness

use serde_json::Value;
use tracing::{warn, debug};
use crate::errors::EnforcementError;
use crate::security::verification::DecisionVerifier;
use crate::security::revocation::RevocationChecker;

pub struct DecisionValidator {
    verifier: DecisionVerifier,
    revocation_checker: RevocationChecker,
}

impl DecisionValidator {
    pub fn new(verifier: DecisionVerifier, revocation_checker: RevocationChecker) -> Self {
        Self {
            verifier,
            revocation_checker,
        }
    }
    
    /// Validate decision structure and integrity
    pub fn validate(&self, decision_json: &str) -> Result<(), EnforcementError> {
        // Parse decision
        let decision: Value = serde_json::from_str(decision_json)
            .map_err(|e| EnforcementError::InvalidFormat(format!("Invalid JSON: {}", e)))?;
        
        // Check required fields
        self.check_required_fields(&decision)?;
        
        // Verify signature
        self.verifier.verify_decision(decision_json)?;
        
        // Check revocation
        let decision_id = decision.get("decision_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnforcementError::InvalidFormat("Missing decision_id".to_string()))?;
        
        self.revocation_checker.check_decision(decision_id)?;
        
        debug!("Decision validation successful: {}", decision_id);
        Ok(())
    }
    
    fn check_required_fields(&self, decision: &Value) -> Result<(), EnforcementError> {
        let required_fields = vec![
            "decision_id",
            "created_at",
            "alert_id",
            "policy_id",
            "policy_version",
            "decision",
            "allowed_actions",
            "required_approvals",
            "evidence_reference",
            "kill_chain_stage",
            "severity",
            "reasoning",
            "policy_signature",
            "decision_hash",
        ];
        
        for field in required_fields {
            if !decision.get(field).is_some() {
                return Err(EnforcementError::InvalidFormat(
                    format!("Missing required field: {}", field)
                ));
            }
        }
        
        Ok(())
    }
}

