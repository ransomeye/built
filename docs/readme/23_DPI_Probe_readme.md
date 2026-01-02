# Phase 23: DPI Probe (Standalone)

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/23_DPI_Probe_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 23 - DPI Probe (Standalone)

---

## STATUS: ❌ NOT_IMPLEMENTED

**Guardrails Status:** NOT_IMPLEMENTED  
**Implementation Location:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/` (path exists but not production-ready)  
**Type:** Standalone probe (not installed by main installer)

---

## ⚠️ IMPORTANT DISCLAIMER

**This README documents planned architecture only.**

**No production-ready standalone probe is currently implemented.**

**No installer or lifecycle guarantees apply.**

This component is a standalone probe and is intentionally excluded from the unified installer and systemd model. Any references to implementation, deployment, or enforcement in this document are architectural plans only, not current reality.

---

## PLANNED ARCHITECTURE

### Planned Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/` (or `/home/ransomeye/rebuild/edge/dpi/`)
- **Type:** Standalone Rust binary
- **Status:** NOT_IMPLEMENTED (planned only)

### Planned Core Components

1. **Probe Main** (planned)
   - Main entry point
   - High-throughput packet capture
   - Flow assembly
   - Feature extraction
   - Event signing
   - Transport to Core

2. **Capture Engine** (planned)
   - High-throughput packet capture using libpcap
   - Passive only (no packet modification)
   - Sustained 10Gbps+ traffic handling
   - Interface selection

3. **Flow Assembler** (planned)
   - Network flow tracking
   - Packet assembly
   - Flow timeout management
   - Flow state tracking

4. **Feature Extractor** (planned)
   - Flow feature extraction
   - Metadata extraction
   - Feature normalization
   - **NO AI classification** (metadata only)

5. **Event Signing** (planned)
   - RSA-4096-PSS-SHA256 signing
   - Event signature generation
   - Signature verification

6. **Transport Client** (planned)
   - mTLS client for Core communication
   - Event transmission
   - Backpressure handling
   - Reconnection logic

7. **Disk Buffer** (planned)
   - Persistent buffering when Core unavailable
   - Fail-open behavior
   - Buffer persistence
   - Resume upload from buffer

---

## WHAT DOES NOT EXIST

1. **No production-ready probe** - Not implemented
2. **No installer** - Not implemented
3. **No systemd service** - Not implemented (standalone probes excluded from unified systemd)
4. **No lifecycle management** - Not implemented
5. **No deployment guarantees** - Not implemented

---

## DATABASE SCHEMAS

**NONE** - Phase 23 (DPI Probe) does not create database tables. Probes send telemetry to Core, which stores data.

---

## RUNTIME SERVICES

**NONE** - Phase 23 has no systemd service in the unified installer model.

**Standalone Probe Status:**
- Standalone probes are intentionally excluded from unified installer
- Standalone probes have independent lifecycle
- Standalone probes are excluded from unified systemd validation

---

## GUARDRAILS ALIGNMENT

Phase 23 (planned) would align with guardrails:

1. **Standalone Probe** - Not installed by main installer
2. **Independent Lifecycle** - Separate installer required
3. **Rootless Operation** - Must be rootless
4. **Fail-Open (Sensor)** - Core unavailability → Buffer to disk
5. **Passive Only** - Zero packet modification

**Current Status:**
- NOT_IMPLEMENTED per guardrails.yaml
- No production-ready implementation
- Architecture documented only

---

## INSTALLER BEHAVIOR

**Installation:**
- **NOT installed by main installer** - Standalone probes excluded
- **Dedicated installer required** - If implemented, would require separate installer
- **No lifecycle guarantees** - Not implemented

**Note:** This component is a standalone probe and is intentionally excluded from the unified installer and systemd model.

---

## SYSTEMD INTEGRATION

**NONE** - Phase 23 has no systemd service in unified installer model.

**Standalone Probe Policy:**
- Standalone probes are excluded from unified systemd directory
- Standalone probes may have their own systemd units (if implemented)
- Standalone probes are not validated by unified systemd rules

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 23 (DPI Probe) does not use AI/ML models. Probes extract features (metadata only), no ML classification.

---

## FAIL-CLOSED BEHAVIOR

**FAIL-OPEN (Sensor Behavior):**

1. **Core Unavailable** → Buffer to disk (fail-open)
2. **Identity Failure** → Probe DISABLED (fail-closed)
3. **Signing Failure** → Event REJECTED (fail-closed)
4. **Buffer Full** → Backpressure signal (fail-open)
5. **Packet Capture Failure** → Probe DISABLED (fail-closed)

**Note:** This behavior is planned only. No production-ready probe exists to enforce these rules.

---

## STATUS SUMMARY

❌ **NOT_IMPLEMENTED** - No production-ready standalone DPI probe exists

**Planned Components:**
- ❌ Probe main (not implemented)
- ❌ Capture engine (not implemented)
- ❌ Flow assembler (not implemented)
- ❌ Feature extractor (not implemented)
- ❌ Event signing (not implemented)
- ❌ Transport client (not implemented)
- ❌ Disk buffer (not implemented)

**Current Reality:**
- No production-ready implementation
- No installer
- No deployment guarantees
- Architecture documented only

**Note:** This component is a standalone probe and is intentionally excluded from the unified installer and systemd model. This README documents planned architecture only. No production-ready standalone probe is currently implemented. No installer or lifecycle guarantees apply.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
