# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/assurance_mode_lock.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Assurance Mode Lock Documentation (PROMPT-59-A)

# Assurance Mode Lock (PROMPT-59-A)

## Overview

Assurance Mode Lock is the **permanent, irreversible control** that ensures RansomEye operates in **Permanent Assurance Mode** with **zero tolerance for degradation**.

## Purpose

- **Zero Degradation**: System cannot silently degrade or drift
- **Mandatory Checks**: All verifier checks become mandatory (no warnings)
- **Service Protection**: Critical services cannot be disabled
- **Permanent Enforcement**: Lock cannot be removed without integrity violation

## Lock File

### Location

`/etc/ransomeye/ASSURANCE_MODE_LOCK`

### Creation

```bash
python3 /home/ransomeye/rebuild/core/assurance/assurance_lock.py
```

### Contents

JSON file containing:
- Version: `1.0.0-enterprise-ship`
- Created timestamp
- Mode: `PERMANENT_ASSURANCE`
- Irreversible: `true`
- Protected services list

### Protection

- **File Permissions**: Read-only (444)
- **Immutable**: Cannot be modified after creation
- **Removal**: Triggers integrity violation if attempted

## Verifier Enforcement

### Assurance Mode Detection

The verifier checks for assurance lock on every run:

```python
def check_assurance_mode() -> bool:
    assurance_lock_path = Path("/etc/ransomeye/ASSURANCE_MODE_LOCK")
    return assurance_lock_path.exists()
```

### Warning-to-Failure Conversion

When assurance mode is active, **all warnings become failures**:

- Model registry warnings → Failures
- Threat intel warnings → Failures
- DPI protocol warnings → Failures
- Linux agent warnings → Failures
- Artifact hash warnings → Failures

### Enforcement Logic

```python
# In assurance mode, all warnings become failures
if model_error and ("WARNING" not in model_error or assurance_mode):
    results["failures"].append(f"Model registry: {model_error}")
    results["overall_healthy"] = False
```

## Service Protection

### Protected Services

The following services **cannot be stopped or masked**:

- `ransomeye-verifier`
- `ransomeye-compliance-automation`
- `ransomeye-compliance-automation.timer`

### Protection Mechanism

Service protection script (`service_protection.py`) checks protected services:

- Verifies services are active
- Verifies services are enabled
- Writes `SYSTEM_INTEGRITY_VIOLATION` audit entry on violation

### Violation Response

Any attempt to disable protected services triggers:

1. **Audit Entry**: `SYSTEM_INTEGRITY_VIOLATION` written to audit log
2. **Immediate Failure**: System enters failure state
3. **No Recovery**: Cannot proceed until services restored

## Acceptance Criteria

- [x] Lock file created successfully
- [x] Verifier enforces mandatory checks
- [x] All warnings become failures
- [x] Protected services cannot be disabled
- [x] Violations trigger audit entries

## Maintenance

### Lock Status

Check lock status:

```bash
cat /etc/ransomeye/ASSURANCE_MODE_LOCK
```

### Verification

Verify assurance mode is active:

```bash
python3 /home/ransomeye/rebuild/core/assurance/service_protection.py
```

### Removal (Prohibited)

**DO NOT** attempt to remove the lock file. Removal will:
- Trigger integrity violation
- Cause system failure
- Require manual intervention

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

