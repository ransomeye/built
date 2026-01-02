# Swap Policy Enforcement Rules

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/swap_policy/enforcement_rules.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Swap policy enforcement rules - fail-closed on violation

---

## Overview

Swap policy enforcement is **mandatory and fail-closed**. Violations result in startup failure. No exceptions allowed.

---

## Enforcement Rules

### Rule 1: Core Engine Requires Swap

**Component:** `ransomeye_master_core`

**Requirement:**
- Swap >= max(16GB, RAM size)
- Swap must be active
- Swap must be persistent

**Enforcement:**
- Check at startup
- Verify swap exists
- Verify swap active
- Verify swap size

**Violation Response:**
- FAIL STARTUP
- Log error
- Human notification
- No service start

---

### Rule 2: DPI Probe Requires Swap

**Component:** `ransomeye_dpi_probe`

**Requirement:**
- Swap >= max(16GB, RAM size)
- Swap must be active
- Swap must be persistent

**Enforcement:**
- Check at startup
- Verify swap exists
- Verify swap active
- Verify swap size

**Violation Response:**
- FAIL STARTUP
- Log error
- Human notification
- No service start

---

### Rule 3: Linux Agent Forbidden from Swap

**Component:** `ransomeye_linux_agent`

**Requirement:**
- NO swap check
- NO swap requirement
- NO swap enforcement
- Lightweight operation

**Enforcement:**
- No swap check in code
- No swap requirement
- No swap enforcement
- Lightweight operation only

**Violation Response:**
- Code review rejection
- Build failure
- Runtime warning (if swap check found)

---

### Rule 4: Windows Agent Forbidden from Swap

**Component:** `ransomeye_windows_agent`

**Requirement:**
- NO swap check
- NO swap requirement
- NO swap enforcement
- Lightweight operation

**Enforcement:**
- No swap check in code
- No swap requirement
- No swap enforcement
- Lightweight operation only

**Violation Response:**
- Code review rejection
- Build failure
- Runtime warning (if swap check found)

---

## Enforcement Implementation

### Implementation 1: Installer Check

**Location:** `/home/ransomeye/rebuild/ransomeye_installer/system/swap_check.py`

**Function:** `check_swap()`

**Process:**
1. Detect RAM size
2. Calculate required swap
3. Check swap exists
4. Check swap active
5. Check swap size
6. Return result

**Failure:** Fail installation

---

### Implementation 2: Startup Check

**Location:** Core Engine, DPI Probe startup code

**Function:** `verify_swap()`

**Process:**
1. Check swap exists
2. Check swap active
3. Check swap size
4. Verify swap >= required

**Failure:** Fail startup

---

### Implementation 3: Runtime Monitoring

**Location:** Core Engine, DPI Probe runtime code

**Function:** `monitor_swap()`

**Process:**
1. Monitor swap usage
2. Alert if swap low
3. Prevent OOM conditions
4. Log swap events

**Failure:** Alert human, continue operation

---

## Violation Detection

### Detection Method 1: Static Analysis

**Process:**
- Code review
- Static analysis tools
- Pattern matching
- Rule enforcement

**Detection:**
- Swap check in forbidden components
- Missing swap check in required components
- Incorrect swap calculation
- Missing swap enforcement

---

### Detection Method 2: Runtime Check

**Process:**
- Startup verification
- Runtime monitoring
- Swap usage tracking
- OOM prevention

**Detection:**
- Swap missing at startup
- Swap insufficient at startup
- Swap inactive at startup
- Swap exhaustion at runtime

---

## Violation Response

### Response 1: Startup Failure

**Detection:** Swap missing or insufficient at startup

**Response:**
- FAIL STARTUP
- Log error with details
- Human notification
- Service not started

### Response 2: Code Violation

**Detection:** Swap check in forbidden component

**Response:**
- Code review rejection
- Build failure
- Runtime warning
- Fix required

---

## Last Updated

Phase 2 Implementation

