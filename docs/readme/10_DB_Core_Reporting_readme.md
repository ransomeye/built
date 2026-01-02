# Phase 10: DB Core & Reporting

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/10_DB_Core_Reporting_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 10 - DB Core & Reporting

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/reporting/`
- **Service:** `systemd/ransomeye-reporting.service`
- **Binary:** `ransomeye_reporting` (Rust CLI)

### Core Components

1. **Evidence Collector** (`src/collector.rs`)
   - Gathers evidence from various sources
   - Prepares evidence for preservation
   - Ensures proper attribution and timestamps
   - Computes integrity hashes

2. **Evidence Store** (`src/evidence_store.rs`)
   - Immutable append-only evidence storage
   - Hash chaining
   - Bundle sealing
   - Evidence retrieval

3. **Evidence Hasher** (`src/hasher.rs`)
   - Cryptographic hashing (SHA-256)
   - Hash chain computation
   - Integrity verification

4. **Forensic Timeline** (`src/timeline.rs`)
   - Deterministic chronological event ordering
   - Timeline generation
   - Event correlation
   - Timeline export

5. **Report Builder** (`src/report_builder.rs`)
   - Constructs reproducible reports
   - Multi-format support (PDF, HTML, CSV)
   - Report versioning
   - Report metadata

6. **Report Exporter** (`src/exporter.rs`, `formats/`)
   - PDF export
   - HTML export
   - CSV export
   - JSON export (optional)
   - Markdown export (optional)

7. **Evidence Verifier** (`src/verifier.rs`)
   - Validates evidence integrity
   - Verifies hash chains
   - Detects tampering
   - Corruption detection

8. **Retention Manager** (`src/retention.rs`)
   - Enforces retention policies
   - Secure deletion
   - Retention period tracking
   - Disk cleanup

---

## WHAT DOES NOT EXIST

1. **No separate DB Core service** - Database integration is library-based
2. **No database schema creation in reporting** - Schemas created by other phases
3. **No database initialization** - Database setup handled separately

---

## DATABASE SCHEMAS

**NONE** - Phase 10 does not create database tables.

**Database Usage:**
- Phase 10 reads from database (evidence, events, detections)
- Database schemas created by other phases:
  - Phase 6: Playbook execution tables
  - Phase 9: Network scanner tables
  - Other phases: Their respective tables

**PostgreSQL Integration:**
- Database connection via environment variables
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASS`
- Credentials: `DB_USER=gagan`, `DB_PASS=gagan`

---

## RUNTIME SERVICES

**Service:** `ransomeye-reporting.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-reporting.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-enforcement.service`
- **ExecStart:** `/usr/bin/ransomeye_operations start ransomeye-reporting`

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/reporting, /run/ransomeye/reporting

**CLI Usage:**
- `ransomeye_reporting verify <store_path>` - Verify evidence store
- `ransomeye_reporting export <report_id> <output_dir> <format>` - Export report
- `ransomeye_reporting retention <store_path> [--dry-run]` - Enforce retention

---

## GUARDRAILS ALIGNMENT

Phase 10 enforces guardrails:

1. **Immutable Evidence** - Evidence cannot be modified after sealing
2. **Hash Chaining** - Cryptographic hash chain ensures integrity
3. **Ed25519 Signing** - All evidence signed with Ed25519
4. **Retention Limits** - <= 7 years retention enforced
5. **Fail-Closed** - Invalid evidence → REJECTED

---

## INSTALLER BEHAVIOR

**Installation:**
- Reporting service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Binary built from Rust crate: `core/reporting/`
- Service disabled by default

---

## SYSTEMD INTEGRATION

**Service File:**
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Restart always
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 10 does not use AI/ML models.

**Reporting is deterministic:**
- Evidence collection
- Timeline generation
- Report generation
- No ML inference
- No model loading

---

## COPILOT REALITY

**NONE** - Phase 10 does not provide copilot functionality.

**Reports:**
- Generated from evidence
- Exported in multiple formats
- Used by analysts
- Used by compliance teams

---

## UI REALITY

**NONE** - Phase 10 has no UI.

**Reports:**
- Exported as PDF, HTML, CSV
- Can be viewed in any compatible viewer
- UI integration handled by Phase 11

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Evidence** → REJECTED
2. **Hash Chain Broken** → REJECTED
3. **Invalid Signature** → REJECTED
4. **Retention Violation** → Evidence DELETED
5. **Corruption Detected** → REJECTED

**No Bypass:**
- No `--skip-verification` flag
- No `--skip-signature` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 10 is fully implemented and production-ready:

✅ **Complete Implementation**
- Evidence collection functional
- Evidence store working
- Forensic timeline working
- Report generation working
- Multi-format export working
- Retention management working

✅ **Guardrails Alignment**
- Immutable evidence
- Hash chaining
- Ed25519 signing
- Retention limits
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Evidence rejected on any failure

**Note:** DB Core functionality is library-based, not a separate service. Database schemas created by other phases.

**Recommendation:** Deploy as-is. Phase 10 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
