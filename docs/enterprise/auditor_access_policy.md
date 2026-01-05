# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/auditor_access_policy.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Auditor access policy - read-only audit envelope generation with cryptographic signing and time-bound validity

# Auditor Access Policy (PROMPT-62 Phase 1)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Auditor access envelope provides strictly read-only access to audit information for external auditors, regulators, customers, and courts.

---

## Access Envelope Contents

### Execution Inventory (Redacted)

- Systemd services status
- Component registry
- Model registry summary
- All secrets redacted

### Audit Chain Sample

- Sample of audit log entries (last 100)
- Chain hash verification
- Payload hashes
- Chain integrity proof

### Verifier Invariant Report

- System health status
- Check results (redacted)
- Warnings and failures count
- Overall health indicator

### Drift Snapshot

- File modification tracking
- Systemd unit changes
- Database schema changes
- Drift detection status

### Model Registry Summary

- Total models
- Active models
- Model versions
- SHAP coverage percentage

### Threat Intel Delta Summary

- Total deltas
- Deltas by type
- Recent deltas (30 days)

---

## Rules

### Read-Only Access

- No write access to database
- No service control
- No secrets exposed
- Database connection set to READ ONLY

### On-Demand Generation

- Generated on-demand via CLI
- No automatic generation
- Requires explicit request

### Cryptographic Signing

- Envelope signed with hash-based signature
- Signature included in archive
- Verifiable offline

### Time-Bound Validity

- Default validity: 72 hours
- Envelope includes `valid_until` timestamp
- Expired envelopes should not be used

---

## Implementation

### Module: `core/audit/auditor_envelope_generator.py`

**Functions:**

- `AuditorEnvelopeGenerator.get_execution_inventory()` - Get redacted execution inventory
- `AuditorEnvelopeGenerator.get_audit_chain_sample()` - Get audit chain sample
- `AuditorEnvelopeGenerator.get_verifier_invariant_report()` - Get verifier report
- `AuditorEnvelopeGenerator.get_drift_snapshot()` - Get drift snapshot
- `AuditorEnvelopeGenerator.get_model_registry_summary()` - Get model registry summary
- `AuditorEnvelopeGenerator.get_threat_intel_delta_summary()` - Get threat intel delta summary
- `AuditorEnvelopeGenerator.generate_envelope()` - Generate complete envelope
- `AuditorEnvelopeGenerator.sign_envelope()` - Sign envelope
- `AuditorEnvelopeGenerator.save_envelope()` - Save as signed archive

**Usage:**

```bash
python3 /home/ransomeye/rebuild/core/audit/auditor_envelope_generator.py
```

**With custom output:**

```bash
python3 /home/ransomeye/rebuild/core/audit/auditor_envelope_generator.py \
    --output /path/to/envelope.tar.gz
```

---

## Envelope Format

### Archive Structure

```
envelope.tar.gz
├── envelope.json      # Envelope data
└── envelope.sig       # Cryptographic signature
```

### Envelope JSON Structure

```json
{
  "envelope_id": "audit_20260128120000",
  "generated_at": "2026-01-28T12:00:00Z",
  "valid_until": "2026-01-31T12:00:00Z",
  "validity_hours": 72,
  "read_only": true,
  "envelope_hash": "sha256:...",
  "signature": "sha256:...",
  "contents": {
    "execution_inventory": {...},
    "audit_chain_sample": {...},
    "verifier_invariant_report": {...},
    "drift_snapshot": {...},
    "model_registry_summary": {...},
    "threat_intel_delta_summary": {...}
  }
}
```

---

## Security Considerations

### Secret Redaction

- All password fields redacted
- All secret/token fields redacted
- All credential fields redacted
- File paths may be redacted if sensitive

### Access Control

- Envelope generation requires database access
- Envelope files stored in `/var/lib/ransomeye/auditor_envelopes/`
- File permissions: 644 (read-only for others)

### Verification

- Envelope hash verifies integrity
- Signature verifies authenticity
- Timestamp verifies validity period

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Envelope generation failure → FAIL-CLOSED
3. Signature generation failure → FAIL-CLOSED
4. Archive creation failure → FAIL-CLOSED

---

## Integration

### Downstream Systems

- **External Auditors** - Receive envelopes for audit
- **Regulators** - Receive envelopes for compliance verification
- **Customers** - Receive envelopes for security reviews
- **Courts** - Receive envelopes for legal proceedings

---

## Last Updated

PROMPT-62 Phase 1 Implementation

