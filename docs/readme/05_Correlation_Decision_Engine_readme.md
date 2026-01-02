# Phase 5: Correlation & Decision Engine

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/05_Correlation_Decision_Engine_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 5 - Correlation & Decision Engine

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/engine/`
- **Service:** `systemd/ransomeye-correlation.service`
- **Main Module:** `correlation/src/lib.rs`

### Core Components

1. **Correlation Engine** (`correlation/src/engine.rs`)
   - Main orchestrator
   - Entity state management
   - Kill-chain inference
   - Confidence scoring
   - Temporal correlation
   - Entity graph building

2. **Entity State Manager** (`correlation/src/entity_state.rs`)
   - Tracks entity states
   - Manages entity lifecycle
   - Entity TTL enforcement
   - State transitions

3. **Kill-Chain Inferencer** (`correlation/src/kill_chain/`)
   - MITRE ATT&CK kill-chain inference
   - Stage detection
   - Signal-to-stage mapping
   - Kill-chain progression tracking

4. **Confidence Scorer** (`correlation/src/scoring.rs`)
   - Confidence calculation
   - Signal contribution analysis
   - Threshold management
   - Confidence breakdown

5. **Temporal Correlator** (`correlation/src/temporal.rs`)
   - Temporal window management
   - Event ordering
   - Time-based correlation
   - Window-based analysis

6. **Entity Graph** (`correlation/src/graph.rs`)
   - Entity relationship graph
   - Graph traversal
   - Relationship inference
   - Graph export

7. **Explainability** (`correlation/src/explainability.rs`)
   - SHAP explainability integration
   - Signal explanations
   - Stage explanations
   - Confidence breakdowns
   - Explainability artifacts

8. **Invariant Enforcer** (`correlation/src/invariants.rs`)
   - Deterministic invariants
   - Invariant validation
   - Fail-closed on invariant violation

9. **Entity Scheduler** (`correlation/src/scheduler.rs`)
   - Entity processing scheduling
   - Priority management
   - Resource allocation

10. **Input/Output** (`correlation/src/input/`, `correlation/src/output/`)
    - Validated event input
    - Detection result output
    - Schema validation

---

## WHAT DOES NOT EXIST

1. **No database persistence in correlation** - Correlation does not write to database directly (if implemented, handled separately)
2. **No policy evaluation** - Policy handled by Phase 6
3. **No enforcement** - Enforcement handled by Phase 7
4. **No AI model training** - Models provided by Phase 3

---

## DATABASE SCHEMAS

**NONE** - Phase 5 does not create database tables directly.

**If persistence implemented:**
- Detection results may be stored
- Entity states may be persisted
- Correlation artifacts may be stored

---

## RUNTIME SERVICES

**Service:** `ransomeye-correlation.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-correlation.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-ingestion.service`
- **ExecStart:** `/usr/bin/ransomeye_operations start ransomeye-correlation`

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/correlation, /run/ransomeye/correlation

---

## GUARDRAILS ALIGNMENT

Phase 5 enforces guardrails:

1. **Deterministic Correlation** - Same input → Same output (always)
2. **No AI Dependency** - Correlation is deterministic, no ML models
3. **Fail-Closed** - Ambiguity → No detection
4. **Invariant Enforcement** - Invariant violations → Fail-closed
5. **Explainability** - All outputs include SHAP explanations

---

## INSTALLER BEHAVIOR

**Installation:**
- Correlation service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Service disabled by default
- Binary built from Rust crate

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

**NONE** - Phase 5 does not train models.

**Correlation is deterministic:**
- Rule-based correlation
- No ML inference
- No model loading
- Explainability via SHAP (from Phase 3/8)

---

## COPILOT REALITY

**NONE** - Phase 5 does not provide copilot functionality.

**Correlation outputs:**
- Detection results
- Confidence scores
- Explainability artifacts
- Used by Phase 6 (Policy) and Phase 8 (AI Advisory)

---

## UI REALITY

**NONE** - Phase 5 has no UI.

**Correlation metrics:**
- Available via systemd journal
- Detection results logged
- Entity state changes logged
- Confidence scores logged

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Input** → Detection REJECTED
2. **Invariant Violation** → Detection REJECTED
3. **Ambiguous Correlation** → No detection
4. **Missing Context** → No detection
5. **Explainability Failure** → Output BLOCKED

**No Bypass:**
- No `--skip-validation` flag
- No `--skip-invariants` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 5 is fully implemented and production-ready:

✅ **Complete Implementation**
- Correlation engine functional
- Kill-chain inference working
- Confidence scoring working
- Temporal correlation working
- Entity graph building working
- Explainability integration complete

✅ **Guardrails Alignment**
- Deterministic correlation
- No AI dependency
- Fail-closed behavior
- Invariant enforcement
- Explainability required

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Detections rejected on any failure

**Recommendation:** Deploy as-is. Phase 5 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
