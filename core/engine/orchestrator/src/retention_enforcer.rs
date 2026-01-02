// Path and File Name : /home/ransomeye/rebuild/core/engine/orchestrator/src/retention_enforcer.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Runtime DB retention enforcer (purge-only) with fail-closed validation and immutable audit logging.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use tokio_postgres::Row;
use tracing::info;
use uuid::Uuid;

use super::db::CoreDb;

const DENYLIST_TABLES: &[&str] = &[
    "ransomeye.immutable_audit_log",
    "ransomeye.trust_verification_records",
    "ransomeye.signature_validation_events",
    "ransomeye.retention_policies",
];

const ALLOWED_SCHEMAS: &[&str] = &["ransomeye", "public"];

const CANDIDATE_TIME_COLUMNS: &[&str] = &[
    // Preferred
    "created_at",
    // Common telemetry/event time variants
    "observed_at",
    "event_time",
    "received_at",
    // Common ops/health time variants
    "last_seen_at",
    "first_seen_at",
    // Some public tables use this (often quoted in DDL, but appears as `timestamp` in information_schema)
    "timestamp",
];

#[derive(Debug, Clone)]
pub struct RetentionEnforcerConfig {
    pub batch_size: i64,
    pub max_batches_per_table: i64,
    pub sleep_ms_between_batches: i64,
}

impl RetentionEnforcerConfig {
    pub fn from_env() -> Result<Self, String> {
        let batch_size = env_i64("RANSOMEYE_RETENTION_BATCH_SIZE", 1000)?;
        if batch_size <= 0 {
            return Err("FAIL-CLOSED: RANSOMEYE_RETENTION_BATCH_SIZE must be > 0".to_string());
        }

        let max_batches_per_table = env_i64("RANSOMEYE_RETENTION_MAX_BATCHES_PER_TABLE", 200)?;
        if max_batches_per_table <= 0 {
            return Err("FAIL-CLOSED: RANSOMEYE_RETENTION_MAX_BATCHES_PER_TABLE must be > 0".to_string());
        }

        let sleep_ms_between_batches = env_i64("RANSOMEYE_RETENTION_SLEEP_MS_BETWEEN_BATCHES", 0)?;
        if sleep_ms_between_batches < 0 {
            return Err("FAIL-CLOSED: RANSOMEYE_RETENTION_SLEEP_MS_BETWEEN_BATCHES must be >= 0".to_string());
        }

        Ok(Self {
            batch_size,
            max_batches_per_table,
            sleep_ms_between_batches,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedTable {
    pub schema: String,
    pub table: String,
}

impl QualifiedTable {
    pub fn as_fqn(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }

    pub fn quote_ident(ident: &str) -> Result<String, String> {
        // Fail-closed: strict identifier contract; no quotes, dots, or whitespace allowed.
        // We only accept [A-Za-z_][A-Za-z0-9_]* and quote it for SQL.
        if ident.is_empty() {
            return Err("FAIL-CLOSED: empty identifier".to_string());
        }
        let mut chars = ident.chars();
        let first = chars.next().unwrap();
        if !(first == '_' || first.is_ascii_alphabetic()) {
            return Err(format!("FAIL-CLOSED: illegal identifier (first char): '{ident}'"));
        }
        for c in chars {
            if !(c == '_' || c.is_ascii_alphanumeric()) {
                return Err(format!("FAIL-CLOSED: illegal identifier char in '{ident}'"));
            }
        }
        Ok(format!("\"{ident}\""))
    }

    pub fn parse(fqn: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fqn.split('.').collect();
        if parts.len() != 2 {
            return Err(format!(
                "FAIL-CLOSED: retention_policies.table_name must be 'schema.table' (got '{fqn}')"
            ));
        }

        let schema = parts[0].trim();
        let table = parts[1].trim();

        if !ALLOWED_SCHEMAS.contains(&schema) {
            return Err(format!(
                "FAIL-CLOSED: Illegal schema '{schema}' in retention_policies (allowed: {})",
                ALLOWED_SCHEMAS.join(", ")
            ));
        }

        // Validate identifier shape (fail-closed, prevents SQL injection via table_name).
        let _ = Self::quote_ident(schema)?;
        let _ = Self::quote_ident(table)?;

        Ok(Self {
            schema: schema.to_string(),
            table: table.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TableRetentionResult {
    pub table: QualifiedTable,
    pub retention_days: i64,
    pub time_column: String,
    pub cutoff: DateTime<Utc>,
    pub eligible: bool,
    pub reason_not_eligible: Option<String>,
    pub dry_run_rows_older: Option<i64>,
    pub deleted_rows: i64,
    pub batches_executed: i64,
}

pub struct RetentionEnforcer {
    cfg: RetentionEnforcerConfig,
}

impl RetentionEnforcer {
    pub fn new(cfg: RetentionEnforcerConfig) -> Self {
        Self { cfg }
    }

    pub fn new_from_env() -> Result<Self, String> {
        Ok(Self::new(RetentionEnforcerConfig::from_env()?))
    }

    pub async fn enforce(
        &self,
        db: &CoreDb,
        actor_component_id: Option<Uuid>,
        dry_run: bool,
    ) -> Result<(Uuid, Vec<TableRetentionResult>), String> {
        let run_id = Uuid::new_v4();
        let started_at = Utc::now();

        // Fail-closed: retention_policies MUST exist and MUST have enabled rows.
        let policies = self.fetch_enabled_policies(db).await?;
        if policies.is_empty() {
            return Err("FAIL-CLOSED: No retention_policies rows with retention_enabled=true".to_string());
        }

        // Fail-closed: denylist must never be targeted (even if policy exists).
        for (qt, _) in &policies {
            if DENYLIST_TABLES.contains(&qt.as_fqn().as_str()) {
                return Err(format!(
                    "FAIL-CLOSED: Illegal retention target '{}' (immutable/protected table)",
                    qt.as_fqn()
                ));
            }
        }

        // Fail-closed: never touch append-only protected tables.
        let append_only = self.fetch_append_only_tables(db).await?;
        for (qt, _) in &policies {
            if append_only.contains(&qt.as_fqn()) {
                return Err(format!(
                    "FAIL-CLOSED: Illegal retention target '{}' (append-only trigger protected)",
                    qt.as_fqn()
                ));
            }
        }

        let mut results: Vec<TableRetentionResult> = Vec::new();
        for (qt, retention_days) in policies {
            let res = self.enforce_one_table(db, &append_only, &qt, retention_days, dry_run).await?;
            results.push(res);
        }

        let ended_at = Utc::now();
        let payload = build_audit_payload(run_id, started_at, ended_at, dry_run, &self.cfg, &results);
        let audit_id = db
            .insert_immutable_audit_log(
                actor_component_id,
                if dry_run {
                    "runtime_retention_dry_run"
                } else {
                    "runtime_retention_enforcement"
                },
                "other",
                actor_component_id,
                &payload,
            )
            .await?;

        Ok((audit_id, results))
    }

    async fn fetch_enabled_policies(&self, db: &CoreDb) -> Result<Vec<(QualifiedTable, i64)>, String> {
        // Log DB name and search_path for debugging
        let db_name_row = db
            .client()
            .query_one("SELECT current_database()", &[])
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot query current_database(): {e}"))?;
        let db_name: String = db_name_row.get(0);

        let search_path_row = db
            .client()
            .query_one("SELECT current_setting('search_path')", &[])
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot query search_path: {e}"))?;
        let search_path: String = search_path_row.get(0);

        // Explicitly query ransomeye.retention_policies to avoid search_path ambiguity
        let query = r#"
                SELECT table_name, retention_days
                FROM ransomeye.retention_policies
                WHERE retention_enabled = TRUE
                ORDER BY table_name
                "#;

        info!(
            "[RETENTION] Querying retention policies: db_name={}, search_path={}, query={}",
            db_name, search_path, query.trim()
        );

        let rows = db
            .client()
            .query(query, &[])
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot read ransomeye.retention_policies: {e}"))?;

        let mut out: Vec<(QualifiedTable, i64)> = Vec::new();
        for r in rows {
            let table_name: String = r.get(0);
            let retention_days: i64 = r.get::<usize, i32>(1) as i64;
            let qt = QualifiedTable::parse(&table_name)?;
            out.push((qt, retention_days));
        }

        info!(
            "[RETENTION] Found {} enabled retention policy row(s)",
            out.len()
        );

        Ok(out)
    }

    async fn fetch_append_only_tables(&self, db: &CoreDb) -> Result<HashSet<String>, String> {
        let rows = db
            .client()
            .query(
                r#"
                SELECT DISTINCT n.nspname AS table_schema, c.relname AS table_name
                FROM pg_trigger t
                JOIN pg_class c ON c.oid = t.tgrelid
                JOIN pg_namespace n ON n.oid = c.relnamespace
                JOIN pg_proc p ON p.oid = t.tgfoid
                WHERE NOT t.tgisinternal
                  AND p.proname = 'prevent_update_delete'
                "#,
                &[],
            )
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot discover append-only protected tables: {e}"))?;

        let mut set: HashSet<String> = HashSet::new();
        for r in rows {
            let schema: String = r.get(0);
            let table: String = r.get(1);
            set.insert(format!("{schema}.{table}"));
        }
        Ok(set)
    }

    async fn enforce_one_table(
        &self,
        db: &CoreDb,
        append_only: &HashSet<String>,
        qt: &QualifiedTable,
        retention_days: i64,
        dry_run: bool,
    ) -> Result<TableRetentionResult, String> {
        let started = Utc::now();

        // Guard: even if the global check passed, re-check per-table (defense-in-depth).
        if DENYLIST_TABLES.contains(&qt.as_fqn().as_str()) {
            return Err(format!(
                "FAIL-CLOSED: Illegal retention target '{}' (immutable/protected table)",
                qt.as_fqn()
            ));
        }
        if append_only.contains(&qt.as_fqn()) {
            return Err(format!(
                "FAIL-CLOSED: Illegal retention target '{}' (append-only trigger protected)",
                qt.as_fqn()
            ));
        }

        // Determine time column used for retention cutoff.
        let time_col = self.find_time_column(db, qt).await?;

        // Compute cutoff timestamp deterministically from NOW() in DB, but also provide a local approximation for reporting.
        let cutoff = Utc::now() - chrono::Duration::days(retention_days);

        let mut result = TableRetentionResult {
            table: qt.clone(),
            retention_days,
            time_column: time_col.clone(),
            cutoff,
            eligible: true,
            reason_not_eligible: None,
            dry_run_rows_older: None,
            deleted_rows: 0,
            batches_executed: 0,
        };

        // Dry-run: counts only (no deletes).
        let rows_older = self.count_rows_older_than_cutoff(db, qt, &time_col, retention_days).await?;
        result.dry_run_rows_older = Some(rows_older);

        if dry_run {
            info!(
                "[RETENTION][DRY-RUN] {} rows eligible for purge in {} (retention_days={}, col={})",
                rows_older,
                qt.as_fqn(),
                retention_days,
                time_col
            );
            return Ok(result);
        }

        // Live run: bounded batches.
        if rows_older == 0 {
            info!(
                "[RETENTION] No rows to purge in {} (retention_days={}, col={})",
                qt.as_fqn(),
                retention_days,
                time_col
            );
            return Ok(result);
        }

        let mut total_deleted: i64 = 0;
        let mut batches: i64 = 0;
        for _ in 0..self.cfg.max_batches_per_table {
            let deleted = self
                .delete_batch(db, qt, &time_col, retention_days, self.cfg.batch_size)
                .await?;
            batches += 1;
            total_deleted += deleted;

            if deleted == 0 {
                break;
            }

            if self.cfg.sleep_ms_between_batches > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    self.cfg.sleep_ms_between_batches as u64,
                ))
                .await;
            }
        }

        result.deleted_rows = total_deleted;
        result.batches_executed = batches;

        let elapsed_ms = (Utc::now() - started).num_milliseconds();
        info!(
            "[RETENTION] Purged {} row(s) from {} in {} batch(es) ({} ms)",
            total_deleted,
            qt.as_fqn(),
            batches,
            elapsed_ms
        );

        Ok(result)
    }

    async fn find_time_column(&self, db: &CoreDb, qt: &QualifiedTable) -> Result<String, String> {
        // Fail-closed: ensure table exists.
        let exists = db
            .client()
            .query_opt(
                r#"
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = $1 AND table_name = $2 AND table_type = 'BASE TABLE'
                LIMIT 1
                "#,
                &[&qt.schema, &qt.table],
            )
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot probe table existence for {}: {e}", qt.as_fqn()))?
            .is_some();
        if !exists {
            return Err(format!(
                "FAIL-CLOSED: retention_policies references non-existent table '{}'",
                qt.as_fqn()
            ));
        }

        let rows: Vec<Row> = db
            .client()
            .query(
                r#"
                SELECT column_name, data_type
                FROM information_schema.columns
                WHERE table_schema = $1 AND table_name = $2
                "#,
                &[&qt.schema, &qt.table],
            )
            .await
            .map_err(|e| format!("FAIL-CLOSED: Cannot read columns for {}: {e}", qt.as_fqn()))?;

        let mut by_name: HashMap<String, String> = HashMap::new();
        for r in rows {
            let col: String = r.get(0);
            let dtype: String = r.get(1);
            by_name.insert(col, dtype);
        }

        for cand in CANDIDATE_TIME_COLUMNS {
            if let Some(dtype) = by_name.get(*cand) {
                let dtype_l = dtype.to_lowercase();
                if dtype_l.contains("timestamp") || dtype_l.contains("date") {
                    return Ok(cand.to_string());
                }
            }
        }

        Err(format!(
            "FAIL-CLOSED: Table '{}' has no acceptable time column for retention (tried: {})",
            qt.as_fqn(),
            CANDIDATE_TIME_COLUMNS.join(", ")
        ))
    }

    async fn count_rows_older_than_cutoff(
        &self,
        db: &CoreDb,
        qt: &QualifiedTable,
        time_col: &str,
        retention_days: i64,
    ) -> Result<i64, String> {
        let schema_q = QualifiedTable::quote_ident(&qt.schema)?;
        let table_q = QualifiedTable::quote_ident(&qt.table)?;
        let col_q = QualifiedTable::quote_ident(time_col)?;

        let sql = format!(
            "SELECT COUNT(*)::bigint FROM {schema}.{table} WHERE {col} < (NOW() - ($1::int * INTERVAL '1 day'))",
            schema = schema_q,
            table = table_q,
            col = col_q
        );

        let row = db
            .client()
            .query_one(&sql, &[&(retention_days as i32)])
            .await
            .map_err(|e| format!("FAIL-CLOSED: Count query failed for {}: {e}", qt.as_fqn()))?;
        Ok(row.get::<usize, i64>(0))
    }

    async fn delete_batch(
        &self,
        db: &CoreDb,
        qt: &QualifiedTable,
        time_col: &str,
        retention_days: i64,
        batch_size: i64,
    ) -> Result<i64, String> {
        let schema_q = QualifiedTable::quote_ident(&qt.schema)?;
        let table_q = QualifiedTable::quote_ident(&qt.table)?;
        let col_q = QualifiedTable::quote_ident(time_col)?;

        let sql = format!(
            r#"
            WITH todel AS (
                SELECT ctid
                FROM {schema}.{table}
                WHERE {col} < (NOW() - ($1::int * INTERVAL '1 day'))
                ORDER BY {col} ASC
                LIMIT $2
            )
            DELETE FROM {schema}.{table} t
            USING todel
            WHERE t.ctid = todel.ctid
            RETURNING 1
            "#,
            schema = schema_q,
            table = table_q,
            col = col_q
        );

        let rows = db
            .client()
            .query(&sql, &[&(retention_days as i32), &(batch_size as i64)])
            .await
            .map_err(|e| format!("FAIL-CLOSED: Delete batch failed for {}: {e}", qt.as_fqn()))?;
        Ok(rows.len() as i64)
    }
}

fn env_i64(key: &str, default_value: i64) -> Result<i64, String> {
    match std::env::var(key) {
        Ok(v) => v
            .parse::<i64>()
            .map_err(|e| format!("FAIL-CLOSED: Invalid {key}='{v}': {e}")),
        Err(_) => Ok(default_value),
    }
}

fn build_audit_payload(
    run_id: Uuid,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    dry_run: bool,
    cfg: &RetentionEnforcerConfig,
    results: &[TableRetentionResult],
) -> JsonValue {
    let mut per_table: Vec<JsonValue> = Vec::new();
    for r in results {
        per_table.push(serde_json::json!({
            "table": r.table.as_fqn(),
            "eligible": r.eligible,
            "reason_not_eligible": r.reason_not_eligible,
            "retention_days": r.retention_days,
            "time_column": r.time_column,
            "cutoff_utc": r.cutoff.to_rfc3339(),
            "dry_run_rows_older": r.dry_run_rows_older,
            "deleted_rows": r.deleted_rows,
            "batches_executed": r.batches_executed
        }));
    }

    serde_json::json!({
        "event": "runtime_retention_enforcement",
        "run_id": run_id.to_string(),
        "dry_run": dry_run,
        "started_at_utc": started_at.to_rfc3339(),
        "ended_at_utc": ended_at.to_rfc3339(),
        "config": {
            "batch_size": cfg.batch_size,
            "max_batches_per_table": cfg.max_batches_per_table,
            "sleep_ms_between_batches": cfg.sleep_ms_between_batches
        },
        "protected_tables_denylist": DENYLIST_TABLES,
        "append_only_trigger_function": "prevent_update_delete",
        "results": per_table
    })
}

#[cfg(test)]
mod tests {
    use super::QualifiedTable;

    #[test]
    fn parse_qualified_table_accepts_allowed() {
        let qt = QualifiedTable::parse("ransomeye.normalized_events").unwrap();
        assert_eq!(qt.schema, "ransomeye");
        assert_eq!(qt.table, "normalized_events");
    }

    #[test]
    fn parse_qualified_table_rejects_bad_schema() {
        let err = QualifiedTable::parse("pg_catalog.pg_class").unwrap_err();
        assert!(err.contains("Illegal schema"));
    }

    #[test]
    fn quote_ident_rejects_injection() {
        let err = QualifiedTable::quote_ident("x;DROP TABLE y;").unwrap_err();
        assert!(err.contains("illegal identifier"));
    }
}


