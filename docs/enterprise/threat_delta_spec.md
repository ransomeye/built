# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/threat_delta_spec.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Threat delta capture specification - controlled delta capture comparing new threat intel vs frozen baseline

# Threat Delta Capture Specification (PROMPT-61 Phase 1)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Threat delta capture compares new threat intelligence against the last frozen baseline and classifies deltas for controlled model refresh.

---

## Delta Classification

### Delta Types

1. **NEW_IOC**
   - New IOC not present in baseline
   - Indicates new threat intelligence

2. **IOC_MUTATION**
   - IOC value changed
   - Indicates IOC evolution or correction

3. **CONFIDENCE_SHIFT**
   - Confidence score changed significantly (>0.1 threshold)
   - Indicates confidence recalculation

4. **TTP_PATTERN**
   - Tags or metadata changed
   - Indicates TTP pattern evolution

---

## Rules

### Append-Only Storage

- No overwrite of existing IOCs
- All deltas append-only with hashes
- Fail-closed on schema or integrity mismatch

### Baseline Snapshot

- Baseline = current state of `threat_intel` table at capture time
- Baseline hash = SHA-256 of baseline JSON
- Baseline stored in `threat_intel_delta.baseline_snapshot_hash`

### Delta Hash

- Delta hash = SHA-256 of delta record (type, old_value, new_value, timestamp)
- Ensures integrity of delta records

---

## Database Schema

### Table: `threat_intel_delta`

```sql
CREATE TABLE threat_intel_delta (
    delta_id uuid PRIMARY KEY,
    baseline_snapshot_hash bytea NOT NULL,
    delta_type text NOT NULL,
    ioc_type text,
    ioc_value text,
    source text,
    old_value jsonb,
    new_value jsonb,
    delta_hash bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
```

### Indexes

- `idx_threat_intel_delta_baseline_hash` - Baseline hash lookup
- `idx_threat_intel_delta_created_at` - Time-based queries
- `idx_threat_intel_delta_type` - Delta type filtering
- `idx_threat_intel_delta_ioc` - IOC lookup

---

## Implementation

### Module: `core/intel/threat_delta_capture.py`

**Functions:**

- `ThreatDeltaCapture.get_baseline_snapshot()` - Load baseline from `threat_intel` table
- `ThreatDeltaCapture.get_current_snapshot()` - Load current state
- `ThreatDeltaCapture.classify_delta()` - Classify delta type
- `ThreatDeltaCapture.capture_deltas()` - Capture and store deltas

**Usage:**

```bash
python3 /home/ransomeye/rebuild/core/intel/threat_delta_capture.py
```

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Table creation failure → FAIL-CLOSED
3. Baseline snapshot failure → FAIL-CLOSED
4. Delta capture failure → FAIL-CLOSED

### Audit Logging

All failures logged to:
- `immutable_audit_log` table (action: `THREAT_DELTA_CAPTURE_FAILURE`)
- System logs

---

## Integration

### Downstream Systems

- **Shadow Retraining** - Uses delta data for training
- **Analytics** - Delta trend analysis
- **Alerting** - Significant delta detection

---

## Last Updated

PROMPT-61 Phase 1 Implementation

