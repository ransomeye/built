# RansomEye Architecture Overview

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/docs/architecture_overview.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** High-level architecture overview of RansomEye system

---

## Overview

RansomEye uses a **zero-trust, plane-separated architecture** with explicit trust boundaries, one-way data flows, and fail-closed enforcement.

---

## Architectural Planes

### 1. Data Plane (Untrusted)

**Components:**
- DPI Probe
- Linux Agent
- Windows Agent

**Properties:**
- High volume
- Potentially hostile input
- Never authoritative
- Never long-term stateful
- Never enforces policy
- Never invokes AI

**Swap Policy:** Required for Core/DPI, Forbidden for Agents

---

### 2. Control Plane (Authoritative)

**Components:**
- Core Correlation Engine
- Policy Engine
- Enforcement Dispatcher

**Properties:**
- Deterministic
- Fail-closed
- Signed inputs only
- No AI dependency
- Only plane allowed to authorize enforcement

**Swap Policy:** Required

---

### 3. Intelligence Plane (Advisory Only)

**Components:**
- AI / ML models
- Baseline Intelligence Pack
- SHAP explainability
- Threat Intelligence Fusion
- LLM SOC Copilot

**Properties:**
- Fully trained Day-1
- Read-only inputs
- Advisory outputs only
- Zero enforcement authority
- Must be suppressible without impact

**Swap Policy:** Not required

---

### 4. Management Plane (Human)

**Components:**
- Installer / Upgrader
- Reporting
- Forensics
- Audit & Compliance
- UI

**Properties:**
- Human-initiated
- Authenticated
- Audited
- Two-person integrity for critical ops
- No direct Data Plane access

**Swap Policy:** Not required

---

## Trust Boundaries

### Boundary 1: Data → Control

**Direction:** One-way (Data → Control)

**Enforcement:**
- All data signed
- Identity verified
- Signature validated
- Unsigned data rejected

---

### Boundary 2: Control → Intelligence

**Direction:** One-way (Control → Intelligence)

**Enforcement:**
- Read-only access only
- No write operations
- No state modification
- No enforcement functions

---

### Boundary 3: Intelligence → Human

**Direction:** One-way (Intelligence → Human)

**Enforcement:**
- Advisory outputs only
- No automatic enforcement
- Human review required
- Control Plane validation required

---

### Boundary 4: Control → Enforcement

**Direction:** One-way (Control → Enforcement)

**Enforcement:**
- Only Control Plane can authorize
- All requests signed
- All requests validated
- Unauthorized requests rejected

---

## Forbidden Flows

### Flow 1: AI → Control Plane

**Reason:** AI is non-authoritative

**Enforcement:**
- Process termination
- Component revocation
- Audit log entry

---

### Flow 2: Data Plane → Policy Engine

**Reason:** Data must go through Core Correlation Engine

**Enforcement:**
- Process termination
- Component revocation
- Audit log entry

---

### Flow 3: Intelligence → Enforcement

**Reason:** Intelligence Plane has zero enforcement authority

**Enforcement:**
- Process termination
- Component revocation
- Audit log entry

---

## Identity Model

### Properties

- Unique keypair per component instance
- No shared secrets
- Mutual authentication everywhere
- Identity expiration
- Revocation lists
- Trust-chain validation

### Signing

- All operations signed
- RSA-4096 with PSS padding
- SHA-256 hashing
- Replay protection (timestamp + nonce)

---

## Swap Policy

### Required For

- Core Engine
- DPI Probe

**Requirement:** max(16GB, RAM size)

### Forbidden For

- Linux Agent
- Windows Agent

**Requirement:** NO swap check, NO swap requirement

---

## Fail-Closed Enforcement

All violations result in:
- Process termination
- Component revocation
- Audit log entry
- Human notification

No warnings-only mode.
No bypass mechanisms.
No partial state.

---

## Last Updated

Phase 2 Implementation

