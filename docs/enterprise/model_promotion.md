# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/model_promotion.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Model promotion specification - controlled promotion of candidate models to ACTIVE state with human approval

# Model Promotion Specification (PROMPT-61 Phase 4)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Controlled promotion of candidate models to ACTIVE state with human approval, generating promotion audit and attestation.

---

## Promotion Rules

### If Approved

- Promote candidate → ACTIVE
- Update `model_registry.is_active = true`
- Deactivate previous active model
- Generate promotion audit + attestation

### If Not Approved

- Candidate expires automatically (30 days)
- Expiration logged to `immutable_audit_log`
- Action: `CANDIDATE_EXPIRATION`

---

## Promotion Process

### Step 1: Validation

1. Check candidate expiration (30 days)
2. Check regression gate passed
3. Verify candidate exists in registry

### Step 2: Deactivate Current Model

- Set `model_registry.is_active = false` for current model
- Update `model_registry.updated_at = now()`

### Step 3: Activate Candidate

- Set `model_registry.is_active = true` for candidate
- Update `model_registry.updated_at = now()`

### Step 4: Generate Audit

- Create promotion audit entry in `immutable_audit_log`
- Action: `MODEL_PROMOTION`
- Include approver identifier
- Compute chain hash for audit chain

### Step 5: Generate Attestation

- Create attestation file: `/var/lib/ransomeye/attestations/promotion_{model_name}_{version}_{timestamp}.json`
- Include promotion metadata
- Signed by approver

---

## Candidate Expiration

### Automatic Expiration

- Candidates older than 30 days automatically expire
- Expiration checked during promotion process
- Expired candidates cannot be promoted

### Expiration Logging

```json
{
  "action": "CANDIDATE_EXPIRATION",
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "created_at": "2026-01-28T12:00:00Z",
  "expired_at": "2026-02-27T12:00:00Z"
}
```

---

## Promotion Audit

### Audit Entry

```json
{
  "action": "MODEL_PROMOTION",
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "approver": "admin@ransomeye.tech",
  "timestamp": "2026-01-28T12:00:00Z",
  "promotion_type": "CANDIDATE_TO_ACTIVE"
}
```

### Chain Hash

- Computed from previous audit entry chain hash + current payload SHA-256
- Ensures audit chain integrity

---

## Attestation File

### Location

`/var/lib/ransomeye/attestations/promotion_{model_name}_{version}_{timestamp}.json`

### Content

```json
{
  "action": "MODEL_PROMOTION",
  "model_name": "threat_delta_classifier",
  "version": "candidate-20260128120000",
  "approver": "admin@ransomeye.tech",
  "timestamp": "2026-01-28T12:00:00Z",
  "promotion_type": "CANDIDATE_TO_ACTIVE",
  "previous_model": "threat_delta_classifier_v1.0",
  "gate_passed": true,
  "metrics": {
    "accuracy": 0.95,
    "precision": 0.92,
    "recall": 0.88,
    "f1_score": 0.90
  }
}
```

---

## Implementation

### Module: `core/ai/registry/promote_candidate.py`

**Functions:**

- `CandidatePromoter.check_candidate_expired()` - Check if candidate expired
- `CandidatePromoter.check_gate_passed()` - Verify regression gate passed
- `CandidatePromoter.deactivate_current_model()` - Deactivate current model
- `CandidatePromoter.promote_candidate()` - Promote candidate to ACTIVE
- `CandidatePromoter.generate_promotion_audit()` - Generate audit and attestation
- `CandidatePromoter.expire_candidates()` - Expire old candidates

**Usage:**

```bash
python3 /home/ransomeye/rebuild/core/ai/registry/promote_candidate.py \
    --model-name threat_delta_classifier \
    --version candidate-20260128120000 \
    --approver admin@ransomeye.tech
```

**Force Promotion (Skip Checks):**

```bash
python3 /home/ransomeye/rebuild/core/ai/registry/promote_candidate.py \
    --model-name threat_delta_classifier \
    --version candidate-20260128120000 \
    --approver admin@ransomeye.tech \
    --force
```

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Candidate expired → FAIL-CLOSED (unless --force)
3. Gate not passed → FAIL-CLOSED (unless --force)
4. Promotion failure → FAIL-CLOSED
5. Audit generation failure → FAIL-CLOSED

---

## Integration

### Upstream Systems

- **Regression Gate** - Must pass before promotion
- **Shadow Retraining** - Produces candidates for promotion

### Downstream Systems

- **Model Registry** - Tracks active model versions
- **Inference Pipeline** - Uses active models
- **Verifier** - Validates active models

---

## Human-in-the-Loop

### Approval Required

- Promotion requires explicit human approval
- Approver identifier must be provided
- No automatic promotion (fail-closed)

### Approval Tracking

- Approver logged in audit entry
- Approver included in attestation file
- Full audit trail maintained

---

## Last Updated

PROMPT-61 Phase 4 Implementation

