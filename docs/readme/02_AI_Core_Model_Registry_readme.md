# Phase 2: AI Core & Model Registry

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/02_AI_Core_Model_Registry_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 2 - AI Core & Model Registry

---

## WHAT EXISTS

### Implementation Location
- **Architecture Documentation:** `/home/ransomeye/rebuild/ransomeye_architecture/`
- **AI Advisory (Phase 8):** `/home/ransomeye/rebuild/core/ai/` (provides AI functionality)
- **Intelligence (Phase 3):** `/home/ransomeye/rebuild/ransomeye_intelligence/` (provides baseline models)

### Core Components

1. **Architecture Documentation** (`ransomeye_architecture/`)
   - System architecture specifications
   - Trust boundary definitions
   - Plane separation documentation
   - Zero-trust principles
   - **Type:** Documentation/library (no service)

2. **AI Advisory System** (`core/ai/`)
   - Advisory-only AI inference
   - Model registry and versioning
   - SHAP explainability
   - RAG knowledge base
   - **Note:** Functionality provided by Phase 8, not separate Phase 2 module

3. **Intelligence System** (`ransomeye_intelligence/`)
   - Baseline Intelligence Pack
   - Model registry
   - Model verification
   - **Note:** Functionality provided by Phase 3, not separate Phase 2 module

---

## WHAT DOES NOT EXIST

1. **No standalone `ransomeye_ai_core` module** - PHANTOM MODULE (removed from code)
2. **No separate model registry service** - Functionality in Phase 3 (Intelligence) and Phase 8 (AI Advisory)
3. **No dedicated AI Core service** - AI functionality distributed across Phase 3 and Phase 8
4. **No systemd service for Phase 2** - Phase 2 is documentation only

**Canonical Mapping (from MODULE_PHASE_MAP.yaml):**
- `ransomeye_ai_core` → PHANTOM MODULE
- Functionality provided by `ransomeye_ai_advisory` (Phase 8)
- Architecture documentation in `ransomeye_architecture` (Phase 2)

---

## DATABASE SCHEMAS

**NONE** - Phase 2 (architecture documentation) does not interact with databases.

**Model Registry:**
- **Canonical Registry:** `/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/registry.json`
- Only models explicitly listed in `registry.json` are governed
- Model metadata stored in filesystem (not database)
- Baseline Intelligence Pack stored in filesystem
- Model manifests in JSON format
- **Governance Rule:** Only models explicitly registered in `model_registry/registry.json` are subject to SHAP validation, metadata requirements, and signature verification. Unregistered models (including dependencies, test artifacts, and files in `.venv/` or `site-packages/`) are explicitly excluded from validation.

---

## RUNTIME SERVICES

**NONE** - Phase 2 is documentation only, no runtime service.

**Related Services:**
- Phase 3 (Intelligence): `ransomeye-intelligence.service` - Manages baseline models
- Phase 8 (AI Advisory): No separate service, integrated into correlation/policy flow

---

## GUARDRAILS ALIGNMENT

Phase 2 (architecture) defines guardrails:

1. **Zero-Trust Architecture** - All components untrusted by default
2. **Plane Separation** - Data, Control, Intelligence, Management planes
3. **One-Way Authority Flows** - Authority flows only from Control Plane
4. **AI Non-Authority** - AI cannot authorize enforcement
5. **Fail-Closed Defaults** - Ambiguity → DENY

**Implementation:**
- Guardrails enforced by Phase 0 (Guardrails Enforcement)
- Architecture principles guide all implementation

---

## INSTALLER BEHAVIOR

**Installation:**
- Architecture documentation installed as part of main installer
- No separate installation step
- Documentation available at `/home/ransomeye/rebuild/ransomeye_architecture/`

**Model Installation:**
- Models installed by Phase 3 (Intelligence)
- Baseline Intelligence Pack validated at startup
- Models must be signed and verified

---

## SYSTEMD INTEGRATION

**NONE** - Phase 2 has no systemd service.

**Related Services:**
- `ransomeye-intelligence.service` (Phase 3) - Manages models
- AI Advisory integrated into other services (no separate service)

---

## AI/ML/LLM TRAINING REALITY

**Baseline Models:**
- Provided by Phase 3 (Intelligence)
- Baseline Intelligence Pack contains pre-trained models
- Models must be signed and verified
- SHAP explainability required

**Model Registry:**
- Managed by Phase 3 (Intelligence)
- Model versioning and rollback
- Model verification and trust chain

**Training:**
- Models trained offline
- Training provenance tracked
- Models signed before deployment

---

## COPILOT REALITY

**SOC Copilot:**
- Provided by Phase 8 (AI Advisory)
- Advisory-only assistance
- Read-only access to data
- No enforcement authority

---

## UI REALITY

**NONE** - Phase 2 has no UI.

**Architecture Documentation:**
- Markdown files in `ransomeye_architecture/`
- No web UI or dashboard

---

## FAIL-CLOSED BEHAVIOR

**Architecture Principles:**
- All components fail-closed by default
- Ambiguity → DENY
- Missing context → DENY
- Invalid signatures → REJECT

**Implementation:**
- Fail-closed behavior implemented in each phase
- Architecture principles guide implementation

---

## FINAL VERDICT

**PARTIALLY VIABLE — HIGH RISK**

Phase 2 status:

✅ **Architecture Documentation**
- Complete architecture specifications
- Trust boundaries defined
- Plane separation documented
- Zero-trust principles established

❌ **No Standalone AI Core Module**
- `ransomeye_ai_core` is PHANTOM MODULE
- Functionality distributed across Phase 3 and Phase 8
- No dedicated model registry service

**Risk Assessment:**
- **LOW RISK** for architecture documentation (complete)
- **MEDIUM RISK** for AI Core functionality (distributed, but functional)
- Functionality exists but not as standalone Phase 2 module

**Recommendation:**
- Architecture documentation is production-ready
- AI Core functionality exists in Phase 3 and Phase 8
- Consider consolidating or documenting distribution clearly
- No critical gaps, but structure differs from original specification

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
