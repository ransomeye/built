# PHASE C — DATABASE ENABLEMENT (DEFERRED, FAIL-CLOSED)

**Status:** ✅ COMPLETE  
**Last Updated:** 2025-12-29  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU

---

## Overview

Phase C implements **OPTIONAL PostgreSQL database support** for RansomEye with strict fail-closed guarantees.

**Critical Principles:**
- Database is **DISABLED by default**
- Explicitly enabled ONLY via environment flag
- Schema is cryptographically signed and hash-verified
- Core installation succeeds WITHOUT database
- NO fallback logic, NO auto-repair, NO silent recovery

---

## 1. DATABASE ENABLEMENT GATE

### 1.1 Hard Gate Rule

Database features MUST be disabled unless:
```bash
RANSOMEYE_ENABLE_DB=1
```

**Valid Enable Values:**
- `1`, `true`, `TRUE`, `True`, `yes`, `YES`, `Yes`

**Valid Disable Values:**
- `0`, `false`, `FALSE`, `False`, `no`, `NO`, `No`, `` (empty)

**Malformed Values:**
- Cause **FAIL-CLOSED abort** if `fail_closed=True`
- Default to DISABLED if `fail_closed=False`

### 1.2 Implementation

**File:** `ransomeye_db_core/db_gate.py`

```python
from ransomeye_db_core.db_gate import DatabaseGate

# Check if enabled
if DatabaseGate.is_enabled():
    # Database features available
    pass

# Require database (abort if not enabled)
DatabaseGate.require_enabled(context="operation name")

# Get DB configuration
db_config = DatabaseGate.get_db_config()
```

**Single Source of Truth:** All DB feature checks MUST use `DatabaseGate.is_enabled()`

---

## 2. DATABASE TYPE (FIXED)

**Supported Database:**
- PostgreSQL ONLY

**Forbidden Databases:**
- ❌ SQLite
- ❌ MySQL / MariaDB
- ❌ Embedded / file-based databases

---

## 3. SIGNED DATABASE SCHEMA

### 3.1 Schema Artifacts

**Location:** `/opt/ransomeye/db/` (runtime), `ransomeye_db_core/schema/` (build)

**Files:**
- `schema.sql` — Authoritative schema (immutable)
- `schema.hash` — SHA256 hash of schema.sql
- `schema.sig` — Ed25519 signature of schema.hash

### 3.2 Schema Signing

**Signer:** `ransomeye_db_core/schema_signer.py`

**Key Reuse:** Uses existing manifest signing keys (NO new keypair)

**Sign Schema:**
```bash
cd /home/ransomeye/rebuild
sudo python3 ransomeye_db_core/schema_signer.py --sign
```

**Verify Schema:**
```bash
sudo python3 ransomeye_db_core/schema_signer.py --verify
```

### 3.3 Schema Structure

**Tables:**
- `ransomeye.alerts` — Security alerts and detections
- `ransomeye.telemetry` — Endpoint telemetry
- `ransomeye.forensics` — Forensic evidence and artifacts
- `ransomeye.threat_intel` — Threat intelligence indicators
- `ransomeye.audit_log` — System audit trail (7-year retention)
- `ransomeye.incident_summaries` — LLM-generated summaries with SHAP
- `ransomeye.retention_policies` — Data retention definitions

**Functions:**
- `ransomeye.cleanup_expired_data()` — Automated retention enforcement

**Views:**
- `ransomeye.alert_summary` — Aggregated alert statistics

### 3.4 Installer Behavior

**On Install (if DB enabled):**
1. Verify `schema.hash` matches `schema.sql`
2. Verify `schema.sig` is valid
3. On failure → **ABORT installation**

**On Install (if DB disabled):**
- Skip all DB validation

### 3.5 Validator Behavior

**Global Validator Integration:**
- Recomputes schema hash
- Verifies signature
- **FAIL-CLOSED on mismatch**

---

## 4. DATABASE CREDENTIAL HANDLING

### 4.1 Required Environment Variables

**ALL-OR-NOTHING:** All variables MUST be present, or DB is disabled.

```bash
export RANSOMEYE_DB_HOST="localhost"
export RANSOMEYE_DB_PORT="5432"
export RANSOMEYE_DB_NAME="ransomeye"
export RANSOMEYE_DB_USER="ransomeye_app"
export RANSOMEYE_DB_PASSWORD="CHANGE_ME_SECURELY"
```

### 4.2 Security Rules

**NEVER:**
- ❌ Log database credentials
- ❌ Store credentials in code
- ❌ Store credentials in config files
- ❌ Store credentials in manifests
- ❌ Embed credentials in systemd units

**ONLY:**
- ✅ Read from environment variables
- ✅ Pass via secure environment injection

### 4.3 Missing Credentials

**Behavior:**
- Missing ANY variable → Database DISABLED
- Does NOT abort core installation
- Logs warning (no credential details)

---

## 5. DATABASE ROLE & PERMISSIONS

### 5.1 Roles

**Schema Owner:** `ransomeye_admin`
- Creates and owns schema
- Has DDL privileges
- NOT used by application

**Application Role:** `ransomeye_app`
- Used by RansomEye services
- **Least-privilege**

### 5.2 Application Permissions

**Granted:**
- `SELECT` on all tables
- `INSERT` on all tables
- `UPDATE` on all tables
- `USAGE` on sequences
- `EXECUTE` on `cleanup_expired_data()` function

**Forbidden (Revoked):**
- ❌ `DELETE`
- ❌ `DROP`
- ❌ `TRUNCATE`
- ❌ `ALTER SCHEMA`
- ❌ `SUPERUSER`
- ❌ `EXTENSIONS`

### 5.3 Setup Commands

```sql
-- Create roles (as superuser)
CREATE ROLE ransomeye_admin WITH LOGIN PASSWORD 'CHANGE_ME';
CREATE ROLE ransomeye_app WITH LOGIN PASSWORD 'CHANGE_ME';

-- Apply schema (as ransomeye_admin)
\c ransomeye ransomeye_admin
\i schema.sql

-- Permissions are granted in schema.sql
```

---

## 6. RETENTION ENFORCEMENT

### 6.1 Retention Policies

**Table:** `ransomeye.retention_policies`

**Default Policies:**
| Table | Retention | Purpose |
|-------|-----------|---------|
| `alerts` | 90 days | Security alerts |
| `telemetry` | 30 days | Endpoint telemetry |
| `forensics` | 180 days | Forensic evidence |
| `threat_intel` | 365 days | Threat intelligence |
| `audit_log` | 2555 days | Compliance (7 years) |
| `incident_summaries` | 365 days | Incident reports |

### 6.2 Hard Ceilings

**Maximum Retention:** 2555 days (7 years)

**Enforcement:**
- Validator checks retention does NOT exceed maximum
- Violation → **FAIL-CLOSED**

### 6.3 Automated Cleanup

**Function:** `ransomeye.cleanup_expired_data()`

**Execution:**
```sql
SELECT * FROM ransomeye.cleanup_expired_data();
```

**Returns:** Table of (table_name, rows_deleted)

**Scheduling:** Configure via cron or systemd timer (outside schema)

### 6.4 Validator Checks

- Retention policies exist
- Limits do NOT exceed maximums
- Cleanup function exists

---

## 7. GLOBAL VALIDATOR EXTENSION

### 7.1 DB Validation Module

**File:** `ransomeye_db_core/db_validator.py`

**Checks (when DB enabled):**
1. Database reachable
2. Schema hash matches signed version
3. Signature valid
4. `ransomeye` schema exists
5. Retention policies exist and valid
6. Role permissions are least-privileged

### 7.2 Integration

**Global Validator:** `core/global_validator/validate.py`

```python
from ransomeye_db_core.db_validator import DatabaseValidator

validator = DatabaseValidator()
passed, violations = validator.validate_all()

if not passed:
    # CRITICAL violation
    sys.exit(1)
```

### 7.3 Behavior

**If DB Disabled:**
- Skips all DB checks
- Installation proceeds

**If DB Enabled:**
- Runs all DB checks
- Any failure → **CRITICAL violation**
- Installation ABORTED

---

## 8. FAILURE MODES (EXPLICIT)

### 8.1 Database Down

**Scenario:** PostgreSQL service not running

**Behavior:**
- Core services run WITHOUT database
- DB operations fail gracefully
- Logs connection errors (no credentials)

**NO auto-repair, NO fallback**

### 8.2 Schema Mismatch

**Scenario:** schema.sql modified but not re-signed

**Behavior:**
- Hash verification FAILS
- Database DISABLED
- Validation reports mismatch

**NO auto-repair, NO silent recovery**

### 8.3 Signature Invalid

**Scenario:** schema.sig corrupted or tampered

**Behavior:**
- Signature verification FAILS
- **INSTALL FAIL** (if DB enabled)
- Installation ABORTED

**NO auto-repair, NO fallback**

### 8.4 Disk Full

**Scenario:** Database disk full

**Behavior:**
- DB writes BLOCKED
- Services continue without DB
- Logs disk full errors

**NO auto-repair, NO silent recovery**

### 8.5 DB Tampering

**Scenario:** Schema modified outside installer

**Behavior:**
- Validator detects tampering
- **Validator FAIL**
- Manual remediation required

**NO auto-repair, NO silent recovery**

### 8.6 Missing Credentials

**Scenario:** One or more DB env vars missing

**Behavior:**
- Database DISABLED
- Core installation SUCCEEDS
- Logs warning (no credential details)

**Does NOT abort installation**

---

## 9. SECURITY GUARANTEES

### 9.1 Cryptographic Enforcement

✅ Schema signed with Ed25519  
✅ SHA256 hash integrity  
✅ Signature verified on install  
✅ Signature verified by validator  

### 9.2 Least-Privilege

✅ Application role has NO DDL privileges  
✅ Application role has NO DELETE privilege  
✅ Application role has NO SUPERUSER privilege  
✅ Schema owner is NOT application role  

### 9.3 Fail-Closed

✅ Malformed enable flag → ABORT  
✅ Signature invalid → ABORT install  
✅ Schema tampering → Validator FAIL  
✅ Excessive retention → Validator FAIL  

### 9.4 No Secret Leakage

✅ Credentials ONLY from environment  
✅ Credentials NEVER logged  
✅ Credentials NEVER stored  
✅ Credentials NEVER in manifests  

---

## 10. OPERATOR INSTRUCTIONS

### 10.1 Enable Database

```bash
# 1. Set enable flag
export RANSOMEYE_ENABLE_DB=1

# 2. Set credentials
export RANSOMEYE_DB_HOST="localhost"
export RANSOMEYE_DB_PORT="5432"
export RANSOMEYE_DB_NAME="ransomeye"
export RANSOMEYE_DB_USER="ransomeye_app"
export RANSOMEYE_DB_PASSWORD="STRONG_PASSWORD_HERE"

# 3. Run installer
sudo ./install.sh
```

### 10.2 Deploy Schema

```bash
# 1. Create database (as PostgreSQL superuser)
sudo -u postgres createdb ransomeye

# 2. Create roles (as PostgreSQL superuser)
sudo -u postgres psql -d ransomeye << 'EOF'
CREATE ROLE ransomeye_admin WITH LOGIN PASSWORD 'ADMIN_PASSWORD_HERE';
CREATE ROLE ransomeye_app WITH LOGIN PASSWORD 'APP_PASSWORD_HERE';
EOF

# 3. Deploy schema (as ransomeye_admin)
PGPASSWORD='ADMIN_PASSWORD_HERE' psql -U ransomeye_admin -d ransomeye -f \
  /home/ransomeye/rebuild/ransomeye_db_core/schema/schema.sql

# 4. Verify
PGPASSWORD='APP_PASSWORD_HERE' psql -U ransomeye_app -d ransomeye -c \
  "SELECT COUNT(*) FROM ransomeye.retention_policies;"
```

### 10.3 Verify Installation

```bash
# 1. Check schema signature
cd /home/ransomeye/rebuild
sudo python3 ransomeye_db_core/schema_signer.py --verify

# 2. Run DB validator
sudo python3 ransomeye_db_core/db_validator.py

# 3. Run global validator
sudo python3 core/global_validator/validate.py
```

### 10.4 Disable Database

```bash
# 1. Unset or set to 0
export RANSOMEYE_ENABLE_DB=0

# 2. Re-run installer (core features unaffected)
sudo ./install.sh
```

### 10.5 Schedule Retention Cleanup

**Option A: Cron**
```cron
# Run daily at 2 AM
0 2 * * * psql -U ransomeye_app -d ransomeye -c "SELECT * FROM ransomeye.cleanup_expired_data();" >> /var/log/ransomeye/retention_cleanup.log 2>&1
```

**Option B: Systemd Timer** (recommended)
```bash
# Create timer and service in /etc/systemd/system/
```

---

## 11. TROUBLESHOOTING

### 11.1 "Database features disabled" Message

**Cause:** `RANSOMEYE_ENABLE_DB` not set to 1

**Fix:**
```bash
export RANSOMEYE_ENABLE_DB=1
```

### 11.2 "Missing database credential" Error

**Cause:** One or more DB env vars missing

**Fix:** Set ALL required variables:
```bash
export RANSOMEYE_DB_HOST="localhost"
export RANSOMEYE_DB_PORT="5432"
export RANSOMEYE_DB_NAME="ransomeye"
export RANSOMEYE_DB_USER="ransomeye_app"
export RANSOMEYE_DB_PASSWORD="password"
```

### 11.3 "Schema signature verification FAILED"

**Cause:** schema.sql modified without re-signing

**Fix:**
```bash
cd /home/ransomeye/rebuild
sudo python3 ransomeye_db_core/schema_signer.py --sign
```

### 11.4 "Database connection failed"

**Cause:** PostgreSQL not running or credentials incorrect

**Fix:**
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection manually
psql -h localhost -p 5432 -U ransomeye_app -d ransomeye
```

### 11.5 "psycopg2 not installed"

**Cause:** PostgreSQL Python driver missing

**Fix:**
```bash
pip install psycopg2-binary
```

---

## 12. FILE STRUCTURE

```
ransomeye_db_core/
├── db_gate.py              # Central DB enablement gate
├── schema_signer.py        # Schema signing and verification
├── db_validator.py         # DB validation for Global Validator
└── schema/
    ├── schema.sql          # Authoritative PostgreSQL schema
    ├── schema.hash         # SHA256 of schema.sql
    └── schema.sig          # Ed25519 signature

/opt/ransomeye/db/          # Runtime location (deployed by installer)
├── schema.sql
├── schema.hash
└── schema.sig
```

---

## 13. VALIDATION CHECKLIST

### 13.1 Installation (DB Disabled)

- [ ] Core install completes successfully
- [ ] No DB-related errors
- [ ] Services start without DB
- [ ] Validator passes

### 13.2 Installation (DB Enabled)

- [ ] DB enablement flag set
- [ ] All credentials provided
- [ ] Schema signature verified
- [ ] DB connection succeeds
- [ ] Schema deployed
- [ ] Retention policies present
- [ ] Validator passes

### 13.3 Security

- [ ] Schema cryptographically signed
- [ ] Application role has NO DDL
- [ ] Application role has NO DELETE
- [ ] No credentials in logs
- [ ] No credentials in manifests

---

## 14. CONCLUSION

Phase C implements **optional, fail-closed PostgreSQL support** with:

1. **Hard Enablement Gate:** Disabled by default, explicit enable required
2. **Cryptographic Enforcement:** Signed schema, hash verification
3. **Least-Privilege:** Application role has minimal permissions
4. **Fail-Closed Safety:** Any violation aborts installation
5. **No Runtime Dependency:** Core succeeds without DB

**All enforcement mechanisms are mandatory, non-bypassable, and fail-closed.**

---

**Document Status:** ✅ COMPLETE  
**Validation:** All components implemented and tested  
**Next Steps:** Integrate DB validator with Global Validator (if needed)

