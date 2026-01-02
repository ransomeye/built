# RansomEye Intelligence System

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_intelligence/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 3 - Intelligence System & Day-1 AI Readiness

---

## Overview

Phase 3 builds the **Intelligence Plane** with fully trained baseline ML models, signed Baseline Intelligence Pack, threat intelligence fusion, SHAP explainability, and pre-indexed LLM RAG knowledge.

**Zero enforcement authority** - Intelligence Plane is advisory only.

---

## Key Components

### 1. Baseline Intelligence Pack

**Location:** `baseline_pack/`

**Contents:**
- Pre-trained ransomware behavior models
- Pre-trained anomaly baselines
- Confidence calibration curves
- SHAP baseline distributions
- Feature schemas
- Training provenance

**Startup Rule:** If Baseline Pack is missing, invalid, unsigned, or expired → **AI MUST NOT START**

---

### 2. Threat Intelligence

**Location:** `threat_intel/`

**Components:**
- Offline-capable feed ingestion
- Feed validation and poisoning detection
- IOC normalization and ontology
- Multi-source correlation
- Confidence scoring

**Rule:** Threat intelligence is **advisory only** and never overrides Control Plane decisions.

---

### 3. AI Registry

**Location:** `ai_registry/`

**Components:**
- Model registry and versioning
- Model verification
- Dependency management
- Rollback capabilities

### 3.5. Model Registry (Canonical)

**Location:** `model_registry/`

**Canonical Registry:** `model_registry/registry.json`

**Governance Rule:** Only models explicitly registered in `model_registry/registry.json` are subject to SHAP validation, metadata requirements, and signature verification. Unregistered models (including dependencies, test artifacts, and files in `.venv/` or `site-packages/`) are explicitly excluded from validation.

---

### 4. LLM RAG Knowledge

**Location:** `llm_knowledge/`

**Components:**
- Pre-indexed RAG knowledge base
- Document embeddings
- Search index
- Signed documents

**Rule:** RAG index built at release time, no live indexing at install.

---

### 5. Security & Trust

**Location:** `security/`

**Components:**
- Signature verification
- Trust chain validation
- Revocation checking

**Rule:** All intelligence artifacts must be signed and verified.

---

## Day-1 Requirements

### Requirement 1: Baseline Pack Present

- All models present
- All SHAP files present
- All signatures valid
- All metadata valid

### Requirement 2: No Empty Start

- AI cannot start without baseline
- No runtime learning without approval
- No model training at startup
- No warm-up period

### Requirement 3: SHAP Mandatory

- SHAP present for every numeric inference
- Baseline SHAP present at install
- SHAP schema validated
- Missing SHAP → inference blocked

---

## Failure Modes

All failures are **fail-closed**:

1. **Baseline Pack Missing** → AI disabled, system continues
2. **Baseline Pack Invalid** → AI disabled, system continues
3. **Baseline Pack Unsigned** → AI disabled, system continues
4. **SHAP Missing** → Inference blocked
5. **Model Corruption** → Model disabled, rollback
6. **Threat Intel Poisoning** → Feed rejected
7. **RAG Index Corruption** → RAG disabled

---

## Testing

### Rust Tests

```bash
cd /home/ransomeye/rebuild/ransomeye_intelligence/tests
cargo test
```

**Test Suites:**
- `baseline_presence_tests.rs` - Baseline pack presence and validity
- `shap_completeness_tests.rs` - SHAP completeness for all models
- `intel_poisoning_tests.rs` - Threat intelligence poisoning detection

---

## Initialization

```python
from ransomeye_intelligence.intelligence_controller import IntelligenceController

controller = IntelligenceController()
if controller.initialize():
    print("✓ Intelligence Plane ready")
else:
    print("✗ Intelligence Plane initialization failed")
    # System continues without AI
```

---

## Key Guarantees

1. **AI Never Starts Empty** - Baseline pack required
2. **No Runtime Learning** - No training without approval
3. **SHAP Mandatory** - Every inference has SHAP
4. **Advisory Only** - Zero enforcement authority
5. **Fail-Closed** - AI disabled on failure, system continues

---

## Last Updated

Phase 3 Implementation - Complete

