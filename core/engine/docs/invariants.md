# Invariant Enforcement

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_core/docs/invariants.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Hard invariants enforced by correlation engine

## Overview

The correlation engine enforces **hard invariants** that must never be violated. Violation of any invariant results in **fail-closed behavior** and audit logging.

## Invariants

### 1. No Stage Skip Without Evidence

**Rule:** Cannot transition to a non-sequential stage without sufficient evidence.

**Enforcement:**
- Transition rules validate allowed transitions
- Invariant enforcer checks evidence presence
- Violation → `ABORT_CORRELATION` + audit log

**Example Violation:**
- Current stage: InitialAccess
- Attempted transition: EncryptionExecution
- Evidence: None
- Result: Transition rejected, correlation aborted

### 2. No Confidence Increase Without New Signal

**Rule:** Confidence cannot increase unless a new signal is processed.

**Enforcement:**
- Confidence scorer tracks signal contributions
- Invariant enforcer validates signal presence
- Violation → `REJECT_CONFIDENCE_UPDATE` + audit log

**Example Violation:**
- Old confidence: 0.5
- New confidence: 0.7
- New signals: None
- Result: Confidence update rejected

### 3. No Detection Without Minimum Signal Set

**Rule:** Detection cannot occur without the minimum required signal set.

**Enforcement:**
- Configuration defines minimum signal set
- Invariant enforcer validates signal presence
- Violation → `REJECT_DETECTION` + audit log

**Example Violation:**
- Required signals: {network_connection, process_creation}
- Actual signals: {network_connection}
- Result: Detection rejected

### 4. No State Explosion Without Eviction

**Rule:** Entity state cannot exceed bounds without eviction being triggered.

**Enforcement:**
- Entity state manager tracks state size
- Eviction triggered at capacity
- Violation → `ABORT_CORRELATION` + audit log

**Example Violation:**
- Max state size: 1000 signals
- Current state: 1500 signals
- Eviction: Not triggered
- Result: Correlation aborted

## Fail-Closed Behavior

When an invariant is violated:

1. **Immediate Abort:** Correlation for the entity is aborted
2. **Audit Log:** Violation is logged with full context
3. **No Detection:** No detection result is produced
4. **State Preserved:** Entity state is preserved for investigation

## Audit Logging

All invariant violations are logged with:
- Timestamp
- Violation type
- Entity ID
- Action taken
- Full context

## Testing

Invariant violations are tested in `invariant_violation_tests.rs`:
- Stage skip violations
- Confidence increase violations
- Minimum signal set violations
- State explosion violations

