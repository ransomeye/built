# Ambiguity Rules

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/docs/ambiguity_rules.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Ambiguity rules - how the engine handles ambiguous situations

## Overview

The Policy Engine follows a **strict ambiguity policy**: any ambiguity → DENY.

## Ambiguity Cases

### 1. Multiple Policies Match (Same Priority)
**Symptom**: Multiple policies match with the same priority  
**Action**: DENY  
**Reason**: Cannot determine which policy to apply  
**Recovery**: Adjust policy priorities

### 2. Conflicting Decisions
**Symptom**: Policies specify conflicting actions  
**Action**: DENY  
**Reason**: Cannot resolve conflict  
**Recovery**: Resolve policy conflicts

### 3. Missing Context
**Symptom**: Required context field is missing  
**Action**: DENY  
**Reason**: Cannot evaluate without context  
**Recovery**: Provide complete context

### 4. Policy Ambiguity
**Symptom**: Policy conditions are ambiguous  
**Action**: DENY  
**Reason**: Cannot determine policy applicability  
**Recovery**: Clarify policy conditions

## Ambiguity Detection

### Priority-Based Resolution
- Policies evaluated by priority (higher first)
- First matching policy wins
- Same priority → ambiguity → DENY

### Condition Evaluation
- All conditions must match deterministically
- Ambiguous conditions → DENY
- Missing conditions → DENY

## Default Behavior

### No Matching Policy
**Symptom**: No policy matches the context  
**Action**: DENY (default)  
**Reason**: Better to deny than allow by default  
**Recovery**: Create appropriate policy

### Evaluation Error
**Symptom**: Error during policy evaluation  
**Action**: DENY  
**Reason**: Cannot determine correct action  
**Recovery**: Fix evaluation error

## Fail-Closed Principle

- **Ambiguity → DENY**: Better to deny than allow incorrectly
- **Error → DENY**: Better to deny than proceed with error
- **Missing → DENY**: Better to deny than assume

This ensures:
- Security is never compromised
- Decisions are always defensible
- Audits are always clean

