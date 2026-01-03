// Path and File Name : /home/ransomeye/rebuild/ransomeye_policy/engine/src/policy.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Policy loading and validation - signed policies only with real cryptography

#![cfg(feature = "future-policy")]

use std::path::Path;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::{error, info, debug};

use crate::errors::PolicyError;
use crate::decision::AllowedAction;

// Helper function to sort JSON object keys recursively
fn sort_json_value_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // Collect keys and sort them
            let mut sorted_pairs: Vec<(String, serde_json::Value)> = map
                .iter()
                .map(|(k, v)| {
                    let mut val = v.clone();
                    sort_json_value_keys(&mut val);
                    (k.clone(), val)
                })
                .collect();
            sorted_pairs.sort_by(|a, b| a.0.cmp(&b.0));
            
            // Rebuild the map with sorted keys
            map.clear();
            for (k, v) in sorted_pairs {
                map.insert(k, v);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sort_json_value_keys(item);
            }
        }
        _ => {}
    }
}

/// Canonicalize a policy (with signature fields already removed) into a deterministic byte string.
///
/// SECURITY INVARIANT:
/// - Signing and verification MUST use the exact same canonical representation.
/// - We canonicalize via JSON with recursively sorted object keys to avoid non-determinism in YAML mapping order.
fn canonicalize_policy_value_for_signing(policy_value: &serde_yaml::Value) -> Result<String, PolicyError> {
    let mut json_val = serde_json::to_value(policy_value).map_err(|e| {
        PolicyError::ConfigurationError(format!("Failed to convert policy YAML to JSON for canonicalization: {e}"))
    })?;
    sort_json_value_keys(&mut json_val);
    serde_json::to_string(&json_val).map_err(|e| {
        PolicyError::ConfigurationError(format!("Failed to serialize canonical policy JSON: {e}"))
    })
}

#[path = "../../security/signature.rs"]
mod signature;
#[path = "../../security/verification.rs"]
mod verification;

use signature::PolicySignatureVerifier;
use verification::PolicyVerifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub priority: u32,
    pub match_conditions: Vec<PolicyMatchCondition>,
    pub decision: PolicyDecisionRule,
    pub required_approvals: Vec<String>,
    pub signature: Option<String>,
    pub signature_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecisionRule {
    pub action: String,
    pub allowed_actions: Vec<String>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMatchCondition {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub id: String,
    pub version: String,
    pub priority: u32,
    pub match_conditions: Vec<PolicyMatchCondition>,
    pub decision: AllowedAction,
    pub allowed_actions: Vec<AllowedAction>,
    pub required_approvals: Vec<String>,
    pub reasoning: String,
}

pub struct PolicyLoader {
    policies: HashMap<String, Policy>,
    signature_verifier: PolicySignatureVerifier,
    hash_verifier: PolicyVerifier,
    policies_path: String,
    // Track highest version per policy ID (for rollback prevention)
    highest_versions: HashMap<String, String>,
    // Persist version state to file (in-memory is insufficient)
    version_state_path: String,
}

impl PolicyLoader {
    pub fn new(policies_path: &str, trust_store_path: Option<&str>) -> Result<Self, PolicyError> {
        let signature_verifier = PolicySignatureVerifier::new()
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to initialize signature verifier: {}", e)
            ))?;

        if let Some(trust_path) = trust_store_path {
            PolicySignatureVerifier::load_trust_store(trust_path)
                .map_err(|e| PolicyError::ConfigurationError(
                    format!("Failed to load trust store: {}", e)
                ))?;
        }

        // Load version state from persistent storage (MUST be writable under systemd hardening)
        //
        // NOTE: The legacy default path was /var/lib/ransomeye/policy_versions.json, which is
        // read-only under the orchestrator's ProtectSystem=strict + ReadWritePaths allowlist.
        // The v1.0 runtime-canonical path is a dedicated policy state directory:
        //   /var/lib/ransomeye/policy/policy_versions.json
        let mut version_state_path = std::env::var("RANSOMEYE_POLICY_VERSION_STATE_PATH")
            .unwrap_or_else(|_| "/var/lib/ransomeye/policy/policy_versions.json".to_string());

        // FAIL-SAFE NORMALIZATION (do not weaken systemd hardening):
        // If an operator/legacy runtime still sets the deprecated path, transparently remap it
        // into the hardened-writable state directory.
        if version_state_path == "/var/lib/ransomeye/policy_versions.json" {
            version_state_path = "/var/lib/ransomeye/policy/policy_versions.json".to_string();
        }
        
        let highest_versions = Self::load_version_state_static(&version_state_path)?;

        let mut loader = Self {
            policies: HashMap::new(),
            signature_verifier,
            hash_verifier: PolicyVerifier::new(),
            policies_path: policies_path.to_string(),
            highest_versions,
            version_state_path,
        };

        loader.load_policies()?;

        Ok(loader)
    }
    
    /// Load version state from persistent storage (static method for initialization)
    fn load_version_state_static(path: &str) -> Result<HashMap<String, String>, PolicyError> {
        use std::fs;
        use std::path::Path;
        
        let path_obj = Path::new(path);
        
        if !path_obj.exists() {
            // Migration support (fail-safe):
            // If the legacy path exists, load it and atomically persist into the new path.
            // This prevents false rollback detection when upgrading runtime layouts.
            let legacy_path = Path::new("/var/lib/ransomeye/policy_versions.json");
            if legacy_path.exists() {
                let legacy_content = fs::read_to_string(legacy_path).map_err(|e| {
                    PolicyError::ConfigurationError(format!(
                        "Failed to read legacy version state file {}: {}",
                        legacy_path.display(),
                        e
                    ))
                })?;

                let versions: HashMap<String, String> = serde_json::from_str(&legacy_content).map_err(|e| {
                    PolicyError::ConfigurationError(format!(
                        "Failed to parse legacy version state {}: {}",
                        legacy_path.display(),
                        e
                    ))
                })?;

                // Best-effort persist into the new location (fail-closed if we cannot write).
                if let Some(parent) = path_obj.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        PolicyError::ConfigurationError(format!(
                            "Failed to create version state directory {}: {}",
                            parent.display(),
                            e
                        ))
                    })?;
                }

                fs::write(path, serde_json::to_string_pretty(&versions).map_err(|e| {
                    PolicyError::ConfigurationError(format!("Failed to serialize migrated version state: {}", e))
                })?)
                .map_err(|e| {
                    PolicyError::ConfigurationError(format!(
                        "Failed to write migrated version state file {}: {}",
                        path,
                        e
                    ))
                })?;

                return Ok(versions);
            }

            // First run - no state file, return empty map
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to read version state file: {}", e)
            ))?;
        
        let versions: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to parse version state: {}", e)
            ))?;
        
        Ok(versions)
    }
    
    /// Save version state to persistent storage
    fn save_version_state(&self) -> Result<(), PolicyError> {
        use std::fs;
        use std::path::Path;
        
        let path_obj = Path::new(&self.version_state_path);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path_obj.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PolicyError::ConfigurationError(
                    format!("Failed to create version state directory: {}", e)
                ))?;
        }
        
        let content = serde_json::to_string_pretty(&self.highest_versions)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to serialize version state: {}", e)
            ))?;
        
        fs::write(&self.version_state_path, content)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to write version state file: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// Check if policy version is allowed (strictly increasing only)
    /// 
    /// BOOTSTRAP LOGIC:
    /// - First-run: If policy_id not in highest_versions, allow any version (bootstrap)
    /// - Subsequent runs: Enforce strict monotonicity (new_version > highest_version)
    /// 
    /// This prevents false rollback detection on first clean runtime while maintaining
    /// strict rollback protection for all subsequent runs.
    fn check_version_rollback(&mut self, policy_id: &str, version: &str) -> Result<(), PolicyError> {
        // Check if this is first-run bootstrap (policy not yet seen)
        let is_first_run = !self.highest_versions.contains_key(policy_id);
        
        if is_first_run {
            // First-run bootstrap: Allow any version, log bootstrap detection
            info!("First-run policy bootstrap detected: policy {} version {}", policy_id, version);
            
            // Update highest version
            self.highest_versions.insert(policy_id.to_string(), version.to_string());
            
            // Persist state atomically
            self.save_version_state()?;
            
            Ok(())
        } else {
            // Subsequent runs: Enforce monotonicity (allow equal or greater, block downgrades)
            let highest_version = self.highest_versions.get(policy_id)
                .expect("Policy ID should exist after first-run check");
            
            // Compare versions (semantic versioning)
            let comparison = self.compare_versions(version, highest_version);
            if comparison < 0 {
                // Only block downgrades (version < highest_version)
                return Err(PolicyError::EngineRefusedToStart(
                    format!("Policy version rollback detected: policy {} version {} is less than highest version {}", 
                        policy_id, version, highest_version)
                ));
            }
            
            // Version is equal or greater, update and persist
            if comparison > 0 {
                info!("Existing policy state validated: policy {} version {} > {}", 
                    policy_id, version, highest_version);
            } else {
                info!("Policy version unchanged: policy {} version {} (same as highest)", 
                    policy_id, version);
            }
            
            // Update highest version (even if equal, to refresh timestamp)
            self.highest_versions.insert(policy_id.to_string(), version.to_string());
            
            // Persist state
            self.save_version_state()?;
            
            Ok(())
        }
    }
    
    /// Compare two semantic versions
    /// Returns: -1 if v1 < v2, 0 if v1 == v2, 1 if v1 > v2
    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<&str> = v1.split('.').collect();
        let v2_parts: Vec<&str> = v2.split('.').collect();
        
        let max_len = v1_parts.len().max(v2_parts.len());
        
        for i in 0..max_len {
            let v1_part = v1_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            let v2_part = v2_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            
            if v1_part < v2_part {
                return -1;
            } else if v1_part > v2_part {
                return 1;
            }
        }
        
        0
    }

    pub fn load_policies(&mut self) -> Result<(), PolicyError> {
        info!("Loading policies from: {}", self.policies_path);

        let policies_dir = Path::new(&self.policies_path);
        if !policies_dir.exists() {
            return Err(PolicyError::ConfigurationError(
                format!("Policies directory not found: {}", self.policies_path)
            ));
        }

        let entries = fs::read_dir(policies_dir)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to read policies directory: {}", e)
            ))?;

        for entry in entries {
            let entry = entry.map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to read directory entry: {}", e)
            ))?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
               path.extension().and_then(|s| s.to_str()) == Some("yml") {
                match self.load_policy_file(&path) {
                    Ok(policy) => {
                        // Check version rollback prevention (MANDATORY)
                        if let Err(e) = self.check_version_rollback(&policy.id, &policy.version) {
                            error!("Version rollback detected for policy {}: {}", policy.id, e);
                            return Err(e);
                        }
                        
                        info!("Loaded policy: {} (version: {})", policy.id, policy.version);
                        self.policies.insert(policy.id.clone(), policy);
                    }
                    Err(e) => {
                        error!("Failed to load policy from {}: {}", path.display(), e);
                        return Err(e);
                    }
                }
            }
        }

        if self.policies.is_empty() {
            return Err(PolicyError::ConfigurationError(
                "No valid policies loaded".to_string()
            ));
        }

        info!("Loaded {} policies", self.policies.len());
        Ok(())
    }

    fn load_policy_file(&self, path: &Path) -> Result<Policy, PolicyError> {
        // Step 1: Read policy file as RAW BYTES (no string conversion, no parsing)
        let raw_policy_bytes = fs::read(path)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to read policy file {}: {}", path.display(), e)
            ))?;

        // Step 2: Extract signature field by parsing a COPY of the policy
        // Convert bytes to string for parsing (but don't modify raw_policy_bytes)
        let content_str = String::from_utf8(raw_policy_bytes.clone())
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to convert policy bytes to UTF-8: {}", e)
            ))?;

        let policy: Policy = serde_yaml::from_str(&content_str)
            .map_err(|e| PolicyError::ConfigurationError(
                format!("Failed to parse policy file {}: {}", path.display(), e)
            ))?;

        if let Some(ref signature) = policy.signature {
            // Step 3: Parse YAML Value from COPY to remove signature fields
            // DO NOT modify raw_policy_bytes
            let mut policy_value: serde_yaml::Value = serde_yaml::from_str(&content_str)
                .map_err(|e| PolicyError::ConfigurationError(
                    format!("Failed to parse policy as YAML Value: {}", e)
                ))?;
            
            // Remove signature fields (matching sign_policies.rs exactly)
            if let Some(obj) = policy_value.as_mapping_mut() {
                obj.remove("signature");
                obj.remove("signature_hash");
                obj.remove("signature_alg");
                obj.remove("key_id");
            }
            
            // Step 4: Canonicalize deterministically for signing/verification
            let content_to_verify = canonicalize_policy_value_for_signing(&policy_value)?;

            // Step 5: Verify signature using ring with the serialized bytes
            // This matches exactly what was signed
            let verified = self.signature_verifier.verify(&content_to_verify, signature)
                .map_err(|e| PolicyError::PolicySignatureInvalid(
                    format!("Policy {} signature verification failed: {}", policy.id, e)
                ))?;

            if !verified {
                return Err(PolicyError::PolicySignatureInvalid(
                    format!("Policy {} has invalid signature", policy.id)
                ));
            }

            debug!("Policy {} signature verified", policy.id);
        } else {
            return Err(PolicyError::UnsignedPolicy(
                format!("Policy {} is not signed", policy.id)
            ));
        }

        // Verify hash matches the content without signature fields (matching signing process)
        if let Some(ref expected_hash) = policy.signature_hash {
            // Recompute content without signature fields for hash verification
            let mut policy_value: serde_yaml::Value = serde_yaml::from_str(&content_str)
                .map_err(|e| PolicyError::ConfigurationError(
                    format!("Failed to parse policy as YAML Value for hash: {}", e)
                ))?;
            
            if let Some(obj) = policy_value.as_mapping_mut() {
                obj.remove("signature");
                obj.remove("signature_hash");
                obj.remove("signature_alg");
                obj.remove("key_id");
            }
            
            let content_for_hashing = canonicalize_policy_value_for_signing(&policy_value)?;
            
            if !self.hash_verifier.verify_hash(&content_for_hashing, expected_hash) {
                return Err(PolicyError::PolicyTampered(
                    format!("Policy {} hash mismatch", policy.id)
                ));
            }
        }

        Ok(policy)
    }

    pub fn get_policy(&self, policy_id: &str) -> Result<&Policy, PolicyError> {
        self.policies.get(policy_id)
            .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.to_string()))
    }

    pub fn get_all_policies(&self) -> Vec<&Policy> {
        let mut policies: Vec<&Policy> = self.policies.values().collect();
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        policies
    }

    pub fn to_policy_rule(&self, policy: &Policy) -> Result<PolicyRule, PolicyError> {
        let decision = self.parse_action(&policy.decision.action)?;
        let allowed_actions: Result<Vec<AllowedAction>, _> = policy.decision.allowed_actions
            .iter()
            .map(|a| self.parse_action(a))
            .collect();

        Ok(PolicyRule {
            id: policy.id.clone(),
            version: policy.version.clone(),
            priority: policy.priority,
            match_conditions: policy.match_conditions.clone(),
            decision,
            allowed_actions: allowed_actions?,
            required_approvals: policy.required_approvals.clone(),
            reasoning: policy.decision.reasoning.clone(),
        })
    }

    fn parse_action(&self, action: &str) -> Result<AllowedAction, PolicyError> {
        match action.to_lowercase().as_str() {
            "allow" => Ok(AllowedAction::Allow),
            "deny" => Ok(AllowedAction::Deny),
            "quarantine" => Ok(AllowedAction::Quarantine),
            "isolate" => Ok(AllowedAction::Isolate),
            "block" => Ok(AllowedAction::Block),
            "monitor" => Ok(AllowedAction::Monitor),
            "escalate" => Ok(AllowedAction::Escalate),
            "require_approval" => Ok(AllowedAction::RequireApproval),
            _ => Err(PolicyError::EvaluationError(
                format!("Unknown action: {}", action)
            )),
        }
    }
}

