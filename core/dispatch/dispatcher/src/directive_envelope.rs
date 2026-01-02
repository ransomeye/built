// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/protocol/directive_envelope.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Phase 6 → Phase 7 directive envelope protocol - strict validation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// DirectiveEnvelope - Phase 6 output, Phase 7 input
/// 
/// This is the ONLY valid input format for Phase 7 dispatcher.
/// All fields are mandatory and must be validated before any action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectiveEnvelope {
    /// Unique directive identifier (UUIDv7)
    pub directive_id: String,
    
    /// Policy ID from Phase 6
    pub policy_id: String,
    
    /// Policy version from Phase 6
    pub policy_version: String,
    
    /// Cryptographic signature from Phase 6 policy engine
    pub signature: String,
    
    /// Signature hash for integrity verification
    pub signature_hash: String,
    
    /// Timestamp when directive was issued
    pub issued_at: DateTime<Utc>,
    
    /// Time-to-live in seconds (expired directives MUST be rejected)
    pub ttl_seconds: u64,
    
    /// Nonce for replay protection (MUST be unique per directive)
    pub nonce: String,
    
    /// Target scope - specifies which agents/hosts this applies to
    pub target_scope: TargetScope,
    
    /// Action to be executed
    pub action: String,
    
    /// Preconditions hash - validates system state before execution
    pub preconditions_hash: String,
    
    /// Audit receipt from Phase 6 (proves decision was logged)
    pub audit_receipt: AuditReceipt,
    
    /// Allowed actions list (from policy)
    pub allowed_actions: Vec<String>,
    
    /// Required approvals (from policy)
    pub required_approvals: Vec<String>,
    
    /// Evidence reference
    pub evidence_reference: String,
    
    /// Kill chain stage
    pub kill_chain_stage: String,
    
    /// Severity
    pub severity: String,
    
    /// Reasoning
    pub reasoning: String,
    
    /// Issuer role (must be "GOVERNOR" for dispatch)
    pub issuer_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetScope {
    /// Agent IDs (if specific agents are targeted)
    pub agent_ids: Option<Vec<String>>,
    
    /// Host addresses (if specific hosts are targeted)
    pub host_addresses: Option<Vec<String>>,
    
    /// Platform type (linux, windows, network)
    pub platform: Option<String>,
    
    /// Asset class
    pub asset_class: Option<String>,
    
    /// Environment
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditReceipt {
    /// Receipt ID from Phase 6
    pub receipt_id: String,
    
    /// Receipt signature
    pub receipt_signature: String,
    
    /// Receipt hash
    pub receipt_hash: String,
    
    /// Timestamp when receipt was created
    pub receipt_timestamp: DateTime<Utc>,
}

impl DirectiveEnvelope {
    /// Create a new directive envelope
    pub fn new(
        policy_id: String,
        policy_version: String,
        signature: String,
        signature_hash: String,
        ttl_seconds: u64,
        nonce: String,
        target_scope: TargetScope,
        action: String,
        preconditions_hash: String,
        audit_receipt: AuditReceipt,
        allowed_actions: Vec<String>,
        required_approvals: Vec<String>,
        evidence_reference: String,
        kill_chain_stage: String,
        severity: String,
        reasoning: String,
        issuer_role: String,
    ) -> Self {
        Self {
            directive_id: Uuid::now_v7().to_string(),
            policy_id,
            policy_version,
            signature,
            signature_hash,
            issued_at: Utc::now(),
            ttl_seconds,
            nonce,
            target_scope,
            action,
            preconditions_hash,
            audit_receipt,
            allowed_actions,
            required_approvals,
            evidence_reference,
            kill_chain_stage,
            severity,
            reasoning,
            issuer_role,
        }
    }
    
    /// Check if directive has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiry = self.issued_at + chrono::Duration::seconds(self.ttl_seconds as i64);
        now > expiry
    }
    
    /// Get expiry timestamp
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.issued_at + chrono::Duration::seconds(self.ttl_seconds as i64)
    }
    
    /// Validate all required fields are present
    pub fn validate_structure(&self) -> Result<(), String> {
        if self.directive_id.is_empty() {
            return Err("directive_id is required".to_string());
        }
        if self.policy_id.is_empty() {
            return Err("policy_id is required".to_string());
        }
        if self.policy_version.is_empty() {
            return Err("policy_version is required".to_string());
        }
        if self.signature.is_empty() {
            return Err("signature is required".to_string());
        }
        if self.signature_hash.is_empty() {
            return Err("signature_hash is required".to_string());
        }
        if self.nonce.is_empty() {
            return Err("nonce is required".to_string());
        }
        if self.action.is_empty() {
            return Err("action is required".to_string());
        }
        if self.preconditions_hash.is_empty() {
            return Err("preconditions_hash is required".to_string());
        }
        if self.audit_receipt.receipt_id.is_empty() {
            return Err("audit_receipt.receipt_id is required".to_string());
        }
        if self.audit_receipt.receipt_signature.is_empty() {
            return Err("audit_receipt.receipt_signature is required".to_string());
        }
        Ok(())
    }
}

