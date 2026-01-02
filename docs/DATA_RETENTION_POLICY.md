# RansomEye Data Retention Policy

**Path and File Name:** `/home/ransomeye/rebuild/docs/DATA_RETENTION_POLICY.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Comprehensive data retention policy documentation and enforcement rules

---

## Overview

RansomEye enforces strict data retention policies to ensure compliance, manage storage, and protect critical AI artifacts. All retention policies are **fail-closed** and **enforced at runtime**.

---

## Configuration

Retention policies are configured in `/home/ransomeye/rebuild/config/retention.txt`:

```
TELEMETRY_RETENTION_MONTHS=6
FORENSIC_RETENTION_DAYS=10
DISK_MAX_USAGE_PERCENT=80
```

---

## Retention Categories

### 1. Telemetry Data

**Default Retention:** 6 months  
**Maximum Retention:** 84 months (7 years)  
**Storage:** PostgreSQL partitions

**Enforcement:**
- Automatic partition drop when data exceeds retention period
- Disk pressure triggers early purge (if usage > 80%)
- Signed purge ledger entry for audit trail

**Purge Process:**
1. Identify partitions older than `TELEMETRY_RETENTION_MONTHS`
2. Drop partitions via `DROP TABLE` (PostgreSQL)
3. Log purge event to signed ledger
4. Generate destruction certificate

**Modules:**
- `ransomeye_retention/telemetry_retention.py`
- `ransomeye_retention/retention_parser.py`
- `ransomeye_retention/disk_monitor.py`

---

### 2. Forensic Evidence

**Default Retention:** 10 days  
**Maximum Retention:** 3650 days (10 years)  
**Storage:** File system (`/home/ransomeye/rebuild/ransomeye_forensic/`)

**Enforcement:**
- Chunk-aware secure deletion (3-pass overwrite)
- Disk pressure triggers early purge (if usage > 80%)
- Signed destruction certificate for each purge

**Purge Process:**
1. Identify files older than `FORENSIC_RETENTION_DAYS`
2. Compute SHA-256 hash before deletion
3. Secure deletion (3-pass overwrite with random data)
4. Generate signed destruction certificate
5. Log purge event to signed ledger

**Modules:**
- `ransomeye_retention/forensic_retention.py`
- `ransomeye_retention/retention_parser.py`
- `ransomeye_retention/disk_monitor.py`

---

### 3. AI Training Artifacts

**Minimum Retention:** 2 years (MANDATORY)  
**Maximum Retention:** Unlimited (operator-controlled)  
**Storage:** File system (`/home/ransomeye/rebuild/ransomeye_ai_core/`)

**Protection Rules:**
- **Cannot be deleted by disk pressure**
- **Cannot be deleted if age < 2 years**
- **Requires explicit operator approval** (signed certificate)
- **Audit trail mandatory** for all deletions

**Protected Artifacts:**
- Model files: `.pkl`, `.h5`, `.pb`, `.onnx`, `.pt`, `.pth`, `.ckpt`, `.gguf`
- Training data: `*_training_data.*`, `*_dataset.*`
- SHAP outputs: `*_shap.*`, `*_explainer.*`
- Metadata: `*_metadata.json`
- Checkpoints: `*_checkpoint.*`, `*_weights.*`

**Deletion Process:**
1. Verify artifact age >= 2 years
2. Require explicit operator approval (signed)
3. Generate deletion certificate
4. Secure deletion (3-pass overwrite)
5. Log to audit trail

**Modules:**
- `ransomeye_retention/ai_retention_guard.py`
- `ransomeye_trust/sign_tool.py` (for approval signing)

---

## Disk Pressure Enforcement

**Threshold:** 80% disk usage (configurable via `DISK_MAX_USAGE_PERCENT`)

**Trigger Behavior:**
1. Disk monitor detects usage >= threshold
2. Triggers retention enforcement callbacks
3. Purges data in order:
   - Telemetry (oldest partitions first)
   - Forensics (oldest files first)
   - **AI artifacts are NEVER purged by disk pressure**

**Target Usage:** After purge, target is 70% usage (configurable)

**Modules:**
- `ransomeye_retention/disk_monitor.py`
- `ransomeye_retention/telemetry_retention.py`
- `ransomeye_retention/forensic_retention.py`

---

## Audit Trail

All retention operations are logged to signed ledger:

**Location:** `/home/ransomeye/rebuild/logs/retention_ledger.jsonl`

**Log Format:**
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "type": "telemetry_purge|forensic_purge",
  "reason": "retention_policy|disk_pressure",
  "partitions": [...],
  "retention_months": 6,
  "cutoff_date": "2023-07-01T00:00:00Z"
}
```

**Signing:**
- Ledger entries are signed with `reports` domain key
- Manifest created for ledger file
- Signature stored in `manifest.sig`

---

## Destruction Certificates

Forensic and AI artifact deletions generate signed destruction certificates:

**Location:** `/home/ransomeye/rebuild/ransomeye_forensic/destruction_certificates/`

**Certificate Format:**
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "type": "forensic_destruction|ai_destruction",
  "files": [
    {
      "path": "/path/to/file",
      "size": 1024,
      "hash": "sha256_hash",
      "mtime": "2023-01-01T00:00:00Z"
    }
  ],
  "retention_days": 10,
  "cutoff_date": "2023-12-22T00:00:00Z",
  "total_files": 1,
  "total_size_bytes": 1024
}
```

**Signing:**
- Certificates are signed with `reports` domain key
- Manifest created for certificate
- Signature stored in `manifest.sig`

---

## Operator Approval Process

For AI artifact deletion:

1. **Request:** Operator requests deletion via CLI or API
2. **Validation:** System checks:
   - Artifact age >= 2 years
   - Artifact is AI-related
3. **Approval:** Operator provides signed approval
4. **Deletion:** System performs secure deletion
5. **Certificate:** Destruction certificate generated
6. **Audit:** Event logged to audit trail

**Approval Format:**
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "artifact_path": "/path/to/artifact",
  "operator": "operator_id",
  "reason": "explicit_request",
  "approved": true,
  "min_retention_years": 2
}
```

**Location:** `/home/ransomeye/rebuild/logs/ai_deletion_approvals/`

---

## Compliance

- **GDPR:** Retention policies support data minimization
- **SOC 2:** Audit trail and signed certificates
- **HIPAA:** Secure deletion with destruction certificates
- **Enterprise:** Configurable retention periods

---

## Runtime Enforcement

Retention enforcement runs:
- **Scheduled:** Daily retention check (via systemd timer)
- **Triggered:** Disk pressure detection (real-time)
- **Manual:** Operator-initiated purge (with approval)

**Systemd Services:**
- `ransomeye-retention-enforcer.service`
- `ransomeye-retention-enforcer.timer`

---

## Validation

Retention configuration is validated:
- **CI/CD:** Retention enforcer validates `config/retention.txt`
- **Runtime:** Parser validates on startup
- **Fail-Closed:** Invalid configuration causes startup failure

**Validation Rules:**
- `TELEMETRY_RETENTION_MONTHS`: 0-84 (0-7 years)
- `FORENSIC_RETENTION_DAYS`: 0-3650 (0-10 years)
- `DISK_MAX_USAGE_PERCENT`: 50-100

---

## Default Values

If `config/retention.txt` is missing or invalid, defaults are used:

```
TELEMETRY_RETENTION_MONTHS=6
FORENSIC_RETENTION_DAYS=10
DISK_MAX_USAGE_PERCENT=80
```

**Note:** Defaults are enforced by `ransomeye_retention/retention_parser.py`.

---

## Last Updated

Phase 0 Implementation  
**Enforcement Level:** Fail-Closed  
**Review Required:** Any changes to retention policies require security architecture review

