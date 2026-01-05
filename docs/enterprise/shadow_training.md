# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/shadow_training.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Shadow retraining specification - executes shadow retraining using delta-only data without activating models

# Shadow Retraining Specification (PROMPT-61 Phase 2)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Shadow retraining executes retraining using delta-only data and produces candidate model versions without activation.

---

## Rules

### No Auto-Deploy

- Training artifacts signed
- Stored under `/var/lib/ransomeye/models/candidates/`
- Registry entries marked `state = CANDIDATE`
- Models NOT activated automatically

### Delta-Only Data

- Uses `threat_intel_delta` table data only
- No baseline data used
- Ensures models adapt to new threats only

### Training Artifacts

- Model file: `{model_name}_candidate.pkl`
- SHAP baseline: `{model_name}_candidate_shap.pkl`
- Metadata: `{model_name}_candidate_metadata.json`

---

## Training Process

### Step 1: Load Delta Data

- Query `threat_intel_delta` table
- Extract features from delta records
- Generate labels (new IOC/mutation = 1, other = 0)

### Step 2: Train Model

- Split data: 80% train, 20% test
- Train RandomForestClassifier
- Evaluate: accuracy, precision, recall, F1

### Step 3: Generate SHAP Baseline

- Use TreeExplainer
- Compute SHAP values for training sample
- Save SHAP baseline artifact

### Step 4: Register Candidate

- Create/update model entry in `model_registry`
- Register version with `state = CANDIDATE`
- Store metadata in `model_versions.metadata_json`

---

## Model Registry State

### Candidate State

```json
{
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "state": "CANDIDATE",
  "model_path": "/var/lib/ransomeye/models/candidates/threat_delta_classifier_candidate.pkl",
  "shap_path": "/var/lib/ransomeye/models/candidates/threat_delta_classifier_candidate_shap.pkl",
  "model_hash": "sha256:...",
  "metrics": {
    "accuracy": 0.95,
    "precision": 0.92,
    "recall": 0.88,
    "f1_score": 0.90
  },
  "training_samples": 800,
  "test_samples": 200,
  "created_at": "2026-01-28T12:00:00Z"
}
```

### Registry Entry

- `model_registry.is_active = false` (candidate not active)
- `model_versions.version = "candidate-{timestamp}"`
- `model_versions.metadata_json` contains full metadata

---

## Implementation

### Module: `core/ai/training/shadow_retrain.py`

**Functions:**

- `ShadowRetrainer.load_delta_data()` - Load delta data from database
- `ShadowRetrainer.extract_features()` - Extract features from delta records
- `ShadowRetrainer.train_candidate_model()` - Train candidate model
- `ShadowRetrainer.register_candidate()` - Register in model registry

**Usage:**

```bash
python3 /home/ransomeye/rebuild/core/ai/training/shadow_retrain.py
```

---

## Feature Extraction

### Delta Features

1. **Delta Type** - Encoded: new_ioc=1.0, mutation=2.0, confidence_shift=3.0, ttp_pattern=4.0
2. **IOC Type** - Encoded: ip=1.0, domain=2.0, hash=3.0, url=4.0, email=5.0
3. **Source Hash** - Hash-based encoding (0.0-1.0)
4. **Confidence Change** - Absolute difference
5. **Correlation Count Change** - Absolute difference
6. **Tag Count** - New tag count
7. **Tag Count Change** - Absolute difference

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Delta data load failure → WARNING (no data available)
3. Training failure → FAIL-CLOSED
4. SHAP generation failure → FAIL-CLOSED
5. Registration failure → FAIL-CLOSED

---

## Integration

### Downstream Systems

- **Regression Gate** - Evaluates candidates
- **Promotion System** - Promotes candidates to ACTIVE
- **Model Registry** - Tracks candidate versions

---

## Last Updated

PROMPT-61 Phase 2 Implementation

