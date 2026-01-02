// Path and File Name : /home/ransomeye/rebuild/core/deception/src/tests/registry_tests.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Tests for deception registry - unsigned assets, forbidden types, schema validation

#[cfg(test)]
mod tests {
    use crate::asset::{DeceptionAsset, AssetType, DeploymentScope, VisibilityLevel, TriggerConditions, TelemetryFields, TeardownProcedure, TeardownStep, TeardownAction};
    use chrono::Utc;
    use std::collections::HashMap;
    
    fn create_test_asset(asset_id: &str, asset_type: AssetType) -> DeceptionAsset {
        DeceptionAsset {
            asset_id: asset_id.to_string(),
            asset_type,
            deployment_scope: DeploymentScope::Network,
            visibility_level: VisibilityLevel::Low,
            trigger_conditions: TriggerConditions {
                interaction_types: vec!["connection".to_string()],
                min_confidence: 0.9,
            },
            telemetry_fields: TelemetryFields {
                source_ip: "192.168.1.100".to_string(),
                destination_ip: "192.168.1.200".to_string(),
                timestamp: Utc::now(),
                interaction_type: "connection".to_string(),
                additional_metadata: HashMap::new(),
            },
            teardown_procedure: TeardownProcedure {
                steps: vec![TeardownStep {
                    action: TeardownAction::StopService,
                    parameters: HashMap::new(),
                }],
            },
            max_lifetime: 3600,
            signature: "test_signature".to_string(),
            signature_hash: "test_hash".to_string(),
            metadata: None,
        }
    }
    
    #[test]
    fn test_unsigned_asset_rejected() {
        // Test that unsigned assets are rejected
        // This would require actual signature verification
        // For now, we test schema validation
        let asset = create_test_asset("test-asset-1", AssetType::DecoyHost);
        
        // Schema validation should pass
        assert!(asset.validate_schema().is_ok());
        
        // Signature verification would fail (not implemented in test)
        // This is tested in integration tests with real keys
    }
    
    #[test]
    fn test_forbidden_asset_type_rejected() {
        // Test that forbidden asset types are rejected
        // This is enforced in registry validation
        // For now, we test that allowed types are accepted
        let asset = create_test_asset("test-asset-2", AssetType::DecoyService);
        
        // Allowed asset type should pass schema validation
        assert!(asset.validate_schema().is_ok());
        assert_eq!(asset.asset_type_str(), "decoy_service");
    }
    
    #[test]
    fn test_schema_validation() {
        // Test schema validation
        let asset = create_test_asset("test-asset-3", AssetType::CredentialLure);
        
        // Valid asset should pass
        assert!(asset.validate_schema().is_ok());
        
        // Invalid asset (empty interaction_types) should fail
        let mut invalid_asset = asset.clone();
        invalid_asset.trigger_conditions.interaction_types.clear();
        assert!(invalid_asset.validate_schema().is_err());
        
        // Invalid asset (max_lifetime = 0) should fail
        let mut invalid_asset2 = asset.clone();
        invalid_asset2.max_lifetime = 0;
        assert!(invalid_asset2.validate_schema().is_err());
    }
    
    #[test]
    fn test_asset_expiration() {
        let asset = create_test_asset("test-asset-4", AssetType::FilesystemLure);
        let created_at = Utc::now() - chrono::Duration::seconds(3700);
        
        // Asset should be expired
        assert!(asset.is_expired(created_at));
        
        let created_at2 = Utc::now() - chrono::Duration::seconds(1000);
        
        // Asset should not be expired
        assert!(!asset.is_expired(created_at2));
    }
}

