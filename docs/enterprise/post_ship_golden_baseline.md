# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/post_ship_golden_baseline.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Post-Ship Golden Baseline Documentation (PROMPT-58-A)

# Post-Ship Golden Baseline (PROMPT-58-A)

## Overview

The Golden Baseline is an immutable snapshot of the RansomEye system state at `v1.0.0-enterprise-ship`. It serves as the reference point for all future drift detection and compliance verification.

## Purpose

- **Drift Detection**: Compare current state against golden baseline
- **Compliance Verification**: Prove system integrity matches ship state
- **Forensic Analysis**: Reference point for incident investigation
- **Change Validation**: Verify authorized changes only

## Capture Process

### Execution

```bash
python3 /home/ransomeye/rebuild/core/baseline/golden_baseline_capture.py
```

### Captured Components

1. **OS Information**
   - OS name, release, version
   - Kernel version
   - Distribution information
   - Installed packages (count and hash)

2. **Systemd Unit Hashes**
   - All `ransomeye*.service` files
   - All `ransomeye*.timer` files
   - SHA256 hashes and modification times

3. **Database Schema Checksum**
   - All tables in `ransomeye` schema
   - Column definitions
   - Schema checksum (SHA256)

4. **Artifact Hashes**
   - Re-verification against `ARTIFACT_HASHES.txt`
   - Model files (.pkl, .gguf)
   - Binary files
   - Hash mismatches flagged

5. **Runtime Layout**
   - Directory structure at `/opt/ransomeye`
   - File inventory (first 1000 files)
   - File hashes (for files <100MB)

6. **Service Status**
   - Current systemd service states
   - Load, active, and sub states

## Storage

### Location

`/var/lib/ransomeye/baselines/golden_baseline.json`

### Protection

- File permissions: `444` (read-only for all)
- Immutable after creation
- Never modified, only replaced

### Format

JSON structure with:
- `version`: System version
- `capture_timestamp`: ISO 8601 timestamp
- `baseline_checksum`: SHA256 of entire baseline
- Component-specific data

## Verification

### Against Ship Seal

If `/home/ransomeye/rebuild/docs/enterprise/SHIP_SEAL.txt` exists, compare critical hashes manually.

### Drift Detection

The verifier (`/home/ransomeye/rebuild/core/verifier/verifier.py`) uses the golden baseline for drift detection:

- New files detected
- Modified binaries detected
- Changed systemd units detected
- Changed DB schema detected

## Acceptance Criteria

- [x] Snapshot created successfully
- [x] All hashes match ship seal (if available)
- [x] No drift detected at capture time
- [x] File set to read-only (444)
- [x] Baseline checksum computed

## Maintenance

### Updates

The golden baseline should **never** be updated. If system state changes, create a new baseline with incremented version.

### Retention

- Golden baseline retained indefinitely
- Used for all future compliance checks
- Reference for forensic analysis

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

