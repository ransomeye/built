# RansomEye Posture Engine

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_posture_engine/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Host & Network Management Posture (HNMP) evaluation engine

## Overview

The RansomEye Posture Engine is a **deterministic, advisory-only** posture evaluation system that:

- Ingests telemetry from Linux Agent, Windows Agent, and DPI Probe
- Normalizes signals into posture facts (host hardening, auth hygiene, network exposure, drift detection)
- Evaluates against CIS Benchmarks, STIG profiles, and custom YAML policies
- Produces host and network posture scores
- Generates compliance reports (PDF, HTML, CSV)
- Detects configuration and posture drift
- Maintains full audit trail
- Signs all outputs cryptographically

## Hard Rules

- **ZERO enforcement authority** - Advisory only
- **Deterministic logic only** - NO ML
- **Fail-closed on ambiguity** - Unknown conditions treated as non-compliant
- **Full audit trail** - All operations logged immutably
- **Ed25519 signing ONLY** - RSA is PROHIBITED (aligned with Phase 10)
- **Database is UNTRUSTED** - Every telemetry record MUST be verified with Ed25519
- **Policy hash pinning MANDATORY** - Every evaluation includes policy SHA-256 hash, version, and source path

## Architecture

```
Telemetry Sources
    ↓
Telemetry Ingester (queries database)
    ↓
Signal Normalizer (extracts posture facts)
    ↓
Evaluators (CIS, STIG, Custom)
    ↓
Scorer (calculates posture scores)
    ↓
Drift Detector (compares against baseline)
    ↓
Report Generator (PDF, HTML, CSV)
    ↓
Output Signer (cryptographic signatures)
    ↓
Audit Trail (immutable logging)
```

## Components

### 1. Telemetry Ingester
- Queries PostgreSQL database for signed, validated telemetry events
- Filters by producer type (linux_agent, windows_agent, dpi_probe)
- Supports time-range queries

### 2. Signal Normalizer
- Normalizes telemetry into standardized posture facts
- Categories:
  - Host Hardening
  - Auth Hygiene
  - Network Exposure
  - Drift Detection

### 3. Evaluators
- **CIS Evaluator**: Evaluates against CIS Benchmark controls
- **STIG Evaluator**: Evaluates against STIG requirements
- **Custom Policy Evaluator**: Evaluates against user-defined YAML policies

### 4. Scorer
- Calculates host posture scores (0.0 to 1.0)
- Calculates network posture scores
- Weighted scoring across frameworks

### 5. Drift Detector
- Compares current posture against historical baseline
- Detects score drift, configuration drift, and fact drift
- Generates drift alerts

### 6. Report Generator
- Generates reports in PDF, HTML, and CSV formats
- All reports include footer: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
- Includes timestamp, build hash, and model version hash

### 7. Output Signer
- Cryptographically signs all outputs using RSA-4096-PSS-SHA256
- Generates signature metadata files

### 8. Audit Trail
- Maintains immutable audit log of all operations
- JSONL format, one file per day
- Queryable by time range, action, and host

## Configuration

All configuration via environment variables:

### Database
- `DB_HOST`: PostgreSQL host (default: localhost)
- `DB_PORT`: PostgreSQL port (default: 5432)
- `DB_NAME`: Database name (default: ransomeye)
- `DB_USER`: Database user (default: gagan)
- `DB_PASS`: Database password (default: gagan)

### Policy Directories
- `CIS_BENCHMARKS_DIR`: CIS benchmarks directory
- `STIG_PROFILES_DIR`: STIG profiles directory
- `CUSTOM_POLICIES_DIR`: Custom policies directory

### Output Directories
- `POSTURE_OUTPUT_DIR`: Output directory for reports
- `POSTURE_AUDIT_LOG_DIR`: Audit log directory

### Evaluation Settings
- `POSTURE_EVAL_INTERVAL_SEC`: Evaluation interval in seconds (default: 3600)
- `POSTURE_DRIFT_WINDOW_HOURS`: Drift detection window in hours (default: 24)

### Signing (Ed25519 ONLY)
- `POSTURE_SIGNING_KEY_PATH`: Path to Ed25519 private key for signing (required)
- `POSTURE_TRUST_STORE_PATH`: Path to trust store with Ed25519 public keys (required for signature verification)

## Installation

The posture engine is installed as part of the main RansomEye installer:

```bash
sudo ./install.sh
```

The systemd service is automatically installed and enabled.

## Usage

### Start Service
```bash
sudo systemctl start ransomeye-posture-engine
```

### Check Status
```bash
sudo systemctl status ransomeye-posture-engine
```

### View Logs
```bash
sudo journalctl -u ransomeye-posture-engine -f
```

### Run Manually
```bash
python3 -m ransomeye_posture_engine
```

## Policy Format

### CIS Benchmark Format
```yaml
controls:
  - id: CIS-1.1.1
    title: Control Title
    description: Control description
    severity: high
    check_type: deterministic
    check_logic:
      type: absence
      fact_types: [privileged_execution]
```

### STIG Profile Format
```yaml
requirements:
  - id: STIG-001
    title: Requirement Title
    description: Requirement description
    severity: CAT II
    vuln_id: V-12345
    check_type: deterministic
    check_logic:
      type: threshold
      threshold: 5
```

### Custom Policy Format
```yaml
policies:
  - policy_id: CUSTOM-001
    name: Policy Name
    description: Policy description
    severity: medium
    enabled: true
    check_logic:
      type: pattern
      pattern:
        risk_level: high
```

## Testing

Run unit tests:
```bash
cd /home/ransomeye/rebuild/ransomeye_posture_engine
python3 -m pytest tests/
```

## Output

Reports are generated in:
- `/home/ransomeye/rebuild/ransomeye_posture_engine/output/`

Audit logs are written to:
- `/home/ransomeye/rebuild/logs/posture_engine/`

## Integration

The posture engine integrates with:
- **Phase 10 (Reporting)**: Reports consumed by reporting module
- **Database**: Queries telemetry from PostgreSQL
- **Agents**: Receives telemetry from Linux/Windows agents and DPI probe

## Compliance

- **Ed25519 signing ONLY** - RSA is PROHIBITED (aligned with Phase 10)
- **Database is UNTRUSTED** - Every telemetry record verified with Ed25519 before processing
- **Policy hash pinning** - Every evaluation includes policy SHA-256 hash, version, and source path
- Full audit trail maintained
- Fail-closed on ambiguity
- Zero enforcement (advisory only)
- Deterministic logic only (no ML)

## Support

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

