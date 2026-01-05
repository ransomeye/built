# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/ship_seal_enforcement.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Ship Seal Enforcement Documentation - Immutable ship seal enforcement and runtime binary integrity verification (PROMPT-64-A)

# Ship Seal Enforcement (PROMPT-64-A)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Ship Seal Enforcement provides **irreversible binary integrity protection** for RansomEye v1.0.0-enterprise-ship. Core binaries cannot be replaced silently, and any binary change breaks the verifier, generates SYSTEM_INTEGRITY_VIOLATION, and blocks normal operation.

---

## Architecture

### Components

1. **Ship Seal Enforcer** (`/core/assurance/ship_seal_enforcer.py`)
   - Runtime binary self-hash verification
   - Ship seal hash list embedded (read-only)
   - Immediate fail-closed on mismatch

2. **Verifier Integration** (`/core/verifier/verifier.py`)
   - Ship seal check integrated into continuous verification loop
   - Runs every 5 minutes
   - Fail-closed on violation

3. **Ship Seal Hash List** (`/docs/ARTIFACT_HASHES.txt`)
   - Immutable hash registry for all production binaries
   - SHA-256 checksums for critical artifacts
   - Locked at shipment time

---

## Enforcement Mechanism

### Runtime Binary Self-Hash Check

Every service startup and verifier run:

1. **Load Ship Seal**: Read `ARTIFACT_HASHES.txt` and parse all binary hashes
2. **Verify Critical Binaries**: Compute SHA-256 of all binaries listed in ship seal
3. **Compare Hashes**: Match computed hashes against ship seal hashes
4. **Self-Verification**: Verify enforcer and verifier themselves
5. **Fail-Closed**: On any mismatch:
   - Write `SYSTEM_INTEGRITY_VIOLATION` audit entry
   - Block service startup
   - Exit with non-zero code

### Critical Binaries Protected

- Core binaries (`.so`, `.bin`, executables)
- Model artifacts (`.model`, `.pkl`, `.gguf`)
- Verifier and enforcer scripts
- Systemd service files

---

## Violation Response

### Detection Time

- **Service Startup**: Immediate (before service starts)
- **Continuous Verification**: ≤5 minutes (verifier runs every 5 minutes)

### Violation Actions

1. **Audit Entry**: `SYSTEM_INTEGRITY_VIOLATION` with subtype `SHIP_SEAL_VIOLATION`
2. **Service Block**: Service fails to start or stops immediately
3. **Verifier Failure**: Continuous verifier exits non-zero
4. **Evidence Log**: Violation details logged to audit chain

### Violation Payload

```json
{
  "violation_type": "SYSTEM_INTEGRITY_VIOLATION",
  "violation_subtype": "SHIP_SEAL_VIOLATION",
  "message": "Ship seal violation: N binary hash mismatches",
  "timestamp": "2026-01-28T12:00:00Z",
  "details": {
    "violations": ["binary_path: HASH MISMATCH - expected ..., got ..."],
    "verified_count": 50,
    "total_artifacts": 60
  }
}
```

---

## Ship Seal Format

### ARTIFACT_HASHES.txt Structure

```
# RansomEye Core Production Artifact Hashes
# Generated: 2026-01-04 14:19:36 UTC

/opt/ransomeye/modules/core/ingest/bin/ingest-http
SHA256: 02f32bb01e9df23c78e8a3cdec043974dcc99c4f35fd5b61f2dca087a0dfb0dc

ransomeye_intelligence/baseline_pack/models/anomaly_baseline.model
SHA256: 10566a07cf4c261e0ccd9f952b8d38fa8de4f847be8af49248d43dba8ad48333
```

---

## Integration Points

### Service Startup

Services should call ship seal enforcer before starting:

```python
from core.assurance.ship_seal_enforcer import ShipSealEnforcer

enforcer = ShipSealEnforcer()
if not enforcer.enforce():
    sys.exit(1)  # Fail-closed
```

### Continuous Verifier

Ship seal check integrated into verifier loop:

```python
seal_healthy, seal_error = check_ship_seal()
if not seal_healthy:
    # Write audit and exit
    write_system_integrity_violation_audit(...)
    sys.exit(1)
```

---

## Security Properties

### Immutability

- Ship seal hash list is **read-only** (444 permissions)
- No mechanism to update hashes without detection
- Any hash change triggers violation

### Fail-Closed

- No bypass mechanism
- No warning-only mode
- Violation = immediate service halt

### Non-Repudiation

- All violations logged to immutable audit chain
- Cryptographic evidence of tampering
- Timestamped and chained

---

## Testing

### Manual Verification

```bash
# Run ship seal enforcer directly
python3 /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Expected output:
# ✓ Ship seal verified - all binaries intact
```

### Tamper Simulation

See `/tests/post_ship_tamper_simulation.sh` for safe tamper testing.

---

## Compliance

### Enterprise Requirements

- ✅ Binary integrity protection
- ✅ Runtime verification
- ✅ Fail-closed enforcement
- ✅ Audit trail
- ✅ ≤5 minute detection time

### Regulatory Alignment

- **SOC 2**: Change detection and integrity monitoring
- **ISO 27001**: Asset integrity controls
- **NIST CSF**: PR.DS-6 (Integrity checking)

---

## Limitations

### Known Limitations

1. **File System Attacks**: Cannot protect against kernel-level file system manipulation
2. **Memory Attacks**: Cannot detect in-memory binary modification
3. **Timing Attacks**: Small window between check and execution

### Mitigations

- Continuous verification (5-minute intervals)
- Service startup checks
- Immutable audit logging
- Fail-closed enforcement

---

## Maintenance

### Ship Seal Updates

**WARNING**: Ship seal updates are **irreversible** and require:

1. New shipment version
2. Complete re-verification
3. Customer notification
4. Audit trail update

### Adding New Binaries

1. Compute SHA-256 hash
2. Add to `ARTIFACT_HASHES.txt`
3. Lock file (chmod 444)
4. Re-ship with new version

---

## Conclusion

Ship Seal Enforcement provides **irreversible binary integrity protection** for RansomEye v1.0.0-enterprise-ship. Any binary change is detected within ≤5 minutes and triggers immediate fail-closed response with full audit trail.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

