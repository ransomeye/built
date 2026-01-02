# Path and File Name : /home/ransomeye/rebuild/docs/PROMPT-25_RUNTIME_RETENTION_ENFORCEMENT.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: PROMPT-25 compliance report for runtime data retention enforcement (purge engine, scheduling, auditability, and fail-closed proofs).

## Objective (PROMPT-25)

Implement **runtime enforcement** of data retention based on install-time configuration in:

- `ransomeye.retention_policies` (authoritative DB config; schema is locked)

Runtime enforcement must:

- Periodically enforce retention policies
- Apply only where `retention_enabled = true`
- Purge (or archive+purge) rows older than `retention_days`
- **Never touch**:
  - `ransomeye.immutable_audit_log`
  - `ransomeye.trust_verification_records`
  - `ransomeye.signature_validation_events`
  - Any table protected by append-only triggers
- Be **fully auditable** (log every run to `immutable_audit_log`)
- Be **fail-closed**:
  - Must refuse to run if `retention_policies` is missing
  - Must refuse to run if an illegal table is targeted

---

## Design Choice (Required)

### Selected: **Option A — Purge-Only (Hard Delete)**

**Justification (schema locked):**

- The authoritative schema (`ransomeye_db_core/schema/schema.sql`) defines **no `{table}_archive` tables** and provides **no archive routing metadata** in `retention_policies`.
- PROMPT-25 forbids schema modifications, so the retention engine **cannot create** archive tables.
- Therefore, the only implementable runtime design is **bounded, audited hard deletes** on **explicitly enabled** tables, while fail-closing on immutable/append-only protected targets.

---

## Implementation (Runtime Retention Engine)

### Core module

- `core/engine/orchestrator/src/retention_enforcer.rs`
  - Reads `retention_policies` rows where `retention_enabled = TRUE`
  - **Denylist-protects** immutable tables (never touched)
  - **Detects append-only protected tables** by finding triggers that use `prevent_update_delete()`
  - Determines an eligible time column by probing `information_schema.columns` and selecting from:
    - `created_at`, `observed_at`, `event_time`, `received_at`, `last_seen_at`, `first_seen_at`, `"timestamp"`
  - Computes cutoff as `NOW() - retention_days * interval '1 day'`
  - Executes purge in **bounded batches** using `ctid` deletion windows
  - Emits immutable audit entries with per-table counts

### Standalone service binary

- `core/engine/orchestrator/src/retention_main.rs`
  - `--dry-run` mode: counts only, no deletes
  - `--live` mode: bounded batched deletes
  - On any enforcement failure: exits non-zero and writes `runtime_retention_failed` into `immutable_audit_log` (best-effort)

### Orchestrator startup: dry-run validation (fail-closed)

- `core/engine/orchestrator/src/lib.rs`
  - On orchestrator startup, performs a **dry-run** enforcement validation.
  - If `retention_policies` is missing/empty or targets illegal tables, orchestrator startup **aborts** (fail-closed).

### Periodic schedule: systemd timer/service

- `systemd/ransomeye-retention-enforcer.service`
- `systemd/ransomeye-retention-enforcer.timer`

The timer invokes the oneshot service periodically; the service runs the retention enforcer in `--live` mode and logs to journald.

---

## Audit Logging Contract

Every enforcement run writes one row to:

- `ransomeye.immutable_audit_log`

Audit actions:

- `runtime_retention_dry_run`
- `runtime_retention_enforcement`
- `runtime_retention_failed` (fail-closed proof; includes error string)

The audit payload includes:

- Run id
- Start/end timestamps
- Config (batch sizing)
- Protected table denylist
- Append-only trigger function name (`prevent_update_delete`)
- Per-table results including:
  - `table`
  - `retention_days`
  - chosen `time_column`
  - `dry_run_rows_older`
  - `deleted_rows`
  - `batches_executed`

---

## Verification Proofs (Executed Locally)

The following commands were executed against a disposable local PostgreSQL instance to produce the required proofs.

### 1) Dry-run output (counts only; no deletes)

Command:

```bash
DB_HOST=localhost DB_PORT=5432 DB_NAME=ransomeye_p25 DB_USER=gagan DB_PASS=gagan \
  /home/ransomeye/rebuild/target/debug/ransomeye_retention_enforcer --dry-run
```

Observed output:

```text
Retention enforcer starting (mode=DRY-RUN, batch_size=1000, max_batches_per_table=200)
[RETENTION][DRY-RUN] 1 rows eligible for purge in ransomeye.error_events (retention_days=1, col=created_at)
Retention run complete: audit_id=3dd5f88e-1237-4c56-9940-46974bb7eefc
Totals: would_purge_rows=1 deleted_rows=0 tables=1
```

### 2) Live run output (rows purged)

Command:

```bash
DB_HOST=localhost DB_PORT=5432 DB_NAME=ransomeye_p25 DB_USER=gagan DB_PASS=gagan \
  /home/ransomeye/rebuild/target/debug/ransomeye_retention_enforcer --live
```

Observed output:

```text
Retention enforcer starting (mode=LIVE, batch_size=1000, max_batches_per_table=200)
[RETENTION] Purged 1 row(s) from ransomeye.error_events in 2 batch(es) (58 ms)
Retention run complete: audit_id=f510566d-30f0-43ed-81ae-6d092e361fce
Totals: would_purge_rows=1 deleted_rows=1 tables=1
```

### 3) Audit log entries proving enforcement

Command:

```bash
PGPASSWORD=gagan psql -h localhost -p 5432 -U gagan -d ransomeye_p25 -v ON_ERROR_STOP=1 <<'SQL'
SELECT audit_id, created_at, action,
       (payload_json->>'dry_run') AS dry_run,
       (payload_json->'results') AS results
FROM ransomeye.immutable_audit_log
WHERE action IN ('runtime_retention_dry_run','runtime_retention_enforcement')
ORDER BY created_at DESC
LIMIT 5;
SQL
```

Observed output (trimmed):

```text
... runtime_retention_enforcement ... deleted_rows: 1 ...
... runtime_retention_dry_run     ... dry_run_rows_older: 1 ...
```

### 4) Proof: fail-closed when `retention_policies` is missing

Command:

```bash
DB_HOST=localhost DB_PORT=5432 DB_NAME=ransomeye_p25_missing_policies DB_USER=gagan DB_PASS=gagan \
  /home/ransomeye/rebuild/target/debug/ransomeye_retention_enforcer --dry-run
```

Observed output:

```text
FAIL-CLOSED: Cannot read ransomeye.retention_policies: db error
```

Audit proof:

```bash
PGPASSWORD=gagan psql -h localhost -p 5432 -U gagan -d ransomeye_p25_missing_policies -v ON_ERROR_STOP=1 <<'SQL'
SELECT audit_id, created_at, action, payload_json
FROM ransomeye.immutable_audit_log
WHERE action='runtime_retention_failed'
ORDER BY created_at DESC
LIMIT 3;
SQL
```

Observed output (trimmed):

```text
... action=runtime_retention_failed ... payload_json.error="FAIL-CLOSED: Cannot read ransomeye.retention_policies: ..."
```

### 5) Proof: fail-closed when an illegal immutable table is targeted

Command (create illegal policy row, then run):

```bash
PGPASSWORD=gagan psql -h localhost -p 5432 -U gagan -d ransomeye_p25 -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO ransomeye.retention_policies (table_name, retention_days, retention_enabled)
VALUES ('ransomeye.immutable_audit_log', 1, TRUE)
ON CONFLICT (table_name) DO UPDATE SET retention_days = EXCLUDED.retention_days, retention_enabled = EXCLUDED.retention_enabled;
SQL

DB_HOST=localhost DB_PORT=5432 DB_NAME=ransomeye_p25 DB_USER=gagan DB_PASS=gagan \
  /home/ransomeye/rebuild/target/debug/ransomeye_retention_enforcer --dry-run
```

Observed output:

```text
FAIL-CLOSED: Illegal retention target 'ransomeye.immutable_audit_log' (immutable/protected table)
```

Audit proof:

```bash
PGPASSWORD=gagan psql -h localhost -p 5432 -U gagan -d ransomeye_p25 -v ON_ERROR_STOP=1 <<'SQL'
SELECT audit_id, created_at, action, payload_json
FROM ransomeye.immutable_audit_log
WHERE action='runtime_retention_failed'
ORDER BY created_at DESC
LIMIT 3;
SQL
```

---

## Notes (Strict Constraints)

- **No schema modifications were committed** (PROMPT-25: schema locked).
- Runtime enforcement **never** deletes from denylisted immutable tables.
- Runtime enforcement **refuses** to operate on append-only protected tables (trigger function `prevent_update_delete()`).

---

**© RansomEye.Tech | Support: Gagan@RansomEye.Tech**


