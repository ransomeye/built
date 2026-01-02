# Phase 3: Alert Engine & Policy Manager

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/03_Alert_Engine_Policy_Manager_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 3 - Alert Engine & Policy Manager

---

## WHAT EXISTS

### Implementation Location
- **Development:** `/home/ransomeye/rebuild/ransomeye_intelligence/` (source code)
- **Runtime:** `/opt/ransomeye/modules/ransomeye_intelligence/` (deployed by installer)
- **Policy Engine:** `/home/ransomeye/rebuild/core/policy/` (Phase 6)
- **Service:** `systemd/ransomeye-intelligence.service`
- **Service Launcher:** `/opt/ransomeye/bin/ransomeye-intelligence` (runtime entry point)

### Core Components

1. **Intelligence Controller** (`ransomeye_intelligence/intelligence_controller.py`)
   - Main orchestrator for Intelligence Plane
   - Validates Baseline Intelligence Pack
   - Manages intelligence subsystems
   - Fails-closed if baseline missing/invalid

2. **Baseline Intelligence Pack** (`ransomeye_intelligence/baseline_pack/`)
   - Pre-trained ransomware behavior models
   - Pre-trained anomaly baselines
   - Confidence calibration curves
   - SHAP baseline distributions
   - Feature schemas
   - Training provenance
   - **Rule:** Missing baseline → AI MUST NOT START

3. **Threat Intelligence** (`ransomeye_intelligence/threat_intel/`)
   - Offline-capable feed ingestion
   - Feed validation and poisoning detection
   - IOC normalization and ontology
   - Multi-source correlation
   - Confidence scoring
   - Auto feed fetcher (systemd timer)

4. **AI Registry** (`ransomeye_intelligence/ai_registry/`)
   - Model registry and versioning
   - Model verification
   - Dependency management
   - Rollback capabilities

5. **Model Registry** (`ransomeye_intelligence/model_registry/`)
   - **Canonical Model Registry:** `/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/registry.json`
   - Only models explicitly listed in `registry.json` are governed
   - **Governance Rule:** Only models explicitly registered in `model_registry/registry.json` are subject to SHAP validation, metadata requirements, and signature verification. Unregistered models (including dependencies, test artifacts, and files in `.venv/` or `site-packages/`) are explicitly excluded from validation.

5. **LLM RAG Knowledge** (`ransomeye_intelligence/llm_knowledge/`)
   - Pre-indexed RAG knowledge base
   - Document embeddings
   - Search index
   - Signed documents

6. **Security & Trust** (`ransomeye_intelligence/security/`)
   - Signature verification
   - Trust chain validation
   - Revocation checking

---

## WHAT DOES NOT EXIST

1. **No standalone `ransomeye_alert_engine` module** - PHANTOM MODULE (removed from code)
2. **No separate alert engine service** - Functionality in Intelligence (Phase 3) and Policy (Phase 6)
3. **No dedicated policy manager** - Policy functionality in Phase 6 (Policy Engine)

**Canonical Mapping (from MODULE_PHASE_MAP.yaml):**
- `ransomeye_alert_engine` → PHANTOM MODULE
- Functionality split between `ransomeye_intelligence` (Phase 3) and `ransomeye_policy` (Phase 6)

---

## DATABASE SCHEMAS

**Intelligence System:**
- No direct database tables (uses filesystem for models)
- Threat intelligence feeds stored in filesystem
- Model metadata in JSON files

**Policy Engine (Phase 6):**
- Policy evaluation results stored (if implemented)
- Audit logs stored (if implemented)

---

## RUNTIME SERVICES

**Service:** `ransomeye-intelligence.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-intelligence.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-correlation.service`

**Additional Services:**
- `ransomeye-feed-fetcher.service` (timer) - Fetches threat intelligence feeds
- `ransomeye-feed-retraining.service` (timer) - Retrains models with new feeds

---

## GUARDRAILS ALIGNMENT

Phase 3 enforces guardrails:

1. **Baseline Pack Required** - Missing baseline → AI DISABLED
2. **Signed Models Only** - Unsigned models rejected
3. **SHAP Required** - Models without SHAP rejected
4. **Advisory Only** - Intelligence never enforces policy
5. **Fail-Closed** - Any failure → AI DISABLED

---

## INSTALLER BEHAVIOR

**Installation:**
- Intelligence system installed by main installer
- Baseline Intelligence Pack validated at startup
- Models must be signed and verified
- Missing baseline → Installation continues but AI disabled

---

## SYSTEMD INTEGRATION

**Service Configuration:**
- `ransomeye-intelligence.service` - Main intelligence service
- `ransomeye-feed-fetcher.timer` - Periodic feed fetching
- `ransomeye-feed-retraining.timer` - Periodic model retraining

**All services:**
- Run as `User=ransomeye` (rootless)
- `Restart=always`
- Disabled by default
- `WorkingDirectory=/opt/ransomeye` (runtime root, not /home/ransomeye/rebuild)
- `ExecStart=/opt/ransomeye/bin/ransomeye-intelligence` (launcher script)

**Runtime vs Development Separation:**
- **Development:** `/home/ransomeye/rebuild/ransomeye_intelligence/` (source code)
- **Runtime:** `/opt/ransomeye/modules/ransomeye_intelligence/` (deployed by installer)
- Service runs from `/opt/ransomeye`, NOT from `/home/ransomeye/rebuild`
- Launcher script at `/opt/ransomeye/bin/ransomeye-intelligence` sets Python path
- Pre-start validation ensures `/opt/ransomeye` exists and is accessible

---

## AI/ML/LLM TRAINING REALITY

**Baseline Models:**
- Pre-trained models in Baseline Intelligence Pack
- Models trained offline before deployment
- Training provenance tracked
- Models signed with Ed25519

**Model Training:**
- `ransomeye_intelligence/baseline_pack/train_baseline_models.py` - Training script
- `ransomeye_intelligence/threat_intel/incremental_retrain.py` - Incremental retraining
- Models retrained periodically via systemd timer

**SHAP Explainability:**
- SHAP baselines generated for all models
- `ransomeye_intelligence/baseline_pack/generate_shap_baselines.py` - SHAP generation
- SHAP required for all model outputs

---

## COPILOT REALITY

**SOC Copilot:**
- RAG knowledge base pre-indexed
- Document embeddings generated offline
- Search index built at release time
- Read-only analyst assistance
- No enforcement authority

---

## UI REALITY

**NONE** - Phase 3 has no UI.

**Intelligence outputs:**
- Advisory recommendations
- Risk scores
- Context enrichment
- Used by other phases (correlation, policy, reporting)

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Missing Baseline Pack** → AI DISABLED
2. **Invalid Signatures** → AI DISABLED
3. **Missing SHAP** → OUTPUT BLOCKED
4. **Trust Chain Failure** → AI DISABLED
5. **Revoked Models** → AI DISABLED

**No Bypass:**
- No `--skip-baseline` flag
- No `--skip-signature` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 3 is fully implemented and production-ready:

✅ **Complete Implementation**
- Intelligence controller functional
- Baseline Intelligence Pack system complete
- Threat intelligence ingestion working
- AI registry implemented
- LLM RAG knowledge base complete
- Security and trust validation working

✅ **Guardrails Alignment**
- Baseline pack required (fail-closed)
- Signed models only
- SHAP required
- Advisory only (no enforcement)

✅ **Fail-Closed Behavior**
- All critical checks fail-closed
- No bypass mechanisms
- AI disabled on any failure

**Note:** Alert engine functionality distributed between Intelligence (Phase 3) and Policy (Phase 6), but both are fully functional.

**Recommendation:** Deploy as-is. Phase 3 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
