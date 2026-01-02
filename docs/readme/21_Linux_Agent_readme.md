# Phase 21: Linux Agent (Standalone)

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/21_Linux_Agent_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 21 - Linux Agent (Standalone)

---

## STATUS: ❌ NOT_IMPLEMENTED

**Guardrails Status:** NOT_IMPLEMENTED  
**Implementation Location:** `/home/ransomeye/rebuild/ransomeye_linux_agent/` (path exists but not production-ready)  
**Type:** Standalone agent (not installed by main installer)

---

## ⚠️ IMPORTANT DISCLAIMER

**This README documents planned architecture only.**

**No production-ready standalone agent is currently implemented.**

**No installer or lifecycle guarantees apply.**

This component is a standalone agent and is intentionally excluded from the unified installer and systemd model. Any references to implementation, deployment, or enforcement in this document are architectural plans only, not current reality.

---

## PLANNED ARCHITECTURE

### Planned Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_linux_agent/` (or `/home/ransomeye/rebuild/edge/agent/linux/`)
- **Type:** Standalone Rust binary
- **Status:** NOT_IMPLEMENTED (planned only)

### Planned Core Components

1. **Agent Main** (planned)
   - Main entry point
   - Host telemetry collection
   - Event signing
   - Transport to Core

2. **Process Monitoring** (planned)
   - Process telemetry collection
   - Process lifecycle tracking
   - Process activity monitoring

3. **File Activity Monitoring** (planned)
   - File system activity tracking
   - File access monitoring
   - File modification tracking

4. **Auth Activity Monitoring** (planned)
   - Authentication event tracking
   - Login/logout monitoring
   - Privilege escalation tracking

5. **Network Activity Monitoring** (planned)
   - Network connection tracking
   - Network activity monitoring

6. **Event Signing** (planned)
   - RSA-4096-PSS-SHA256 signing
   - Event signature generation
   - Signature verification

7. **Transport Client** (planned)
   - mTLS client for Core communication
   - Event transmission
   - Heartbeat mechanism

---

## WHAT DOES NOT EXIST

1. **No production-ready agent** - Not implemented
2. **No installer** - Not implemented
3. **No systemd service** - Not implemented (standalone agents excluded from unified systemd)
4. **No lifecycle management** - Not implemented
5. **No deployment guarantees** - Not implemented

---

## DATABASE SCHEMAS

**NONE** - Phase 21 (Linux Agent) does not create database tables. Agents send telemetry to Core, which stores data.

---

## RUNTIME SERVICES

**NONE** - Phase 21 has no systemd service in the unified installer model.

**Standalone Agent Status:**
- Standalone agents are intentionally excluded from unified installer
- Standalone agents have independent lifecycle
- Standalone agents are excluded from unified systemd validation

---

## GUARDRAILS ALIGNMENT

Phase 21 (planned) would align with guardrails:

1. **Standalone Agent** - Not installed by main installer
2. **Independent Lifecycle** - Separate installer required
3. **Rootless Operation** - Must be rootless
4. **Fail-Open (Sensor)** - Core unavailability → Buffer to disk

**Current Status:**
- NOT_IMPLEMENTED per guardrails.yaml
- No production-ready implementation
- Architecture documented only

---

## INSTALLER BEHAVIOR

**Installation:**
- **NOT installed by main installer** - Standalone agents excluded
- **Dedicated installer required** - If implemented, would require separate installer
- **No lifecycle guarantees** - Not implemented

**Note:** This component is a standalone agent and is intentionally excluded from the unified installer and systemd model.

---

## SYSTEMD INTEGRATION

**NONE** - Phase 21 has no systemd service in unified installer model.

**Standalone Agent Policy:**
- Standalone agents are excluded from unified systemd directory
- Standalone agents may have their own systemd units (if implemented)
- Standalone agents are not validated by unified systemd rules

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 21 (Linux Agent) does not use AI/ML models. Agents collect telemetry only.

---

## FAIL-CLOSED BEHAVIOR

**FAIL-OPEN (Sensor Behavior):**

1. **Core Unavailable** → Buffer to disk (fail-open)
2. **Identity Failure** → Agent DISABLED (fail-closed)
3. **Signing Failure** → Event REJECTED (fail-closed)
4. **Buffer Full** → Backpressure signal (fail-open)

**Note:** This behavior is planned only. No production-ready agent exists to enforce these rules.

---

## STATUS SUMMARY

❌ **NOT_IMPLEMENTED** - No production-ready standalone Linux agent exists

**Planned Components:**
- ❌ Agent main (not implemented)
- ❌ Process monitoring (not implemented)
- ❌ File activity monitoring (not implemented)
- ❌ Auth activity monitoring (not implemented)
- ❌ Network activity monitoring (not implemented)
- ❌ Event signing (not implemented)
- ❌ Transport client (not implemented)

**Current Reality:**
- No production-ready implementation
- No installer
- No deployment guarantees
- Architecture documented only

**Note:** This component is a standalone agent and is intentionally excluded from the unified installer and systemd model. This README documents planned architecture only. No production-ready standalone agent is currently implemented. No installer or lifecycle guarantees apply.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
