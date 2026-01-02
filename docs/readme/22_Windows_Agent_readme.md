# Phase 22: Windows Agent (Standalone)

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/22_Windows_Agent_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 22 - Windows Agent (Standalone)

---

## STATUS: ❌ NOT_IMPLEMENTED

**Guardrails Status:** NOT_IMPLEMENTED  
**Implementation Location:** `/home/ransomeye/rebuild/ransomeye_windows_agent/` (path exists but not production-ready)  
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
- **Directory:** `/home/ransomeye/rebuild/ransomeye_windows_agent/` (or `/home/ransomeye/rebuild/edge/agent/windows/`)
- **Type:** Standalone Windows binary
- **Status:** NOT_IMPLEMENTED (planned only)

### Planned Core Components

1. **Agent Main** (planned)
   - Main entry point
   - Endpoint telemetry collection
   - ETW (Event Tracing for Windows) integration
   - Event signing
   - Transport to Core

2. **Process Activity Monitoring** (planned)
   - Process telemetry collection via ETW
   - Process lifecycle tracking
   - Process activity monitoring

3. **File Activity Monitoring** (planned)
   - File system activity tracking via ETW
   - File access monitoring
   - File modification tracking

4. **Registry Activity Monitoring** (planned)
   - Registry activity tracking via ETW
   - Registry key monitoring
   - Registry value monitoring

5. **Auth Activity Monitoring** (planned)
   - Authentication event tracking via ETW
   - Login/logout monitoring
   - Privilege escalation tracking

6. **Network Activity Monitoring** (planned)
   - Network connection tracking via ETW
   - Network activity monitoring

7. **ETW Integration** (planned)
   - ETW provider abstraction
   - High-performance kernel-level events
   - WMI fallback when ETW unavailable

8. **Event Signing** (planned)
   - Ed25519 signing
   - Event signature generation
   - Signature verification

9. **Transport Client** (planned)
   - mTLS client for Core communication
   - Event transmission
   - Backpressure handling
   - Reconnection logic

---

## WHAT DOES NOT EXIST

1. **No production-ready agent** - Not implemented
2. **No MSI installer** - Not implemented
3. **No Windows service** - Not implemented
4. **No lifecycle management** - Not implemented
5. **No deployment guarantees** - Not implemented

---

## DATABASE SCHEMAS

**NONE** - Phase 22 (Windows Agent) does not create database tables. Agents send telemetry to Core, which stores data.

---

## RUNTIME SERVICES

**NONE** - Phase 22 has no Windows service in the unified installer model.

**Standalone Agent Status:**
- Standalone agents are intentionally excluded from unified installer
- Standalone agents have independent lifecycle
- Standalone agents are excluded from unified systemd validation

---

## GUARDRAILS ALIGNMENT

Phase 22 (planned) would align with guardrails:

1. **Standalone Agent** - Not installed by main installer
2. **Independent Lifecycle** - Separate MSI installer required
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
- **Dedicated MSI installer required** - If implemented, would require separate MSI installer
- **No lifecycle guarantees** - Not implemented

**Note:** This component is a standalone agent and is intentionally excluded from the unified installer and systemd model.

---

## SYSTEMD INTEGRATION

**N/A** - Phase 22 is Windows-only (Windows Service, not systemd).

**Standalone Agent Policy:**
- Standalone agents are excluded from unified installer
- Standalone agents may have their own Windows service (if implemented)
- Standalone agents are not validated by unified systemd rules

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 22 (Windows Agent) does not use AI/ML models. Agents collect telemetry only.

---

## FAIL-CLOSED BEHAVIOR

**FAIL-OPEN (Sensor Behavior):**

1. **Core Unavailable** → Buffer to disk (fail-open)
2. **Identity Failure** → Agent DISABLED (fail-closed)
3. **Signing Failure** → Event REJECTED (fail-closed)
4. **Buffer Full** → Backpressure signal (fail-open)
5. **ETW Unavailable** → WMI fallback (fail-open)

**Note:** This behavior is planned only. No production-ready agent exists to enforce these rules.

---

## STATUS SUMMARY

❌ **NOT_IMPLEMENTED** - No production-ready standalone Windows agent exists

**Planned Components:**
- ❌ Agent main (not implemented)
- ❌ ETW integration (not implemented)
- ❌ Process activity monitoring (not implemented)
- ❌ File activity monitoring (not implemented)
- ❌ Registry activity monitoring (not implemented)
- ❌ Auth activity monitoring (not implemented)
- ❌ Network activity monitoring (not implemented)
- ❌ Event signing (not implemented)
- ❌ Transport client (not implemented)

**Current Reality:**
- No production-ready implementation
- No MSI installer
- No Windows service
- No deployment guarantees
- Architecture documented only

**Note:** This component is a standalone agent and is intentionally excluded from the unified installer and systemd model. This README documents planned architecture only. No production-ready standalone agent is currently implemented. No installer or lifecycle guarantees apply.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
