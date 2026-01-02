// Path and File Name : /home/ransomeye/rebuild/core/guardrails/src/spec.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Guardrail specification data structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailSpec {
    pub version: String,
    pub spec_hash: String,
    pub signature: String,
    pub public_key: String,
    pub metadata: SpecMetadata,
    pub allowed_phases: Vec<Phase>,
    pub allowed_modules: Vec<String>,
    pub forbidden_modules: Vec<String>,
    pub env_only_rules: EnvOnlyRules,
    pub crypto_requirements: CryptoRequirements,
    pub fail_closed_requirements: Vec<String>,
    pub systemd_requirements: SystemdRequirements,
    pub standalone_agent_constraints: StandaloneAgentConstraints,
    pub requirements_policy: RequirementsPolicy,
    pub model_requirements: ModelRequirements,
    pub export_requirements: ExportRequirements,
    pub header_requirements: HeaderRequirements,
    pub database_policy: DatabasePolicy,
    pub offline_requirements: OfflineRequirements,
    pub testing_requirements: TestingRequirements,
    pub ci_requirements: CIRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub created: String,
    pub last_modified: String,
    pub signer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub id: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvOnlyRules {
    pub required_env_vars: Vec<String>,
    pub threat_intel_env_vars: Vec<String>,
    pub forbidden_patterns: Vec<ForbiddenPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPattern {
    pub pattern: String,
    pub description: String,
    #[serde(default)]
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoRequirements {
    pub signing_algorithms: Vec<String>,
    pub hash_algorithms: Vec<String>,
    pub encryption_algorithms: Vec<String>,
    pub required_signing_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdRequirements {
    pub unified_directory: String,
    pub required_service_properties: Vec<String>,
    pub standalone_exceptions: Vec<StandaloneException>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneException {
    pub path: String,
    pub allowed_systemd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneAgentConstraints {
    pub linux_agent: AgentConstraint,
    pub windows_agent: AgentConstraint,
    pub dpi_probe: AgentConstraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConstraint {
    pub path: String,
    pub allowed_requirements_txt: bool,
    pub allowed_systemd: bool,
    pub must_be_rootless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsPolicy {
    pub unified_requirements: String,
    pub standalone_exceptions: Vec<String>,
    pub forbidden_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub allowed_formats: Vec<String>,
    pub required_metadata: Vec<String>,
    pub required_shap: bool,
    pub reject_if_missing_shap: bool,
    pub reject_if_missing_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequirements {
    pub required_formats: Vec<String>,
    pub optional_formats: Vec<String>,
    pub required_footer: String,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRequirements {
    pub required_files: Vec<String>,
    pub header_format: String,
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePolicy {
    pub allowed_database: String,
    pub forbidden_databases: Vec<String>,
    pub credentials: DatabaseCredentials,
    pub retention_max_years: u32,
    pub disk_cleanup_threshold_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCredentials {
    pub db_user: String,
    pub db_pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineRequirements {
    pub must_operate_offline: bool,
    pub threat_intel_must_cache: bool,
    pub no_internet_calls_runtime: bool,
    pub no_cdn_dependencies: bool,
    pub api_dependencies_only_with_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingRequirements {
    pub required_test_structure: String,
    pub required_test_types: Vec<String>,
    pub validation_output: String,
    pub no_placeholders: bool,
    pub no_mockups: bool,
    pub no_todos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIRequirements {
    pub must_detect_phantom_modules: bool,
    pub must_detect_forbidden_directories: bool,
    pub must_detect_hardcoded_configs: bool,
    pub must_detect_systemd_misplacement: bool,
    pub must_detect_unsigned_artifacts: bool,
    pub must_fail_on_violation: bool,
}

