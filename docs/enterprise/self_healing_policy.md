# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/self_healing_policy.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Self-Healing Policy Documentation (PROMPT-59-D)

# Self-Healing Policy (PROMPT-59-D)

## Overview

Controlled self-healing with escalation levels and hard stops for integrity violations.

## Purpose

- **Transient Failures**: Auto-restart for recoverable issues
- **Hard Stops**: No recovery for integrity violations
- **Escalation**: Progressive response to persistent failures
- **Audit Trail**: All escalations logged

## Escalation Levels

### Level 1: Auto-Restart

**Trigger**: Service failure (transient)

**Action**:
- Attempt service restart
- Track restart count in 5-minute window
- Maximum 3 restart attempts

**Limitations**:
- Only for transient failures
- No restart loops
- Escalates if >3 attempts in 5 minutes

**Audit**: Restart attempts logged

### Level 2: Quarantine

**Trigger**: Service fails after Level 1

**Action**:
- Stop service
- Mark as quarantined
- Prevent restart attempts

**Limitations**:
- Service remains stopped
- Manual intervention required
- Escalates if system impact

**Audit**: Quarantine action logged

### Level 3: System Lock

**Trigger**: 
- Integrity violation detected
- Drift detected
- Audit failure detected
- Service fails after quarantine

**Action**:
- Stop all RansomEye services
- Lock system state
- Write audit entry
- Require manual intervention

**Limitations**:
- No automatic recovery
- Manual unlock required
- Full system impact

**Audit**: System lock logged with reason

## Hard Stops

### Integrity Violations

**Detection**: Recent `SYSTEM_INTEGRITY_VIOLATION` audit entries

**Response**: Immediate Level 3 (System Lock)

**Rationale**: Cannot recover from integrity violations automatically

### Drift Detection

**Detection**: Verifier detects unauthorized changes

**Response**: Immediate Level 3 (System Lock)

**Rationale**: Drift indicates potential compromise

### Audit Failures

**Detection**: Broken audit chain (missing chain hashes)

**Response**: Immediate Level 3 (System Lock)

**Rationale**: Audit integrity is non-negotiable

## Self-Healing Engine

### Execution

```bash
python3 /home/ransomeye/rebuild/core/self_heal/self_healing_engine.py <service_name>
```

### State Management

Escalation state stored in:
`/var/lib/ransomeye/self_heal/escalation_state.json`

Contains:
- Service escalation levels
- Restart counts
- System lock status
- Lock timestamp and reason

### Decision Flow

1. Check for hard stops (integrity, drift, audit)
2. If hard stop → Level 3 (System Lock)
3. If service failure → Check escalation level
4. Level 1 → Attempt restart
5. Level 2 → Quarantine
6. Level 3 → System Lock

## Acceptance Criteria

- [x] Auto-restart for transient failures
- [x] Hard stop for integrity violations
- [x] Hard stop for drift
- [x] Hard stop for audit failures
- [x] Escalation levels implemented
- [x] No infinite restart loops
- [x] Explicit audit at every escalation

## Maintenance

### State Review

Review escalation state:

```bash
cat /var/lib/ransomeye/self_heal/escalation_state.json
```

### Manual Unlock

If system is locked, manual intervention required:

1. Investigate lock reason
2. Resolve root cause
3. Clear escalation state
4. Restart services

### Service Recovery

After quarantine or lock:

1. Verify root cause resolved
2. Clear escalation state
3. Restart service manually
4. Monitor for recurrence

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

