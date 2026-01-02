# Phase 15: SOC Copilot (Advanced) / Posture Engine

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/15_SOC_Copilot_Advanced_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 15 - SOC Copilot (Advanced) / Posture Engine

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_posture_engine/`
- **Service:** `systemd/ransomeye-posture-engine.service`
- **Type:** Python service

### Core Components

1. **Posture Daemon** (`engine/posture_daemon.py`)
   - Main orchestrator
   - Telemetry ingestion
   - Signal normalization
   - Policy evaluation
   - Scoring
   - Drift detection
   - Report generation

2. **Telemetry Ingester** (`engine/telemetry_ingester.py`)
   - Queries PostgreSQL database
   - Filters by producer type
   - Time-range queries
   - Signature verification (Ed25519)

3. **Signal Normalizer** (`engine/normalizer.py`)
   - Normalizes telemetry into posture facts
   - Categories: Host Hardening, Auth Hygiene, Network Exposure, Drift Detection

4. **Evaluators** (`engine/`)
   - CIS Evaluator (`cis_evaluator.py`)
   - STIG Evaluator (`stig_evaluator.py`)
   - Custom Policy Evaluator (`custom_policy_evaluator.py`)

5. **Scorer** (`engine/scorer.py`)
   - Calculates host posture scores (0.0 to 1.0)
   - Calculates network posture scores
   - Weighted scoring across frameworks

6. **Drift Detector** (`engine/drift_detector.py`)
   - Compares current posture against historical baseline
   - Detects score drift, configuration drift, fact drift
   - Generates drift alerts

7. **Report Generator** (`engine/report_generator.py`)
   - Generates reports in PDF, HTML, and CSV formats
   - Footer: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
   - Includes timestamp, build hash, model version hash

8. **Output Signer** (`engine/output_signer.py`)
   - Cryptographically signs all outputs using Ed25519
   - Generates signature metadata files

9. **Audit Trail** (`engine/audit_trail.py`)
   - Maintains immutable audit log
   - JSONL format, one file per day
   - Queryable by time range, action, host

10. **Signature Verifier** (`engine/signature_verifier.py`)
    - Verifies Ed25519 signatures on telemetry
    - Database is UNTRUSTED - every record verified

11. **Policy Metadata Manager** (`engine/policy_metadata.py`)
    - Policy hash pinning
    - Policy version tracking
    - Policy source path tracking

---

## WHAT DOES NOT EXIST

1. **No multi-modal input** - Not implemented
2. **No playbook linking** - Not implemented
3. **No advanced SOC Copilot features** - Posture engine provides compliance evaluation, not advanced copilot

---

## DATABASE SCHEMAS

**NONE** - Phase 15 does not create database tables.

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

Phase 15 enforces guardrails:

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

**NONE** - Phase 15 does not use AI/ML models.

**Posture evaluation is deterministic:**
- Rule-based evaluation
- CIS/STIG/Custom policy evaluation
- No ML inference
- No model loading

---

## COPILOT REALITY

**NONE** - Phase 15 does not provide advanced SOC Copilot functionality.

**Posture Engine provides:**
- Compliance evaluation
- Posture scoring
- Drift detection
- Advisory reports
- Not advanced copilot features

**Advanced SOC Copilot:**
- Provided by Phase 8 (AI Advisory)
- Multi-modal input (not in Phase 15)
- Playbook linking (not in Phase 15)

---

## UI REALITY

**NONE** - Phase 15 has no UI.

**Posture reports:**
- Generated as PDF, HTML, CSV
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

Phase 15 is fully implemented and production-ready:

✅ **Complete Implementation**
- Posture daemon functional
- Telemetry ingestion working
- Signal normalization working
- CIS/STIG/Custom evaluation working
- Scoring working
- Drift detection working
- Report generation working
- Output signing working
- Audit trail working

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

**Note:** Phase 15 provides Posture Engine (HNMP) functionality, not advanced SOC Copilot. Advanced SOC Copilot features provided by Phase 8 (AI Advisory).

**Recommendation:** Deploy as-is. Phase 15 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
