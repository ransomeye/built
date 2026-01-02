// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/kill_chain/stages.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Ransomware-specific kill-chain stage definitions with entry conditions and evidence requirements

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Ransomware-specific kill-chain stages (NOT generic MITRE)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RansomwareStage {
    /// Initial access to the target system
    InitialAccess,
    /// Malicious code execution
    Execution,
    /// Establishing persistence mechanisms
    Persistence,
    /// Gaining elevated privileges
    PrivilegeEscalation,
    /// Moving across network boundaries
    LateralMovement,
    /// Stealing credentials
    CredentialAccess,
    /// Discovering system resources and data
    Discovery,
    /// Preparing for encryption (key generation, file enumeration)
    EncryptionPreparation,
    /// Actively encrypting files
    EncryptionExecution,
    /// Final impact (data encrypted, ransom note dropped)
    Impact,
}

impl RansomwareStage {
    /// Get all stages in order
    pub fn all_stages() -> Vec<RansomwareStage> {
        vec![
            RansomwareStage::InitialAccess,
            RansomwareStage::Execution,
            RansomwareStage::Persistence,
            RansomwareStage::PrivilegeEscalation,
            RansomwareStage::LateralMovement,
            RansomwareStage::CredentialAccess,
            RansomwareStage::Discovery,
            RansomwareStage::EncryptionPreparation,
            RansomwareStage::EncryptionExecution,
            RansomwareStage::Impact,
        ]
    }

    /// Get stage index (0-based)
    pub fn index(&self) -> usize {
        match self {
            RansomwareStage::InitialAccess => 0,
            RansomwareStage::Execution => 1,
            RansomwareStage::Persistence => 2,
            RansomwareStage::PrivilegeEscalation => 3,
            RansomwareStage::LateralMovement => 4,
            RansomwareStage::CredentialAccess => 5,
            RansomwareStage::Discovery => 6,
            RansomwareStage::EncryptionPreparation => 7,
            RansomwareStage::EncryptionExecution => 8,
            RansomwareStage::Impact => 9,
        }
    }

    /// Get stage name as string
    pub fn name(&self) -> &'static str {
        match self {
            RansomwareStage::InitialAccess => "InitialAccess",
            RansomwareStage::Execution => "Execution",
            RansomwareStage::Persistence => "Persistence",
            RansomwareStage::PrivilegeEscalation => "PrivilegeEscalation",
            RansomwareStage::LateralMovement => "LateralMovement",
            RansomwareStage::CredentialAccess => "CredentialAccess",
            RansomwareStage::Discovery => "Discovery",
            RansomwareStage::EncryptionPreparation => "EncryptionPreparation",
            RansomwareStage::EncryptionExecution => "EncryptionExecution",
            RansomwareStage::Impact => "Impact",
        }
    }
}

/// Entry conditions for a kill-chain stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEntryConditions {
    /// Required signal types that must be present
    pub required_signals: HashSet<String>,
    /// Minimum number of signals required
    pub min_signal_count: usize,
    /// Temporal constraints (e.g., signals must occur within X seconds)
    pub temporal_window_seconds: Option<u64>,
}

/// Evidence requirements for stage transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirements {
    /// Minimum confidence level required
    pub min_confidence: f64,
    /// Required evidence types
    pub required_evidence_types: HashSet<String>,
    /// Whether previous stage must be completed
    pub requires_previous_stage: bool,
}

/// Stage metadata with entry conditions and evidence requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetadata {
    pub stage: RansomwareStage,
    pub entry_conditions: StageEntryConditions,
    pub evidence_requirements: EvidenceRequirements,
    /// Confidence decay rate per hour if no new signals
    pub confidence_decay_per_hour: f64,
}

impl StageMetadata {
    /// Get default metadata for a stage
    pub fn default_for(stage: RansomwareStage) -> Self {
        let (required_signals, min_signal_count, temporal_window) = match stage {
            RansomwareStage::InitialAccess => (
                vec!["network_connection".to_string(), "process_creation".to_string()],
                1,
                Some(300),
            ),
            RansomwareStage::Execution => (
                vec!["process_creation".to_string(), "file_execution".to_string()],
                1,
                Some(60),
            ),
            RansomwareStage::Persistence => (
                vec!["registry_modification".to_string(), "service_creation".to_string(), "scheduled_task".to_string()],
                1,
                Some(300),
            ),
            RansomwareStage::PrivilegeEscalation => (
                vec!["privilege_escalation".to_string(), "token_manipulation".to_string()],
                1,
                Some(300),
            ),
            RansomwareStage::LateralMovement => (
                vec!["network_connection".to_string(), "remote_execution".to_string()],
                1,
                Some(600),
            ),
            RansomwareStage::CredentialAccess => (
                vec!["credential_dump".to_string(), "lsass_access".to_string()],
                1,
                Some(300),
            ),
            RansomwareStage::Discovery => (
                vec!["file_enumeration".to_string(), "network_scanning".to_string(), "system_info".to_string()],
                2,
                Some(600),
            ),
            RansomwareStage::EncryptionPreparation => (
                vec!["file_enumeration".to_string(), "key_generation".to_string(), "process_creation".to_string()],
                2,
                Some(300),
            ),
            RansomwareStage::EncryptionExecution => (
                vec!["file_modification".to_string(), "encryption_activity".to_string()],
                2,
                Some(60),
            ),
            RansomwareStage::Impact => (
                vec!["ransom_note".to_string(), "file_encryption_complete".to_string()],
                1,
                Some(300),
            ),
        };

        Self {
            stage,
            entry_conditions: StageEntryConditions {
                required_signals: required_signals.into_iter().collect(),
                min_signal_count,
                temporal_window_seconds: temporal_window,
            },
            evidence_requirements: EvidenceRequirements {
                min_confidence: 0.6,
                required_evidence_types: HashSet::new(),
                requires_previous_stage: true,
            },
            confidence_decay_per_hour: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_ordering() {
        let stages = RansomwareStage::all_stages();
        for (i, stage) in stages.iter().enumerate() {
            assert_eq!(stage.index(), i);
        }
    }

    #[test]
    fn test_stage_metadata_defaults() {
        for stage in RansomwareStage::all_stages() {
            let metadata = StageMetadata::default_for(stage);
            assert!(!metadata.entry_conditions.required_signals.is_empty());
            assert!(metadata.entry_conditions.min_signal_count > 0);
            assert!(metadata.evidence_requirements.min_confidence > 0.0);
            assert!(metadata.evidence_requirements.min_confidence <= 1.0);
        }
    }
}

