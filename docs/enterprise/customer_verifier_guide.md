# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/customer_verifier_guide.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Customer verifier guide - standalone verifier package that customers can run independently without trusting RansomEye operators

# Customer Verifier Guide (PROMPT-63 Phase 1)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Customer verifier bundle is a standalone verifier package that customers can run independently without trusting RansomEye operators, vendors, or support teams.

---

## Features

### Binary Hash Verification

- Verifies all binaries against `ARTIFACT_HASHES.txt`
- Detects tampering or unauthorized modifications
- No operator trust required

### Model Hash Verification

- Verifies model files against exported registry
- Ensures model integrity
- Validates SHAP artifacts

### Audit Chain Verification

- Verifies audit chain integrity
- Validates hash chaining
- Detects chain breaks or tampering

### Drift Snapshot Comparison

- Compares current state against baseline
- Detects unauthorized changes
- Validates system integrity

### Claim Verification

- Verifies marketing/sales claims
- Validates against evidence
- No unverifiable claims

### Configuration Sanity

- Checks for hardcoded secrets
- Validates localhost-first configuration
- Ensures no exposed credentials

---

## Rules

### No DB Credentials Required

- Uses read-only exports
- No database access needed
- Works with exported data

### No Network Access Required

- Fully offline-capable
- No internet connectivity needed
- Air-gapped operation

### Cryptographically Signed Result

- Customer generates signature
- Result is tamper-evident
- Verifiable independently

---

## Usage

### Basic Verification

```bash
python3 /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py
```

### With Exported Data

```bash
python3 /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py \
    --exports-dir /path/to/exports
```

### Exported Data Structure

```
exports/
├── model_registry_export.json
├── audit_chain_export.json
├── drift_snapshot_export.json
└── claims_verification_export.json
```

---

## Verification Results

### Result Format

```json
{
  "verified_at": "2026-01-28T12:00:00Z",
  "verifier_version": "1.0.0",
  "checks": {
    "binary_hashes": {
      "verified": true,
      "messages": ["Verified 50 artifacts"]
    },
    "model_hashes": {
      "verified": true,
      "messages": ["Verified 10 models"]
    },
    "audit_chain": {
      "verified": true,
      "messages": ["Verified 100 audit chain entries"]
    },
    "drift_snapshot": {
      "verified": true,
      "messages": ["No drift detected"]
    },
    "claims": {
      "verified": true,
      "messages": ["Verified 6 claims"]
    },
    "configuration": {
      "verified": true,
      "messages": ["No hardcoded secrets detected"]
    }
  },
  "overall_verified": true,
  "failures": [],
  "warnings": [],
  "customer_signature": "sha256:..."
}
```

---

## Zero-Trust Principles

### No Operator Trust

- Customer runs verifier independently
- No operator involvement required
- No operator trust assumed

### Cryptographic Verification

- All checks use cryptographic methods
- Hash-based verification
- Tamper-evident results

### Independent Validation

- Customer validates all evidence
- No reliance on operator statements
- Fully customer-controlled

---

## Fail-Closed Enforcement

### Failure Conditions

1. Binary hash mismatch → FAIL
2. Model hash mismatch → FAIL
3. Audit chain break → FAIL
4. Drift detected → FAIL
5. Unverified claims → FAIL
6. Hardcoded secrets → WARNING

---

## Integration

### Upstream Systems

- **Proof Snapshot Generator** - Provides exported data
- **Auditor Envelope** - Provides audit chain sample
- **Claims Verifier** - Provides claims verification

### Downstream Systems

- **Customer Attestation** - Uses verification results
- **Legal Proceedings** - Uses verification evidence
- **Compliance Audits** - Uses verification proof

---

## Last Updated

PROMPT-63 Phase 1 Implementation

