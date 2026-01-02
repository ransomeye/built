# Phase 19: HNMP Engine (Host & Network Management Posture)

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/19_HNMP_Engine_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 19 - HNMP Engine

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_posture_engine/`
- **Service:** `systemd/ransomeye-posture-engine.service`
- **Type:** Python service
- **Status:** FULLY IMPLEMENTED (Same as Phase 15)

### Core Components

**Note:** Phase 19 (HNMP Engine) is the same as Phase 15 (Posture Engine). See Phase 15 README for complete details.

1. **Posture Daemon** (`engine/posture_daemon.py`)
   - Main orchestrator
   - Host compliance evaluation
   - Fleet scoring
   - Drift detection

2. **Compliance Scanner** (`engine/cis_evaluator.py`, `engine/stig_evaluator.py`, `engine/custom_policy_evaluator.py`)
   - CIS Benchmark evaluation
   - STIG profile evaluation
   - Custom policy evaluation

3. **Health Score** (`engine/scorer.py`)
   - Host posture scores (0.0 to 1.0)
   - Network posture scores
   - Fleet health average
   - Weighted scoring

4. **Report Generator** (`engine/report_generator.py`)
   - Compliance reports (PDF, HTML, CSV)
   - Score dashboards
   - Drift alerts

---

## WHAT DOES NOT EXIST

1. **No separate HNMP service** - Same as Phase 15 (Posture Engine)
2. **No additional HNMP-specific features** - All functionality in Phase 15

**Canonical Mapping (from MODULE_PHASE_MAP.yaml):**
- `ransomeye_hnmp_engine` → PHANTOM MODULE (not yet created)
- Functionality provided by `ransomeye_posture_engine` (Phase 15/19)

---

## DATABASE SCHEMAS

**NONE** - Phase 19 does not create database tables.

**Database Usage:**
- Reads telemetry from PostgreSQL (created by other phases)
- Queries signed telemetry events
- Filters by producer type (linux_agent, windows_agent, dpi_probe)

---

## RUNTIME SERVICES

**Service:** `ransomeye-posture-engine.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-posture-engine.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-correlation.service`
- **ExecStart:** Python module execution

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/posture, /run/ransomeye/posture

---

## GUARDRAILS ALIGNMENT

Phase 19 enforces guardrails (same as Phase 15):

1. **Ed25519 Signing ONLY** - RSA prohibited
2. **Database UNTRUSTED** - Every telemetry record verified with Ed25519
3. **Policy Hash Pinning** - Every evaluation includes policy SHA-256 hash, version, source path
4. **Deterministic Logic Only** - NO ML
5. **Advisory Only** - Zero enforcement authority
6. **Fail-Closed** - Ambiguity → Non-compliant

---

## INSTALLER BEHAVIOR

**Installation:**
- Posture engine service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Python package installed
- Service disabled by default

---

## SYSTEMD INTEGRATION

**Service File:**
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Restart always
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 19 does not use AI/ML models.

**Posture evaluation is deterministic:**
- Rule-based evaluation
- CIS/STIG/Custom policy evaluation
- No ML inference
- No model loading

---

## COPILOT REALITY

**NONE** - Phase 19 does not provide copilot functionality.

**Posture Engine provides:**
- Compliance evaluation
- Posture scoring
- Fleet health scoring
- Drift detection
- Advisory reports

---

## UI REALITY

**NONE** - Phase 19 has no UI.

**Posture reports:**
- Generated as PDF, HTML, CSV
- Score dashboards exported
- Can be viewed in any compatible viewer
- UI integration handled by Phase 11

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Telemetry Signature** → REJECTED
2. **Policy Hash Mismatch** → REJECTED
3. **Ambiguous Condition** → Non-compliant
4. **Missing Context** → Non-compliant
5. **Database Corruption** → REJECTED

**No Bypass:**
- No `--skip-signature` flag
- No `--skip-policy-hash` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 19 is fully implemented and production-ready:

✅ **Complete Implementation**
- Posture daemon functional (same as Phase 15)
- Compliance scanning working
- Health scoring working
- Fleet scoring working
- Drift detection working
- Report generation working

✅ **Guardrails Alignment**
- Ed25519 signing only
- Database untrusted (every record verified)
- Policy hash pinning
- Deterministic logic only
- Advisory only
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Telemetry rejected on any failure

**Note:** Phase 19 (HNMP Engine) is the same as Phase 15 (Posture Engine). Both refer to the same implementation: `ransomeye_posture_engine`.

**Recommendation:** Deploy as-is. Phase 19 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
