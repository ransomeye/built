# RansomEye System Architecture

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 2 architecture documentation - zero-trust, plane-separated system architecture

---

## Overview

Phase 2 defines the **constitutional architecture** of RansomEye. It establishes explicit trust boundaries, plane separation, one-way authority flows, and AI non-authority guarantees.

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

**Documentation:** `planes/data_plane.md`

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

**Documentation:** `planes/control_plane.md`

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

**Documentation:** `planes/intelligence_plane.md`

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

**Documentation:** `planes/management_plane.md`

---

## Trust Boundaries

### Allowed Flows

- Data Plane → Control Plane
- Control Plane → Intelligence Plane (read-only)
- Control Plane → Enforcement Dispatcher
- Intelligence Plane → Human
- Control Plane → Reporting

### Forbidden Flows

- AI → Control Plane
- LLM → Control Plane
- Data Plane → Policy Engine
- Data Plane → Enforcement Dispatcher
- Human → Data Plane
- Human → AI training without approval
- Intelligence → Enforcement

**Documentation:** `trust/forbidden_flows.md`

---

## Identity Model

### Properties

- Unique keypair per component instance
- No shared secrets
- Mutual authentication everywhere
- Identity expiration
- Revocation lists
- Trust-chain validation

**Documentation:** `identity/component_identity.md`

---

## Data Contracts

### Contracts Defined

1. **DPI to Core** - `data_contracts/dpi_to_core.md`
2. **Agent to Core** - `data_contracts/agent_to_core.md`
3. **Core to AI** - `data_contracts/core_to_ai.md`
4. **AI to Human** - `data_contracts/ai_to_human.md`
5. **Enforcement Dispatch** - `data_contracts/enforcement_dispatch.md`

---

## Swap Policy

### Required For

- Core Engine (max(16GB, RAM))
- DPI Probe (max(16GB, RAM))

### Forbidden For

- Linux Agent (NO swap required)
- Windows Agent (NO swap required)

**Documentation:** `swap_policy/swap_requirements.md`, `swap_policy/enforcement_rules.md`

---

## Documentation

- **Architecture Overview** - `docs/architecture_overview.md`
- **Zero-Trust Model** - `docs/zero_trust_model.md`
- **Day-1 Readiness** - `docs/day1_readiness.md`
- **Military Assumptions** - `docs/military_assumptions.md`

---

## Tests

### Rust Tests

- **Forbidden Flow Tests** - `tests/forbidden_flow_tests.rs`
- **Identity Violation Tests** - `tests/identity_violation_tests.rs`
- **AI Authority Violation Tests** - `tests/ai_authority_violation_tests.rs`
- **Plane Isolation Tests** - `tests/plane_isolation_tests.rs`

### Running Tests

```bash
cd /home/ransomeye/rebuild/ransomeye_architecture/tests
cargo test
```

---

## Enforcement

### Fail-Closed Enforcement

All violations result in:
- Process termination
- Component revocation
- Audit log entry
- Human notification

### Enforcement Points

1. **Identity Verification** - All operations verify identity
2. **Signature Validation** - All operations validate signatures
3. **Authorization Checks** - All operations check authorization
4. **Boundary Enforcement** - All boundaries enforced at runtime

---

## Key Principles

### Principle 1: Zero Trust

No component is trusted by default. All trust must be explicitly established and continuously verified.

### Principle 2: Explicit Boundaries

All trust boundaries are explicit and enforced. No implicit trust exists.

### Principle 3: One-Way Flows

Data flows are one-way only. Forbidden flows are terminated at runtime.

### Principle 4: AI Non-Authority

AI/ML/LLM components have zero enforcement authority. They are advisory only.

### Principle 5: Fail-Closed

All violations result in fail-closed behavior. No warnings-only mode.

---

## Last Updated

Phase 2 Implementation - Complete

