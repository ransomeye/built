# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/change_control_policy.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Change Control Policy Documentation (PROMPT-58-C)

# Change Control Policy (PROMPT-58-C)

## Overview

Pre-update gate enforcement ensures all changes are validated, staged, and verified before production deployment.

## Purpose

- **Prevent Unauthorized Changes**: Block changes that don't meet requirements
- **Ensure Stability**: Require staging execution and verifier green status
- **Maintain Audit Trail**: Log all change attempts (approved and blocked)
- **Enforce Version Control**: Mandatory version bump for all changes

## Change Control Guard

### Execution

```bash
python3 /home/ransomeye/rebuild/core/change_control/change_guard.py <change_type> <version> [change_details_json]
```

**Parameters**:
- `change_type`: `binary`, `schema`, `model`, or `config`
- `version`: New version string (must be incremented)
- `change_details_json`: Optional JSON with change details

### Example

```bash
python3 /home/ransomeye/rebuild/core/change_control/change_guard.py binary 1.0.1 '{"description": "Security patch"}'
```

## Requirements

### 1. Version Bump

- **Current Version**: `1.0.0-enterprise-ship`
- **Requirement**: New version must be incremented
- **Format**: `MAJOR.MINOR.PATCH[-LABEL]`
- **Validation**: Patch version must be greater than current

### 2. Staging Execution

- **Requirement**: Changes must be executed in staging
- **Marker**: `/var/lib/ransomeye/staging/staging_execution.json`
- **Duration**: Minimum 24 hours in staging
- **Validation**: Execution timestamp checked

### 3. Verifier Green Status

- **Requirement**: Verifier must be green for ≥24 hours
- **Check**: `/var/log/ransomeye/verifier_results.json`
- **Duration**: Minimum 24 hours continuous green
- **Validation**: Timestamp and health status checked

### 4. No Hot Changes

- **Prohibition**: No in-place modifications to running system
- **Requirement**: All changes via approved upgrade procedure
- **Enforcement**: Change guard blocks hot changes

## Change Types

### Binary Changes

- New service binaries
- Updated libraries
- Updated models (.pkl, .gguf)

**Requirements**: All above requirements apply

### Schema Changes

- Database table modifications
- Index changes
- Constraint modifications

**Additional Requirements**:
- Migration script required
- Rollback script required
- Data integrity validation

### Model Changes

- New model versions
- Model retraining
- SHAP explainability updates

**Additional Requirements**:
- Model metadata file
- SHAP explainability file
- Model version hash

### Config Changes

- Environment variable changes
- Policy file updates
- Systemd unit modifications

**Additional Requirements**:
- Configuration backup
- Rollback procedure

## Enforcement

### Approval Flow

1. Change guard validates all requirements
2. If valid: Change approved, audit entry created
3. If invalid: Change blocked, violation audit entry created

### Violation Handling

- **Blocked**: Change rejected, exit code 1
- **Audit Entry**: `CHANGE_CONTROL_VIOLATION` written to audit log
- **Log Entry**: Change control log entry created
- **Alert**: Violation details logged

### Audit Trail

All change attempts (approved and blocked) are logged:

- **Location**: `/var/log/ransomeye/change_control.log`
- **Database**: `ransomeye.immutable_audit_log` table
- **Format**: JSON with timestamp, event type, message, details

## Acceptance Criteria

- [x] Unauthorized changes blocked
- [x] Audit entry on violation
- [x] Version bump enforced
- [x] Staging execution required
- [x] Verifier green ≥24h required
- [x] No hot changes allowed

## Integration

### Pre-Upgrade Hook

Change guard should be called before any upgrade:

```bash
# Validate change
python3 /home/ransomeye/rebuild/core/change_control/change_guard.py binary 1.0.1

# If approved, proceed with upgrade
# If blocked, abort upgrade
```

### CI/CD Integration

- Run change guard in CI pipeline
- Block merges if change guard fails
- Require staging execution before production

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

