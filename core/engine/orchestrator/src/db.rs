// Path and File Name : /home/ransomeye/rebuild/core/engine/orchestrator/src/db.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Core Orchestrator database wiring for applying/validating the authoritative schema and writing core runtime records (startup/health/error/audit) fail-closed.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub pass: String,
}

impl DbConfig {
    /// Strict DB config from environment (FAIL-CLOSED on missing/invalid).
    pub fn from_env_strict() -> Result<Self, String> {
        let required = ["DB_HOST", "DB_PORT", "DB_NAME", "DB_USER", "DB_PASS"];
        let mut missing: Vec<&str> = Vec::new();
        for k in required {
            if std::env::var(k).is_err() {
                missing.push(k);
            }
        }
        if !missing.is_empty() {
            return Err(format!(
                "FAIL-CLOSED: Missing required database environment variables: {}",
                missing.join(", ")
            ));
        }

        let host = std::env::var("DB_HOST").map_err(|e| format!("DB_HOST read error: {e}"))?;
        let port_str = std::env::var("DB_PORT").map_err(|e| format!("DB_PORT read error: {e}"))?;
        let port = port_str
            .parse::<u16>()
            .map_err(|e| format!("Invalid DB_PORT '{port_str}': {e}"))?;
        let name = std::env::var("DB_NAME").map_err(|e| format!("DB_NAME read error: {e}"))?;
        let user = std::env::var("DB_USER").map_err(|e| format!("DB_USER read error: {e}"))?;
        let pass = std::env::var("DB_PASS").map_err(|e| format!("DB_PASS read error: {e}"))?;

        Ok(Self {
            host,
            port,
            name,
            user,
            pass,
        })
    }

    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.name, self.user, self.pass
        )
    }
}

#[derive(Debug)]
pub struct CoreDb {
    client: Client,
}

impl CoreDb {
    /// Connects and configures the session search_path for ransomeye schema use.
    pub async fn connect_strict(cfg: &DbConfig) -> Result<Self, String> {
        let (client, connection) = tokio_postgres::connect(&cfg.connection_string(), NoTls)
            .await
            .map_err(|e| format!("Database connection failed: {e}"))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection task error: {}", e);
            }
        });

        client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| format!("Database connection test query failed: {e}"))?;

        // Ensure queries resolve into ransomeye schema without explicit prefixes.
        client
            .batch_execute("SET search_path = ransomeye, public;")
            .await
            .map_err(|e| format!("Failed to set search_path: {e}"))?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Apply the authoritative schema SQL file (idempotent). FAIL-CLOSED if file missing/unreadable or DDL fails.
    pub async fn apply_authoritative_schema_from_env(&self) -> Result<(), String> {
        // Idempotency constraint:
        // The authoritative file contains CREATE TYPE statements WITHOUT IF NOT EXISTS.
        // Therefore, we must NOT blindly re-apply the full file on already-initialized databases.
        //
        // HARD REQUIREMENT (PROMPT-29B):
        // The runtime DB MUST match the certified schema, including ransomeye.retention_policies.
        // We support incremental, fail-closed schema completion for missing required tables by
        // extracting the exact table DDL from the authoritative file and applying ONLY the missing
        // table blocks (no CREATE TYPE re-execution).

        let schema_sql_path = std::env::var("RANSOMEYE_SCHEMA_SQL_PATH").map_err(|_| {
            "FAIL-CLOSED: RANSOMEYE_SCHEMA_SQL_PATH not set. Must point to the authoritative schema file."
                .to_string()
        })?;

        let schema_path = Path::new(&schema_sql_path);
        if !schema_path.exists() {
            return Err(format!(
                "FAIL-CLOSED: Authoritative schema file not found at RANSOMEYE_SCHEMA_SQL_PATH={}",
                schema_sql_path
            ));
        }

        // Probe whether schema types exist (gate for full-file apply).
        let component_type_exists = self
            .client
            .query_opt(
                r#"
                SELECT 1
                FROM pg_type t
                JOIN pg_namespace n ON n.oid = t.typnamespace
                WHERE n.nspname = 'ransomeye' AND t.typname = 'component_type'
                LIMIT 1
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("Failed to probe schema presence (types): {e}"))?
            .is_some();

        // Probe baseline table (gate for "schema present").
        let components_table_exists = self
            .client
            .query_opt(
                r#"
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'ransomeye' AND table_name = 'components'
                LIMIT 1
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("Failed to probe schema presence (tables): {e}"))?
            .is_some();

        let sql_raw = fs::read_to_string(schema_path).map_err(|e| {
            format!(
                "FAIL-CLOSED: Failed to read authoritative schema file at {}: {}",
                schema_sql_path, e
            )
        })?;

        // If schema is not present, apply full authoritative schema (first run).
        if !components_table_exists || !component_type_exists {
            let sql = compile_authoritative_schema_for_postgres(&sql_raw);
            info!(
                "Applying authoritative DB schema (first-run) from {} ({} bytes)",
                schema_sql_path,
                sql.len()
            );

            self.client
                .batch_execute(&sql)
                .await
                .map_err(|e| format!("FAIL-CLOSED: Schema apply failed: {:?}", e))?;

            // Re-assert search_path after schema apply (schema creation can occur during apply).
            self.client
                .batch_execute("SET search_path = ransomeye, public;")
                .await
                .map_err(|e| format!("Failed to set search_path after schema apply: {e}"))?;

            return Ok(());
        }

        // Schema exists: apply incremental completion if any REQUIRED table is missing.
        let required_tables = vec![
            // A. Agent Telemetry
            "linux_agent_telemetry",
            "windows_agent_telemetry",
            "dpi_probe_telemetry",
            // B. Ingestion & Normalization
            "raw_events",
            "normalized_events",
            // C. Correlation & Detection
            "correlation_graph",
            "detection_results",
            "confidence_scores",
            // D. Policy & Enforcement
            "policy_evaluations",
            "enforcement_decisions",
            "actions_taken",
            // E. AI / ML / LLM
            "model_registry",
            "model_versions",
            "inference_results",
            "shap_explanations",
            "feature_contributions",
            "llm_requests",
            "llm_responses",
            // F. Audit & Forensics
            "immutable_audit_log",
            "trust_verification_records",
            "signature_validation_events",
            // G. System Health & Ops
            "component_health",
            "startup_events",
            "error_events",
            // Supporting contract tables required by Core runtime writes
            "components",
            // PROMPT-25/29B: retention policy configuration table is MANDATORY
            "retention_policies",
        ];

        let existing_tables = self
            .client
            .query(
                r#"
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'ransomeye'
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("Failed to query information_schema.tables for incremental apply: {e}"))?;

        let mut existing: HashSet<String> = HashSet::new();
        for r in existing_tables {
            let name: String = r.get(0);
            existing.insert(name);
        }

        let mut missing: Vec<&str> = Vec::new();
        for t in &required_tables {
            if !existing.contains(&t.to_string()) {
                missing.push(t);
            }
        }

        if missing.is_empty() {
            info!("Authoritative schema already present (including required tables); skipping schema apply");
            // Ensure queries resolve into ransomeye schema without explicit prefixes.
            self.client
                .batch_execute("SET search_path = ransomeye, public;")
                .await
                .map_err(|e| format!("Failed to set search_path: {e}"))?;
            return Ok(());
        }

        info!(
            "Schema present but incomplete; applying incremental authoritative DDL for missing tables: {}",
            missing.join(", ")
        );

        let patch_sql = build_incremental_schema_patch_for_missing_tables(&sql_raw, &missing)
            .map_err(|e| format!("FAIL-CLOSED: Failed to build incremental schema patch: {e}"))?;

        self.client
            .batch_execute(&patch_sql)
            .await
            .map_err(|e| format!("FAIL-CLOSED: Incremental schema apply failed: {:?}", e))?;

        // Re-assert search_path after patch apply.
        self.client
            .batch_execute("SET search_path = ransomeye, public;")
            .await
            .map_err(|e| format!("Failed to set search_path after incremental apply: {e}"))?;

        Ok(())
    }

    /// Validate required tables exist (full contract list) and required columns exist (core-critical tables).
    pub async fn validate_schema_contract(&self) -> Result<(), String> {
        info!("Validating authoritative DB schema contract...");

        // 1) Required tables (PROMPT-21 contract)
        let required_tables = vec![
            // A. Agent Telemetry
            "linux_agent_telemetry",
            "windows_agent_telemetry",
            "dpi_probe_telemetry",
            // B. Ingestion & Normalization
            "raw_events",
            "normalized_events",
            // C. Correlation & Detection
            "correlation_graph",
            "detection_results",
            "confidence_scores",
            // D. Policy & Enforcement
            "policy_evaluations",
            "enforcement_decisions",
            "actions_taken",
            // E. AI / ML / LLM
            "model_registry",
            "model_versions",
            "inference_results",
            "shap_explanations",
            "feature_contributions",
            "llm_requests",
            "llm_responses",
            // F. Audit & Forensics
            "immutable_audit_log",
            "trust_verification_records",
            "signature_validation_events",
            // G. System Health & Ops
            "component_health",
            "startup_events",
            "error_events",
            // Supporting contract tables required by Core runtime writes
            "components",
            // PROMPT-25/29B: retention policy configuration table is MANDATORY
            "retention_policies",
        ];

        let existing_tables = self
            .client
            .query(
                r#"
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'ransomeye'
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("Schema validation failed querying information_schema.tables: {e}"))?;

        let mut existing: HashSet<String> = HashSet::new();
        for r in existing_tables {
            let name: String = r.get(0);
            existing.insert(name);
        }

        let mut missing_tables: Vec<&str> = Vec::new();
        for t in &required_tables {
            if !existing.contains(&t.to_string()) {
                missing_tables.push(t);
            }
        }
        if !missing_tables.is_empty() {
            return Err(format!(
                "FAIL-CLOSED: Authoritative schema validation failed. Missing required tables in schema 'ransomeye': {}",
                missing_tables.join(", ")
            ));
        }

        // 2) Core-critical required columns (must exist for mandatory writes)
        let required_columns: HashMap<&'static str, Vec<&'static str>> = HashMap::from([
            (
                "components",
                vec![
                    "component_id",
                    "component_type",
                    "component_name",
                    "instance_id",
                    "build_hash",
                    "version",
                    "started_at",
                    "last_heartbeat_at",
                    "created_at",
                    "updated_at",
                ],
            ),
            (
                "startup_events",
                vec![
                    "startup_event_id",
                    "created_at",
                    "component_id",
                    "started_at",
                    "boot_reason",
                    "config_sha256",
                    "build_hash",
                    "version",
                    "env_fingerprint_sha256",
                    "details_json",
                ],
            ),
            (
                "component_health",
                vec![
                    "health_id",
                    "created_at",
                    "component_id",
                    "observed_at",
                    "status",
                    "status_details",
                    "metrics_json",
                ],
            ),
            (
                "error_events",
                vec![
                    "error_event_id",
                    "created_at",
                    "component_id",
                    "agent_id",
                    "observed_at",
                    "severity",
                    "error_type",
                    "error_message",
                    "stacktrace",
                    "context_json",
                    "trace_id",
                    "correlation_hint",
                ],
            ),
            (
                "immutable_audit_log",
                vec![
                    "audit_id",
                    "created_at",
                    "actor_component_id",
                    "actor_agent_id",
                    "action",
                    "object_type",
                    "object_id",
                    "event_time",
                    "payload_json",
                    "payload_sha256",
                    "prev_audit_id",
                    "prev_payload_sha256",
                    "chain_hash_sha256",
                    "signature_status",
                    "signed_by",
                    "signature_alg",
                    "signature_b64",
                ],
            ),
            (
                "retention_policies",
                vec![
                    "table_name",
                    "retention_days",
                    "retention_enabled",
                    "created_at",
                    "updated_at",
                ],
            ),
        ]);

        for (table, cols) in required_columns {
            let rows = self
                .client
                .query(
                    r#"
                    SELECT column_name
                    FROM information_schema.columns
                    WHERE table_schema = 'ransomeye' AND table_name = $1
                    "#,
                    &[&table],
                )
                .await
                .map_err(|e| format!("Schema validation failed querying information_schema.columns for {table}: {e}"))?;

            let mut colset: HashSet<String> = HashSet::new();
            for r in rows {
                let c: String = r.get(0);
                colset.insert(c);
            }

            let mut missing_cols: Vec<&str> = Vec::new();
            for c in cols {
                if !colset.contains(&c.to_string()) {
                    missing_cols.push(c);
                }
            }
            if !missing_cols.is_empty() {
                return Err(format!(
                    "FAIL-CLOSED: Schema validation failed for table ransomeye.{table}. Missing required columns: {}",
                    missing_cols.join(", ")
                ));
            }
        }

        info!("Schema validation passed (required tables present; core-critical columns present)");
        Ok(())
    }

    /// Upsert the orchestrator into ransomeye.components and return its component_id (FK anchor for core runtime tables).
    pub async fn upsert_component(
        &self,
        component_type: &str,
        component_name: &str,
        instance_id: Option<&str>,
        build_hash: Option<&str>,
        version: Option<&str>,
    ) -> Result<Uuid, String> {
        let row = self
            .client
            .query_one(
                r#"
                INSERT INTO components (
                    component_type, component_name, instance_id, build_hash, version, started_at, last_heartbeat_at
                )
                VALUES ($1::text::component_type, $2, $3, $4, $5, NOW(), NOW())
                ON CONFLICT (component_type, component_name, (COALESCE(instance_id, '')))
                DO UPDATE SET
                    build_hash = COALESCE(EXCLUDED.build_hash, components.build_hash),
                    version = COALESCE(EXCLUDED.version, components.version),
                    last_heartbeat_at = NOW()
                RETURNING component_id
                "#,
                &[&component_type, &component_name, &instance_id, &build_hash, &version],
            )
            .await
            .map_err(|e| format!("Failed to upsert components row: {e}"))?;

        Ok(row.get::<usize, Uuid>(0))
    }

    pub async fn insert_startup_event(
        &self,
        component_id: Uuid,
        started_at: DateTime<Utc>,
        boot_reason: Option<&str>,
        build_hash: Option<&str>,
        version: Option<&str>,
        env_fingerprint_sha256: Option<&[u8]>,
        details_json: Option<&JsonValue>,
    ) -> Result<Uuid, String> {
        let row = self
            .client
            .query_one(
                r#"
                INSERT INTO startup_events (
                    component_id, started_at, boot_reason, build_hash, version, env_fingerprint_sha256, details_json
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING startup_event_id
                "#,
                &[
                    &component_id,
                    &started_at,
                    &boot_reason,
                    &build_hash,
                    &version,
                    &env_fingerprint_sha256,
                    &details_json,
                ],
            )
            .await
            .map_err(|e| format!("Failed to insert startup_events row: {e}"))?;

        Ok(row.get::<usize, Uuid>(0))
    }

    pub async fn insert_component_health(
        &self,
        component_id: Uuid,
        status: &str,
        status_details: Option<&str>,
        metrics_json: Option<&JsonValue>,
    ) -> Result<Uuid, String> {
        let row = self
            .client
            .query_one(
                r#"
                INSERT INTO component_health (
                    component_id, observed_at, status, status_details, metrics_json
                )
                VALUES ($1, NOW(), $2, $3, $4)
                RETURNING health_id
                "#,
                &[&component_id, &status, &status_details, &metrics_json],
            )
            .await
            .map_err(|e| format!("Failed to insert component_health row: {e}"))?;

        Ok(row.get::<usize, Uuid>(0))
    }

    pub async fn insert_error_event(
        &self,
        component_id: Option<Uuid>,
        severity: &str,
        error_type: &str,
        error_message: &str,
        stacktrace: Option<&str>,
        context_json: Option<&JsonValue>,
        trace_id: Option<&str>,
        correlation_hint: Option<&str>,
    ) -> Result<Uuid, String> {
        let row = self
            .client
            .query_one(
                r#"
                INSERT INTO error_events (
                    component_id, agent_id, observed_at, severity, error_type, error_message,
                    stacktrace, context_json, trace_id, correlation_hint
                )
                VALUES ($1, NULL, NOW(), $2::text::severity_level, $3, $4, $5, $6, $7, $8)
                RETURNING error_event_id
                "#,
                &[
                    &component_id,
                    &severity,
                    &error_type,
                    &error_message,
                    &stacktrace,
                    &context_json,
                    &trace_id,
                    &correlation_hint,
                ],
            )
            .await
            .map_err(|e| format!("Failed to insert error_events row: {e}"))?;

        Ok(row.get::<usize, Uuid>(0))
    }

    fn sha256_bytes(input: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let out = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&out[..]);
        arr
    }

    async fn fetch_last_audit_chain(&self) -> Result<Option<(Uuid, [u8; 32], [u8; 32])>, String> {
        let row = self
            .client
            .query_opt(
                r#"
                SELECT audit_id, chain_hash_sha256, payload_sha256
                FROM immutable_audit_log
                ORDER BY created_at DESC
                LIMIT 1
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("Failed to query last immutable_audit_log row: {e}"))?;

        Ok(row.map(|r| {
            let audit_id: Uuid = r.get(0);
            let chain_hash: Vec<u8> = r.get(1);
            let payload_hash: Vec<u8> = r.get(2);
            let mut ch = [0u8; 32];
            let mut ph = [0u8; 32];
            ch.copy_from_slice(&chain_hash[..32]);
            ph.copy_from_slice(&payload_hash[..32]);
            (audit_id, ch, ph)
        }))
    }

    pub async fn insert_immutable_audit_log(
        &self,
        actor_component_id: Option<Uuid>,
        action: &str,
        object_type: &str,
        object_id: Option<Uuid>,
        payload_json: &JsonValue,
    ) -> Result<Uuid, String> {
        // Deterministic JSON string (field order fixed by construction at callsites).
        let payload_str = serde_json::to_string(payload_json)
            .map_err(|e| format!("Failed to serialize audit payload JSON: {e}"))?;
        let payload_sha256 = Self::sha256_bytes(payload_str.as_bytes());

        let (prev_audit_id, prev_payload_sha256, prev_chain_hash) = match self.fetch_last_audit_chain().await? {
            Some((aid, chain_hash, payload_hash)) => (Some(aid), Some(payload_hash), chain_hash),
            None => (None, None, [0u8; 32]),
        };

        // Chain hash = SHA256(prev_chain_hash || payload_sha256)
        let mut chain_input = Vec::with_capacity(64);
        chain_input.extend_from_slice(&prev_chain_hash);
        chain_input.extend_from_slice(&payload_sha256);
        let chain_hash_sha256 = Self::sha256_bytes(&chain_input);

        let payload_sha_vec: Vec<u8> = payload_sha256.to_vec();
        let prev_payload_vec: Option<Vec<u8>> = prev_payload_sha256.map(|x| x.to_vec());
        let chain_hash_vec: Vec<u8> = chain_hash_sha256.to_vec();

        let row = self
            .client
            .query_one(
                r#"
                INSERT INTO immutable_audit_log (
                    actor_component_id, actor_agent_id, action, object_type, object_id, event_time,
                    payload_json, payload_sha256, prev_audit_id, prev_payload_sha256, chain_hash_sha256, signature_status
                )
                VALUES (
                    $1, NULL, $2, $3::text::trust_object_type, $4, NOW(),
                    $5, $6, $7, $8, $9, 'unknown'
                )
                RETURNING audit_id
                "#,
                &[
                    &actor_component_id,
                    &action,
                    &object_type,
                    &object_id,
                    &payload_json,
                    &payload_sha_vec,
                    &prev_audit_id,
                    &prev_payload_vec,
                    &chain_hash_vec,
                ],
            )
            .await
            .map_err(|e| format!("Failed to insert immutable_audit_log row: {e}"))?;

        Ok(row.get::<usize, Uuid>(0))
    }
}

/// Build an incremental schema patch for a set of missing tables using ONLY the authoritative schema source.
///
/// FAIL-CLOSED:
/// - If we cannot extract a table block, we error.
/// - We do not attempt to re-run CREATE TYPE statements (unsafe on initialized DBs).
fn build_incremental_schema_patch_for_missing_tables(schema_sql: &str, missing_tables: &[&str]) -> Result<String, String> {
    let mut blocks: Vec<String> = Vec::new();
    for table in missing_tables {
        let block = extract_table_ddl_block(schema_sql, table)
            .map_err(|e| format!("missing table '{table}' extraction failed: {e}"))?;
        blocks.push(block);
    }
    Ok(blocks.join("\n\n"))
}

/// Extract a CREATE TABLE IF NOT EXISTS block (and immediately following related statements) for a named table.
///
/// We intentionally keep this extractor conservative:
/// - It looks for a line that starts with `CREATE TABLE IF NOT EXISTS <table>` (no schema prefix)
/// - Captures until the terminating `);`
/// - Then captures contiguous related DDL lines for that table (COMMENT/INDEX/ALTER) until a blank-line+new section/table.
fn extract_table_ddl_block(schema_sql: &str, table: &str) -> Result<String, String> {
    let lines: Vec<&str> = schema_sql.lines().collect();
    let needle = format!("CREATE TABLE IF NOT EXISTS {table} ");

    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&needle) {
            start_idx = Some(i);
            break;
        }
    }

    let start = start_idx.ok_or_else(|| format!("CREATE TABLE block not found for {table}"))?;

    // Capture from CREATE TABLE line through the closing `);`
    let mut out: Vec<String> = Vec::new();
    let mut i = start;
    let mut saw_table_end = false;
    while i < lines.len() {
        let line = lines[i];
        out.push(line.to_string());
        if line.trim() == ");" {
            saw_table_end = true;
            i += 1;
            break;
        }
        i += 1;
    }

    if !saw_table_end {
        return Err(format!("Did not find end of CREATE TABLE statement for {table}"));
    }

    // Capture immediately following related statements for this table.
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Stop at next section/table header.
        if trimmed.starts_with("--") && trimmed.contains("====") {
            break;
        }
        if trimmed.starts_with("CREATE TABLE IF NOT EXISTS ") {
            break;
        }

        // Skip pure blank lines but keep a single separator if we've already collected some related lines.
        if trimmed.is_empty() {
            // Lookahead: if next non-empty line is a new section/table, stop.
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j >= lines.len() {
                break;
            }
            let next = lines[j].trim();
            if next.starts_with("--") && next.contains("====") {
                break;
            }
            if next.starts_with("CREATE TABLE IF NOT EXISTS ") {
                break;
            }
            // Otherwise keep a single blank line and continue.
            out.push(String::new());
            i = j;
            continue;
        }

        // COMMENT blocks may span multiple lines (often multiple string literal lines) until a trailing ';'.
        if trimmed.starts_with(&format!("COMMENT ON TABLE {table}"))
            || trimmed.starts_with(&format!("COMMENT ON COLUMN {table}."))
        {
            out.push(line.to_string());
            i += 1;
            while i < lines.len() {
                let l2 = lines[i];
                out.push(l2.to_string());
                let done = l2.trim_end().ends_with(';');
                i += 1;
                if done {
                    break;
                }
            }
            continue;
        }

        // Include only statements that clearly target this table.
        if trimmed.starts_with("CREATE INDEX IF NOT EXISTS") && trimmed.contains(&format!(" ON {table}")) {
            out.push(line.to_string());
            i += 1;
            continue;
        }
        if trimmed.starts_with(&format!("ALTER TABLE {table}")) {
            out.push(line.to_string());
            i += 1;
            continue;
        }

        // Stop if we hit some other unrelated statement after the table block.
        if trimmed.starts_with("CREATE ") || trimmed.starts_with("ALTER ") || trimmed.starts_with("DROP ") {
            break;
        }

        // Otherwise, ignore non-DDL noise.
        i += 1;
    }

    Ok(out.join("\n"))
}

/// Compile-time normalization of the authoritative schema for PostgreSQL compatibility
/// WITHOUT modifying the on-disk schema file.
///
/// PostgreSQL does not permit UNIQUE *constraints* on expressions (e.g., COALESCE(...)),
/// but it does permit UNIQUE *indexes* with expressions. The schema contract uses a
/// UNIQUE constraint name for such cases; we rewrite those constraint lines into
/// `CREATE UNIQUE INDEX IF NOT EXISTS <constraint_name> ON <table> (...)` immediately
/// after the table definition, preserving semantics and idempotency.
fn compile_authoritative_schema_for_postgres(sql: &str) -> String {
    let mut out: Vec<String> = Vec::new();

    let mut in_create_table: bool = false;
    let mut current_table: Option<String> = None;
    let mut table_block: Vec<String> = Vec::new();
    let mut deferred_unique_indexes: Vec<(String, String, String)> = Vec::new(); // (table, index_name, expr_list)

    for line in sql.lines() {
        let trimmed = line.trim();

        if !in_create_table
            && trimmed.to_uppercase().starts_with("CREATE TABLE IF NOT EXISTS ")
            && trimmed.ends_with('(')
        {
            in_create_table = true;
            table_block.clear();

            // Parse table name: "CREATE TABLE IF NOT EXISTS <name> ("
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            current_table = if parts.len() >= 6 {
                Some(parts[5].to_string())
            } else {
                None
            };

            table_block.push(line.to_string());
            continue;
        }

        if in_create_table {
            // Rewrite invalid UNIQUE constraints with COALESCE(...) into unique indexes.
            if trimmed.starts_with("CONSTRAINT ") && trimmed.contains(" UNIQUE ") && trimmed.contains("COALESCE(") {
                let table = current_table.clone().unwrap_or_else(|| "unknown_table".to_string());
                let after_constraint = trimmed.strip_prefix("CONSTRAINT ").unwrap_or(trimmed);
                let mut it = after_constraint.splitn(2, ' ');
                let constraint_name = it.next().unwrap_or("unknown_unique").to_string();
                let remainder = it.next().unwrap_or("").trim().trim_end_matches(',').trim();

                // Extract the outer (...) list for UNIQUE while preserving inner parentheses (e.g., COALESCE()).
                // Example remainder: "UNIQUE (component_type, component_name, COALESCE(instance_id, ''))"
                let upper = remainder.to_uppercase();
                let unique_pos = upper.find("UNIQUE").unwrap_or(0);
                let after_unique = &remainder[unique_pos..];
                let paren_start_rel = after_unique.find('(').unwrap_or(0);
                let paren_start = unique_pos + paren_start_rel;
                let paren_end = remainder.rfind(')').unwrap_or(remainder.len().saturating_sub(1));
                let expr_list = if paren_end > paren_start {
                    remainder[paren_start + 1..paren_end].trim().to_string()
                } else {
                    String::new()
                };

                deferred_unique_indexes.push((table, constraint_name, expr_list));
                continue;
            }

            // End of CREATE TABLE block.
            if trimmed == ");" {
                // Strip a trailing comma from the last non-empty line in the table block.
                for idx in (0..table_block.len()).rev() {
                    let l = table_block[idx].trim_end();
                    if l.is_empty() {
                        continue;
                    }
                    if l.ends_with(',') {
                        let without = l.trim_end_matches(',');
                        // Preserve original indentation prefix from the stored line.
                        let prefix_len = table_block[idx].len() - table_block[idx].trim_start().len();
                        let indent = " ".repeat(prefix_len);
                        table_block[idx] = format!("{}{}", indent, without.trim_start());
                    }
                    break;
                }

                table_block.push(line.to_string());

                // Emit the table block to output.
                out.extend(table_block.drain(..));

                // Emit deferred unique indexes for this table.
                if let Some(table) = current_table.take() {
                    let mut remaining: Vec<(String, String, String)> = Vec::new();
                    for (t, idx, exprs) in deferred_unique_indexes.drain(..) {
                        if t == table {
                            out.push(format!(
                                "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} ({});",
                                idx, table, exprs
                            ));
                        } else {
                            remaining.push((t, idx, exprs));
                        }
                    }
                    deferred_unique_indexes = remaining;
                }

                in_create_table = false;
                continue;
            }

            table_block.push(line.to_string());
            continue;
        }

        out.push(line.to_string());
    }

    // If file ended mid-table, flush what we have (should not happen).
    if in_create_table {
        out.extend(table_block);
    }

    out.join("\n")
}


