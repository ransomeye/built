// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/output/receipt.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Signed decision receipt for verifiable replay

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use hex;
use base64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReceipt {
    pub receipt_id: String,
    pub created_at: DateTime<Utc>,
    pub decision_id: String,
    pub policy_id: String,
    pub decision_hash: String,
    pub audit_record_hash: String,
    pub signature: String,
}

impl DecisionReceipt {
    pub fn new(
        receipt_id: String,
        decision_id: String,
        policy_id: String,
        decision_hash: String,
        audit_record_hash: String,
    ) -> Self {
        let created_at = Utc::now();
        let mut receipt = Self {
            receipt_id,
            created_at,
            decision_id,
            policy_id,
            decision_hash,
            audit_record_hash,
            signature: String::new(),
        };

        receipt.signature = receipt.compute_signature();
        receipt
    }

    fn compute_signature(&self) -> String {
        let mut hasher = Sha256::new();
        let json_bytes = serde_json::to_vec(self).expect("Failed to serialize receipt");
        hasher.update(&json_bytes);
        let hash = hasher.finalize();
        base64::encode(hash)
    }

    pub fn verify(&self) -> bool {
        let expected_signature = self.signature.clone();
        let mut receipt_clone = self.clone();
        receipt_clone.signature = String::new();
        let computed_signature = receipt_clone.compute_signature();
        computed_signature == expected_signature
    }
}

pub struct ReceiptGenerator;

impl ReceiptGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        decision_id: &str,
        policy_id: &str,
        decision_hash: &str,
        audit_record_hash: &str,
    ) -> DecisionReceipt {
        use uuid::Uuid;
        DecisionReceipt::new(
            Uuid::new_v4().to_string(),
            decision_id.to_string(),
            policy_id.to_string(),
            decision_hash.to_string(),
            audit_record_hash.to_string(),
        )
    }
}

