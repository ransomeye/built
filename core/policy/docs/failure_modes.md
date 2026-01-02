# Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Failure modes documentation - how the engine handles failures

## Overview

The Policy Engine is **fail-closed**: on any ambiguity or error, it denies rather than proceeding.

## Failure Modes

### 1. Unsigned Policy
**Symptom**: Policy is not signed  
**Action**: ENGINE REFUSES TO START  
**Reason**: Cannot trust unsigned policy  
**Recovery**: Sign policy, restart engine

### 2. Policy Ambiguity
**Symptom**: Multiple policies match with same priority  
**Action**: DENY  
**Reason**: Cannot determine which policy to apply  
**Recovery**: Adjust policy priorities

### 3. Missing Context
**Symptom**: Required context field is missing  
**Action**: DENY  
**Reason**: Cannot evaluate without context  
**Recovery**: Provide complete context

### 4. Evaluation Error
**Symptom**: Error during policy evaluation  
**Action**: DENY  
**Reason**: Cannot determine correct action  
**Recovery**: Fix evaluation error

### 5. Policy Tampered
**Symptom**: Policy hash mismatch  
**Action**: ENGINE REFUSES TO START  
**Reason**: Policy may have been tampered  
**Recovery**: Restore policy, restart engine

### 6. No Matching Policy
**Symptom**: No policy matches the context  
**Action**: DENY (default)  
**Reason**: Better to deny than allow by default  
**Recovery**: Create appropriate policy

## Error Handling

### Fail-Closed Principle
- On any ambiguity → DENY
- On any error → DENY
- On missing context → DENY
- On unsigned policy → REFUSE TO START

### Logging
- All failures are logged
- Failure reasons are recorded
- Recovery actions are documented

### Recovery
- Most failures are recoverable
- Unsigned policy requires signing
- Engine can be restarted after fixes

## Prevention

### Policy Validation
- Policies validated on load
- Signatures verified
- Versions checked

### Context Validation
- Context validated before evaluation
- Missing fields detected
- Completeness ensured

### Ambiguity Detection
- Ambiguities detected during evaluation
- Conflicts resolved by priority
- Default to DENY on ambiguity

