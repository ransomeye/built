# Phase 8: AI Advisory & SOC Copilot

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/08_UI_Dashboards_SOC_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 8 - AI Advisory & SOC Copilot

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/ai/`
- **Main Module:** `inference/src/lib.rs`
- **No separate service** - Integrated into correlation/policy flow

### Core Components

1. **Advisory Inference** (`inference/src/inference.rs`)
   - Advisory-only inference
   - Risk scoring
   - Context enrichment
   - No enforcement authority

2. **Model Loader** (`inference/src/loader.rs`)
   - Loads signed models
   - Validates model signatures
   - Model versioning
   - Model rollback

3. **Confidence Calibrator** (`inference/src/calibration.rs`)
   - Confidence calibration
   - Calibrated output generation
   - Confidence bounds

4. **Feature Extractor** (`inference/src/features.rs`)
   - Feature extraction
   - Feature normalization
   - Feature validation

5. **SHAP Explainability** (`explainability/src/`)
   - SHAP explanation generation
   - Mandatory SHAP for all outputs
   - SHAP baseline validation
   - Output blocked if SHAP missing

6. **RAG Knowledge Base** (`rag/src/`)
   - Pre-indexed RAG knowledge
   - Document embeddings
   - Search index
   - Read-only analyst assistance

7. **Model Registry** (`registry/src/`)
   - Model registry management
   - Model versioning
   - Model verification
   - Dependency management

---

## WHAT DOES NOT EXIST

1. **No separate systemd service** - AI Advisory integrated into other services
2. **No UI** - UI handled by Phase 11
3. **No enforcement authority** - Advisory only
4. **No policy decisions** - Policy handled by Phase 6

---

## DATABASE SCHEMAS

**NONE** - Phase 8 does not create database tables directly.

**Model Storage:**
- Models stored in filesystem
- Model metadata in JSON files
- RAG index in filesystem

---

## RUNTIME SERVICES

**NONE** - Phase 8 has no separate systemd service.

**Integration:**
- AI Advisory integrated into correlation engine
- AI Advisory integrated into policy engine
- No standalone service

---

## GUARDRAILS ALIGNMENT

Phase 8 enforces guardrails:

1. **Advisory Only** - AI cannot enforce policy
2. **Signed Models Only** - Unsigned models rejected
3. **SHAP Required** - Outputs without SHAP blocked
4. **Baseline Required** - Missing baseline → AI DISABLED
5. **Fail-Closed** - Any failure → AI DISABLED

---

## INSTALLER BEHAVIOR

**Installation:**
- AI Advisory components installed by main installer
- Models installed by Phase 3 (Intelligence)
- No separate installation step

---

## SYSTEMD INTEGRATION

**NONE** - Phase 8 has no systemd service.

**Integration:**
- AI Advisory called by correlation engine
- AI Advisory called by policy engine
- No standalone service

---

## AI/ML/LLM TRAINING REALITY

**Baseline Models:**
- Provided by Phase 3 (Intelligence)
- Baseline Intelligence Pack contains pre-trained models
- Models must be signed and verified
- SHAP explainability required

**Model Training:**
- Models trained offline
- Training provenance tracked
- Models signed before deployment

**RAG Knowledge Base:**
- Pre-indexed at release time
- Document embeddings generated offline
- Search index built offline
- No live indexing at install

---

## COPILOT REALITY

**SOC Copilot:**
- Read-only analyst assistance
- RAG knowledge base queries
- Context enrichment
- Risk scoring
- No enforcement authority
- Advisory recommendations only

---

## UI REALITY

**NONE** - Phase 8 has no UI.

**UI Integration:**
- UI handled by Phase 11
- AI Advisory outputs consumed by UI
- No direct UI in Phase 8

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Missing Baseline** → AI DISABLED
2. **Invalid Signature** → AI DISABLED
3. **Missing SHAP** → OUTPUT BLOCKED
4. **Trust Chain Failure** → AI DISABLED
5. **Revoked Models** → AI DISABLED

**No Bypass:**
- No `--skip-baseline` flag
- No `--skip-shap` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 8 is fully implemented and production-ready:

✅ **Complete Implementation**
- Advisory inference functional
- Model loading working
- SHAP explainability working
- RAG knowledge base complete
- SOC Copilot functional

✅ **Guardrails Alignment**
- Advisory only (no enforcement)
- Signed models only
- SHAP required
- Baseline required
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- AI disabled on any failure

**Note:** Phase 8 provides AI Advisory and SOC Copilot functionality. UI is handled by Phase 11.

**Recommendation:** Deploy as-is. Phase 8 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
