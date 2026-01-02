// Path and File Name : /home/ransomeye/rebuild/ransomeye_dispatcher/protocol/acknowledgment_envelope.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gQ
// Details of functionality of this file: Acknowledgment envelope for agent execution confirmation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// AcknowledgmentEnvelope - Agent execution confirmation
/// 
/// Agents MUST return this structure after executing a directive.
/// Must be signed by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcknowledgmentEnvelope {
    /// Directive ID being acknowledged
    pub directive_id: String,
    
    /// Agent ID that executed the directive
    pub agent_id: String,
    
    /// Execution result
    pub execution_result: ExecutionResult,
    
    /// Timestamp when execution completed
    pub timestamp: DateTime<Utc>,
    
    /// Agent signature (proves agent executed this)
    pub signature: String,
    
    /// Signature hash
    pub signature_hash: String,
    
    /// Execution details (optional)
    pub execution_details: Option<String>,
    
    /// Error message (if execution failed)
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionResult {
    /// Execution succeeded
    Success,
    
    /// Execution failed
    Failed,
    
    /// Execution partially completed
    Partial,
    
    /// Execution was skipped (e.g., target not found)
    Skipped,
}

impl AcknowledgmentEnvelope {
    /// Create a new acknowledgment
    pub fn new(
        directive_id: String,
        agent_id: String,
        execution_result: ExecutionResult,
        signature: String,
        signature_hash: String,
    ) -> Self {
        Self {
            directive_id,
            agent_id,
            execution_result,
            timestamp: Utc::now(),
            signature,
            signature_hash,
            execution_details: None,
            error_message: None,
        }
    }
    
    /// Validate structure
    pub fn validate_structure(&self) -> Result<(), String> {
        if self.directive_id.is_empty() {
            return Err("directive_id is required".to_string());
        }
        if self.agent_id.is_empty() {
            return Err("agent_id is required".to_string());
        }
        if self.signature.is_empty() {
            return Err("signature is required".to_string());
        }
        if self.signature_hash.is_empty() {
            return Err("signature_hash is required".to_string());
        }
        Ok(())
    }
}

