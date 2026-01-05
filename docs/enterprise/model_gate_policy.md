# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/model_gate_policy.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Model gate policy - automated gate that must pass before candidate models can be promoted

# Model Gate Policy (PROMPT-61 Phase 3)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Automated gate that must pass before candidate models can be promoted to ACTIVE state.

---

## Gate Criteria

### 1. Accuracy ≥ Current

**Requirement:** Candidate accuracy must be ≥ current active model accuracy.

**Check:**
- Get current model metrics from `model_registry` + `model_versions`
- Get candidate metrics from `model_versions.metadata_json`
- Compare: `candidate.accuracy >= current.accuracy`

**Failure:** Accuracy regression detected

---

### 2. False Positive Rate ≤ Current

**Requirement:** Candidate false positive rate must be ≤ current FPR.

**Check:**
- Compute FPR: `1.0 - precision`
- Compare: `candidate.fpr <= current.fpr`

**Failure:** FPR increase detected

---

### 3. SHAP Coverage = 100%

**Requirement:** Candidate must have SHAP explainability enabled and artifact present.

**Check:**
- `model_versions.shap_enabled = true`
- `model_versions.shap_artifact_uri` exists
- SHAP artifact file exists on filesystem

**Failure:** SHAP coverage not 100%

---

### 4. Drift = 0

**Requirement:** No system drift detected.

**Check:**
- Read `/var/lib/ransomeye/verifier/drift_snapshot.json`
- Verify no drift indicators present

**Failure:** Drift detected

---

### 5. Verifier Green ≥ 24h

**Requirement:** Verifier must report healthy system for ≥ 24 hours.

**Check:**
- Read `/var/log/ransomeye/verifier_results.json`
- Verify timestamp ≥ 24 hours ago
- Verify `overall_healthy = true`

**Failure:** Verifier not green for 24h

---

## Gate Outcome

### PASS

- Candidate eligible for human approval
- Gate result logged to `immutable_audit_log`
- Action: `REGRESSION_GATE_EVALUATION` with `passed = true`

### FAIL

- Candidate discarded
- Gate result logged to `immutable_audit_log`
- Action: `REGRESSION_GATE_EVALUATION` with `passed = false` and `failures` list

---

## Implementation

### Module: `core/ai/gates/regression_gate.py`

**Functions:**

- `RegressionGate.get_current_model_metrics()` - Get current active model metrics
- `RegressionGate.get_candidate_metrics()` - Get candidate model metrics
- `RegressionGate.check_shap_coverage()` - Verify SHAP coverage
- `RegressionGate.check_verifier_status()` - Verify verifier green ≥ 24h
- `RegressionGate.check_drift()` - Verify no drift
- `RegressionGate.evaluate_candidate()` - Evaluate all criteria
- `RegressionGate.log_audit()` - Log gate evaluation

**Usage:**

```bash
python3 /home/ransomeye/rebuild/core/ai/gates/regression_gate.py \
    --model-name threat_delta_classifier \
    --version candidate-20260128120000
```

---

## Audit Logging

### Gate Evaluation Entry

```json
{
  "action": "REGRESSION_GATE_EVALUATION",
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "passed": true,
  "failures": [],
  "timestamp": "2026-01-28T12:00:00Z"
}
```

### Failure Entry

```json
{
  "action": "REGRESSION_GATE_EVALUATION",
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "passed": false,
  "failures": [
    "Accuracy regression: 0.92 < 0.95",
    "SHAP coverage not 100%"
  ],
  "timestamp": "2026-01-28T12:00:00Z"
}
```

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Current model not found → WARNING (assume baseline)
3. Candidate not found → FAIL-CLOSED
4. Gate evaluation failure → FAIL-CLOSED
5. Audit logging failure → WARNING (evaluation continues)

---

## Integration

### Upstream Systems

- **Shadow Retraining** - Produces candidates for evaluation

### Downstream Systems

- **Promotion System** - Only promotes gate-passed candidates
- **Model Registry** - Tracks gate evaluation results

---

## Last Updated

PROMPT-61 Phase 3 Implementation

