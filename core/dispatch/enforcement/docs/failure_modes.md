# Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_enforcement/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Failure mode documentation - fail-closed error handling

## Overview

The Enforcement Dispatcher implements **fail-closed** behavior: any failure results in **NO ACTION**. This document describes all failure modes and their handling.

## Failure Categories

### 1. Decision Validation Failures

#### Unsigned Decision
- **Error:** `EnforcementError::UnsignedDecision`
- **Cause:** Decision missing or empty signature
- **Action:** REJECT execution
- **Recovery:** Decision must be signed before resubmission

#### Invalid Signature
- **Error:** `EnforcementError::InvalidSignature`
- **Cause:** Signature verification failed
- **Action:** REJECT execution
- **Recovery:** Decision must be re-signed with valid key

#### Decision Revoked
- **Error:** `EnforcementError::DecisionRevoked`
- **Cause:** Decision has been revoked
- **Action:** REJECT execution
- **Recovery:** Decision cannot be executed (revoked)

#### Integrity Check Failed
- **Error:** `EnforcementError::IntegrityCheckFailed`
- **Cause:** Decision hash mismatch
- **Action:** REJECT execution
- **Recovery:** Decision must be regenerated

### 2. Approval Failures

#### Missing Approval
- **Error:** `EnforcementError::MissingApproval`
- **Cause:** Required approval not present
- **Action:** HOLD execution
- **Recovery:** Record required approval to proceed

### 3. Guardrail Failures

#### Guardrail Violation
- **Error:** `EnforcementError::GuardrailViolation`
- **Cause:** Safety guardrail violated
- **Action:** REJECT execution
- **Recovery:** Adjust decision parameters or guardrail configuration

#### Blast Radius Exceeded
- **Error:** `EnforcementError::BlastRadiusExceeded`
- **Cause:** Blast radius limit exceeded
- **Action:** REJECT execution
- **Recovery:** Reduce target count or wait for window reset

#### Rate Limit Exceeded
- **Error:** `EnforcementError::RateLimitExceeded`
- **Cause:** Rate limit exceeded
- **Action:** REJECT execution
- **Recovery:** Wait for rate limit window reset

### 4. Adapter Failures

#### Adapter Failure
- **Error:** `EnforcementError::AdapterFailure`
- **Cause:** Platform adapter failed
- **Action:** HALT execution
- **Recovery:** Fix adapter issue or use alternative adapter

### 5. Execution Failures

#### Partial Execution
- **Error:** `EnforcementError::PartialExecution`
- **Cause:** Execution partially completed
- **Action:** ROLLBACK required
- **Recovery:** Execute rollback to restore state

#### Rollback Failed
- **Error:** `EnforcementError::RollbackFailed`
- **Cause:** Rollback execution failed
- **Action:** ESCALATE
- **Recovery:** Manual intervention required

## Fail-Closed Behavior

### Principle
**Any failure → NO ACTION**

### Implementation
1. All failures result in execution rejection
2. No partial execution allowed
3. All failures logged with full context
4. All failures returned in error response

### Exception Handling
- No silent failures
- No partial execution
- No retry without explicit approval
- No bypass of safety checks

## Error Response Structure

### Error Types
```rust
pub enum EnforcementError {
    UnsignedDecision(String),
    InvalidSignature(String),
    DecisionRevoked(String),
    MissingApproval(String),
    GuardrailViolation(String),
    BlastRadiusExceeded(String),
    RateLimitExceeded(String),
    IntegrityCheckFailed(String),
    AdapterFailure(String),
    PartialExecution(String),
    RollbackFailed(String),
    InvalidFormat(String),
    ConfigurationError(String),
    InternalError(String),
}
```

### Error Context
- Error message with full context
- Error type for programmatic handling
- Error source for debugging
- Error timestamp for audit

## Failure Recovery

### Automatic Recovery
- **Rate Limit:** Wait for window reset
- **Blast Radius:** Wait for window reset
- **Adapter Failure:** Try alternative adapter

### Manual Recovery
- **Unsigned Decision:** Re-sign decision
- **Invalid Signature:** Re-sign with valid key
- **Missing Approval:** Record required approval
- **Guardrail Violation:** Adjust parameters
- **Partial Execution:** Execute rollback
- **Rollback Failed:** Manual intervention

## Failure Monitoring

### Metrics
- Failure rate per error type
- Failure rate per decision type
- Failure rate per adapter
- Average recovery time

### Alerts
- High failure rate
- Critical failures (rollback failed)
- Adapter failures
- Approval workflow delays

## Failure Testing

### Test Scenarios
- Unsigned decision rejection
- Invalid signature rejection
- Missing approval hold
- Guardrail violation rejection
- Rate limit exceeded rejection
- Blast radius exceeded rejection
- Adapter failure handling
- Partial execution rollback
- Rollback failure escalation

### Test Coverage
- All error types tested
- All failure modes tested
- All recovery paths tested
- All escalation paths tested

## Failure Documentation

### Error Codes
- Standardized error codes
- Error code documentation
- Error code examples

### Error Messages
- Human-readable error messages
- Machine-readable error codes
- Error context included

### Error Logging
- All errors logged
- Error context preserved
- Error timestamps recorded
- Error source tracked

## Failure Prevention

### Design Principles
- Fail-closed by default
- No silent failures
- No partial execution
- No bypass of safety checks

### Implementation
- Comprehensive validation
- Multiple safety checks
- Approval workflows
- Rate limiting
- Blast radius limiting

### Testing
- Failure mode testing
- Recovery testing
- Escalation testing
- Integration testing

