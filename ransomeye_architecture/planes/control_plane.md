# Control Plane Architecture

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/planes/control_plane.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Control Plane definition - authoritative, deterministic, fail-closed enforcement

---

## Overview

The Control Plane is the **authoritative decision-making layer** of RansomEye. It is the ONLY plane allowed to authorize enforcement actions.

---

## Components

### 1. Core Correlation Engine

**Location:** `/home/ransomeye/rebuild/ransomeye_threat_correlation/`

**Function:**
- Correlate events from Data Plane
- Build threat graphs
- Identify attack patterns
- Generate alerts

**Properties:**
- Deterministic (same input → same output)
- Fail-closed (errors → safe state)
- Signed inputs only
- No AI dependency
- Authoritative for correlation

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Signed telemetry from Data Plane
- Output: Correlated events → Enforcement Dispatcher

---

### 2. Policy Engine

**Location:** `/home/ransomeye/rebuild/ransomeye_alert_engine/`

**Function:**
- Load and validate policies
- Match events to policies
- Generate policy violations
- Hot-reload policies

**Properties:**
- Deterministic (policy-based decisions)
- Fail-closed (invalid policy → reject)
- Signed policies only
- No AI dependency
- Authoritative for policy decisions

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Correlated events from Core Correlation Engine
- Output: Policy violations → Enforcement Dispatcher

---

### 3. Enforcement Dispatcher

**Location:** `/home/ransomeye/rebuild/ransomeye_response/`

**Function:**
- Receive enforcement requests
- Validate authorization
- Dispatch enforcement actions
- Log all enforcement

**Properties:**
- Deterministic (rule-based dispatch)
- Fail-closed (invalid request → reject)
- Signed requests only
- No AI dependency
- ONLY plane allowed to authorize enforcement

**Identity:** Unique per-instance keypair

**Data Flow:**
- Input: Policy violations from Policy Engine
- Output: Enforcement actions → Target systems

---

## Trust Properties

### Authoritative

Control Plane is **authoritative** for:
- Policy decisions
- Enforcement authorization
- State management
- Configuration

### Deterministic

All Control Plane operations are **deterministic**:
- Same input → same output
- No randomness in decisions
- Reproducible behavior
- Testable logic

### Fail-Closed

All Control Plane operations are **fail-closed**:
- Errors → safe state
- Invalid input → reject
- Missing data → reject
- No partial state

### Signed Inputs Only

Control Plane accepts **only signed inputs**:
- All telemetry must be signed
- All policies must be signed
- All requests must be signed
- Unsigned input → reject

### No AI Dependency

Control Plane has **no AI dependency**:
- Cannot call ML models
- Cannot use LLM
- Cannot request AI analysis
- Pure rule-based logic

---

## Allowed Operations

1. **Correlation**
   - Correlate events from Data Plane
   - Build threat graphs
   - Identify patterns

2. **Policy Evaluation**
   - Evaluate policies against events
   - Generate policy violations
   - Hot-reload policies

3. **Enforcement Authorization**
   - Authorize enforcement actions
   - Validate enforcement requests
   - Dispatch enforcement

4. **State Management**
   - Manage system state
   - Store configuration
   - Maintain audit logs

---

## Forbidden Operations

1. **AI Invocation**
   - Cannot call ML models
   - Cannot use LLM
   - Cannot request AI analysis

2. **Direct Data Plane Access**
   - Cannot access Data Plane directly
   - Must receive signed telemetry only
   - Cannot modify Data Plane components

3. **Intelligence Plane Authority**
   - Cannot accept AI recommendations as authoritative
   - Cannot delegate enforcement to AI
   - AI is advisory only

---

## Enforcement Authority

Control Plane is the **ONLY plane** allowed to:
- Authorize enforcement actions
- Make policy decisions
- Modify system state
- Access long-term storage

All other planes are **non-authoritative**.

---

## Failure Modes

### Invalid Input

If Control Plane receives invalid input:
- Must reject input
- Must log rejection
- Must maintain safe state
- Must not proceed

### Policy Violation

If Control Plane detects policy violation:
- Must generate alert
- Must dispatch enforcement
- Must log action
- Must maintain audit trail

### Component Failure

If Control Plane component fails:
- Must fail-closed
- Must maintain safe state
- Must log failure
- Must not corrupt state

---

## Enforcement Points

All Control Plane operations are enforced at:
1. **Input Validation** - All inputs must be signed and valid
2. **Policy Validation** - All policies must be signed and valid
3. **Authorization Check** - Only Control Plane can authorize enforcement
4. **State Validation** - All state changes must be validated

---

## Last Updated

Phase 2 Implementation

