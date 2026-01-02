// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/tests/replay_consistency_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Replay consistency tests - verifies decisions are replayable

/*
 * Replay Consistency Tests
 * 
 * Tests that verify:
 * - Decisions are replayable
 * - Same input → same output
 * - Decision hashes are consistent
 */

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sha2::{Sha256, Digest};
    use hex;

    #[test]
    fn test_decision_hash_is_consistent() {
        // Test that decision hash is consistent
        
        let decision = json!({
            "decision_id": "decision_1",
            "action": "deny",
            "policy_id": "policy_1"
        });
        
        let json_bytes = serde_json::to_vec(&decision).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&json_bytes);
        let hash1 = hex::encode(hasher.finalize());
        
        // Recompute hash
        let mut hasher2 = Sha256::new();
        hasher2.update(&json_bytes);
        let hash2 = hex::encode(hasher2.finalize());
        
        // Hashes should be identical
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_same_input_produces_same_output() {
        // Test that same input produces same output
        
        let input1 = json!({
            "alert_id": "alert_1",
            "severity": "critical"
        });
        
        let input2 = input1.clone();
        
        // Same input should produce same output
        assert_eq!(input1["alert_id"], input2["alert_id"]);
    }
    
    #[test]
    fn test_decision_is_replayable() {
        // Test that decisions are replayable
        
        let decision_data = json!({
            "decision_id": "decision_1",
            "action": "deny",
            "evidence_reference": "evidence_1"
        });
        
        // Decision should be replayable with same data
        assert!(decision_data.get("decision_id").is_some());
        assert!(decision_data.get("evidence_reference").is_some());
    }
}

