// Path and File Name : /home/ransomeye/rebuild/ransomeye_ingestion/src/schema.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Schema validation - strictly validates event schemas and versions

/*
 * Schema Validation
 * 
 * Strictly validates event schemas against JSON schema definitions.
 * Enforces version compatibility rules.
 * Fails-closed on schema violations.
 */

use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use tracing::{error, debug};
use jsonschema::{JSONSchema, Draft};

use crate::protocol::event_envelope::EventEnvelope;
use crate::config::Config;
use crate::versioning::VersionManager;

pub struct SchemaValidator {
    config: Config,
    version_manager: Arc<VersionManager>,
    schemas: HashMap<u32, Value>,
    compiled_schemas: HashMap<u32, JSONSchema>,
}

impl SchemaValidator {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut schemas = HashMap::new();
        let mut compiled_schemas = HashMap::new();
        
        // Load schema v1
        let schema_v1_path = std::path::Path::new("protocol/event_schema_v1.json");
        let schema_v1_str = std::fs::read_to_string(schema_v1_path)?;
        let schema_v1: Value = serde_json::from_str(&schema_v1_str)?;
        
        // Compile JSON schema for validation
        let compiled = JSONSchema::compile(&schema_v1)
            .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;
        
        schemas.insert(1, schema_v1);
        compiled_schemas.insert(1, compiled);
        
        Ok(Self {
            config: config.clone(),
            version_manager: Arc::new(VersionManager::new()?),
            schemas,
            compiled_schemas,
        })
    }
    
    pub async fn validate(&self, envelope: &EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Check version compatibility
        if !self.version_manager.is_compatible(envelope.schema_version) {
            error!("Incompatible schema version: {}", envelope.schema_version);
            return Err("Incompatible schema version".into());
        }
        
        // Validate envelope structure
        self.validate_envelope_structure(envelope)?;
        
        // Validate event data against schema
        let event_data: Value = serde_json::from_str(&envelope.event_data)
            .map_err(|e| format!("Failed to parse event data as JSON: {}", e))?;
        self.validate_against_schema(&event_data, envelope.schema_version)?;
        
        debug!("Schema validation passed for producer: {}", envelope.producer_id);
        Ok(())
    }
    
    fn validate_envelope_structure(&self, envelope: &EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Validate required fields
        if envelope.producer_id.is_empty() {
            return Err("Producer ID is required".into());
        }
        if envelope.component_type.is_empty() {
            return Err("Component type is required".into());
        }
        if envelope.signature.is_empty() {
            return Err("Signature is required".into());
        }
        if envelope.integrity_hash.is_empty() {
            return Err("Integrity hash is required".into());
        }
        
        Ok(())
    }
    
    fn validate_against_schema(&self, data: &Value, schema_version: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Use compiled JSON schema for strict validation
        let compiled_schema = self.compiled_schemas
            .get(&schema_version)
            .ok_or("Schema not compiled for version")?;
        
        // Validate event data against JSON schema
        let validation_result = compiled_schema.validate(data);
        
        match validation_result {
            Ok(_) => {
                debug!("Schema validation passed");
                Ok(())
            }
            Err(errors) => {
                let mut error_messages = Vec::new();
                for error in errors {
                    error_messages.push(format!("Schema validation error: {}", error));
                }
                let combined_error = error_messages.join("; ");
                error!("Schema validation failed: {}", combined_error);
                Err(combined_error.into())
            }
        }
    }
}

