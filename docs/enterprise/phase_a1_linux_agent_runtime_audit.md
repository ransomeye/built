# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/phase_a1_linux_agent_runtime_audit.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase A1 - Linux Agent Runtime Audit Report

# Phase A1 - Linux Agent Runtime Audit Report

**Date:** 2026-01-05  
**Auditor:** Automated Enterprise Hardening Validation  
**Status:** COMPLETED WITH FINDINGS

---

## A1.1 - Systemd Unit Verification

### Service Configuration
- **Service File:** `/etc/systemd/system/ransomeye-linux-agent.service`
- **User:** `ransomeye-agent` (uid=995, gid=987) ✅
- **Group:** `ransomeye-agent` ✅
- **Working Directory:** `/opt/ransomeye-linux-agent` ✅
- **Type:** `notify` (systemd watchdog enabled) ✅

### Root Privilege Verification
- **NoNewPrivileges:** `true` ✅
- **User/Group:** `ransomeye-agent` (non-root) ✅
- **Privilege Escalation:** Disabled ✅

### Capability-Based Access
- **CapabilityBoundingSet:** `CAP_SYS_PTRACE CAP_DAC_READ_SEARCH CAP_NET_RAW CAP_NET_ADMIN` ✅
- **CAP_SYS_ADMIN:** NOT PRESENT ✅
- **AmbientCapabilities:** Empty (no ambient capabilities) ✅

### Security Hardening
- **ProtectSystem:** `strict` ✅
- **ProtectHome:** `true` ✅
- **PrivateTmp:** `true` ✅
- **MemoryDenyWriteExecute:** `true` ✅
- **RestrictSUIDSGID:** `true` ✅
- **NoNewPrivileges:** `true` ✅

---

## A1.2 - Signing Key Permissions

### Current State
- **Key Path:** `/etc/ransomeye/keys/linux_agent_signing.key`
- **Current Permissions:** `-rw-r-----` (640) ❌
- **Required Permissions:** `-rw-------` (600) ❌
- **Owner:** `ransomeye:ransomeye-agent` ✅
- **Group:** `ransomeye-agent` ✅

### Finding
**CRITICAL:** Signing key permissions are 640, should be 600 for security compliance.

**Remediation:** Execute `chmod 600 /etc/ransomeye/keys/linux_agent_signing.key`

---

## A1.3 - Binary Hash Verification

### Implementation Status
- **Binary Integrity Check:** Implemented in `hardening.rs` ✅
- **Hash Algorithm:** SHA-256 ✅
- **Verification Points:**
  - Startup verification ✅
  - Periodic runtime checks (every 1000 events) ✅
  - Watchdog thread verification ✅

### Trust Chain Integration
- **Binary Hash Storage:** Computed at initialization ✅
- **Hash Comparison:** SHA-256 comparison on each check ✅
- **Tamper Detection:** Fail-closed on hash mismatch ✅

### Core Trust Chain
- **Trust Chain Module:** `security/trust_chain.rs` exists ✅
- **Integration:** Binary hash verification independent of Core (standalone) ✅
- **Note:** Standalone agent does not require Core trust chain validation (by design)

---

## A1.4 - Fail-Closed Behavior Verification

### Missing Config
- **Implementation:** `AgentConfig::from_env()` returns error on missing required ENV ✅
- **Behavior:** Agent fails to start ✅
- **Location:** `agent/src/main.rs:82-86` ✅

### Missing Signing Key
- **Implementation:** `SecurityEventSigner::from_key_file()` returns error ✅
- **Behavior:** Agent fails to start with `AgentError::SigningFailed` ✅
- **Location:** `agent/src/main.rs:100-109` ✅

### Ingestion Unreachable
- **Implementation:** HTTP client timeout (10 seconds) ✅
- **Behavior:** Events are queued/buffered, agent continues running ✅
- **Note:** This is graceful degradation, not fail-closed (by design for telemetry collection)

### Invalid TLS / Cert
- **Implementation:** TLS validation in HTTP client ✅
- **Behavior:** Connection fails, events buffered ✅
- **Note:** Agent continues running (graceful degradation for telemetry)

---

## A1.5 - Service Startup Issues

### Current State
- **Service Status:** `activating (start)` - timing out ❌
- **Restart Counter:** 57 (excessive restarts) ❌
- **Timeout:** 60 seconds (TimeoutStartSec) ❌
- **Root Cause:** Service not sending systemd notify ready signal within 60 seconds

### Analysis
Service is configured with `Type=notify` but may not be calling `sd_notify(0, "READY=1")` within the timeout window. This could be due to:
1. Long initialization time (hardening checks, eBPF setup)
2. Missing systemd notify call
3. Blocking operations during startup

### Recommendation
- Review agent startup sequence for blocking operations
- Ensure `sd_notify(0, "READY=1")` is called after initialization
- Consider increasing `TimeoutStartSec` if initialization is legitimately long

---

## Summary

### Passed Checks
1. ✅ Runs as `ransomeye-agent` (non-root)
2. ✅ No `CAP_SYS_ADMIN` capability
3. ✅ Capability-based access only
4. ✅ Binary hash verification implemented
5. ✅ Fail-closed on missing config
6. ✅ Fail-closed on missing signing key
7. ✅ Security hardening enabled

### Failed Checks
1. ❌ Signing key permissions (640 vs 600 required)
2. ❌ Service startup timeout (not sending ready signal)

### Blocking Issues
1. **Signing key permissions must be 600** (security requirement)
2. **Service startup timeout must be resolved** (operational requirement)

---

## Conclusion

**Phase A1 Status:** COMPLETED WITH FINDINGS

The Linux agent runtime configuration is mostly compliant with enterprise hardening requirements. Two issues require remediation:
1. Signing key permissions (critical security finding)
2. Service startup timeout (operational issue)

Both issues are fixable and do not indicate fundamental design flaws.

