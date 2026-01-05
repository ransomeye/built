# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/forensic_chain_policy.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Forensic chain of custody policy - case ID generation, evidence sealing, custody transfer logging, and read-only export bundles

# Forensic Chain of Custody Policy (PROMPT-62 Phase 3)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Forensic chain of custody ensures legal defensibility of evidence through case management, evidence sealing, and custody transfer logging.

---

## Case Management

### Case ID Generation

- **Case ID:** UUID (internal identifier)
- **Case Number:** `CASE-YYYYMMDD-XXXXXXXX` (human-readable)
- **Case Name:** Descriptive name
- **Created By:** Custodian identifier
- **Status:** `open`, `closed`, `archived`

### Case Creation

- Cases created via CLI or API
- Case metadata stored in `forensic_cases` table
- Case number is unique and immutable

---

## Evidence Sealing

### Sealing Process

1. Evidence collected and structured
2. Evidence hash computed (SHA-256)
3. Evidence sealed with timestamp
4. Evidence stored in `forensic_evidence` table
5. Evidence marked as sealed (immutable)

### Sealing Requirements

- **Evidence Hash:** SHA-256 of evidence data
- **Sealed At:** Timestamp when sealed
- **Sealed By:** Custodian identifier
- **Evidence Type:** Classification of evidence

### Evidence Immutability

- Once sealed, evidence cannot be modified
- Evidence hash ensures integrity
- Evidence data stored as JSONB

---

## Custody Transfer Log

### Transfer Types

- **COLLECTION** - Initial evidence collection
- **TRANSFER** - Transfer between custodians
- **ANALYSIS** - Transfer to analysis team
- **ARCHIVE** - Transfer to archive
- **DESTRUCTION** - Evidence destruction (with authorization)

### Transfer Logging

- All transfers logged in `custody_transfer_log` table
- Transfer log is append-only
- Transfer hash ensures integrity
- Transfer includes: from, to, reason, timestamp

### Transfer Hash

- SHA-256 of transfer record
- Includes: case_id, evidence_id, transfer_type, from, to, reason, timestamp
- Ensures transfer log integrity

---

## Read-Only Export Bundles

### Bundle Contents

- Case metadata
- All evidence items
- All custody transfers
- Bundle hash
- Export timestamp

### Bundle Format

- JSON structure
- Compressed as `.tar.gz`
- Read-only file permissions (444)
- Verifiable offline

### Bundle Verification

- Bundle hash verifies integrity
- Evidence hashes verify individual items
- Transfer hashes verify custody chain
- All hashes verifiable offline

---

## Database Schema

### Table: `forensic_cases`

```sql
CREATE TABLE forensic_cases (
    case_id uuid PRIMARY KEY,
    case_number text NOT NULL UNIQUE,
    case_name text,
    created_at timestamptz NOT NULL DEFAULT now(),
    created_by text,
    status text NOT NULL DEFAULT 'open',
    description text
);
```

### Table: `forensic_evidence`

```sql
CREATE TABLE forensic_evidence (
    evidence_id uuid PRIMARY KEY,
    case_id uuid NOT NULL REFERENCES forensic_cases(case_id),
    evidence_type text NOT NULL,
    evidence_hash bytea NOT NULL,
    evidence_data jsonb,
    sealed_at timestamptz NOT NULL DEFAULT now(),
    sealed_by text
);
```

### Table: `custody_transfer_log`

```sql
CREATE TABLE custody_transfer_log (
    transfer_id uuid PRIMARY KEY,
    case_id uuid NOT NULL REFERENCES forensic_cases(case_id),
    evidence_id uuid REFERENCES forensic_evidence(evidence_id),
    transfer_type text NOT NULL,
    from_custodian text,
    to_custodian text,
    transfer_reason text,
    transfer_hash bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
```

---

## Implementation

### Module: `core/forensics/chain_of_custody.py`

**Functions:**

- `ChainOfCustody.generate_case_id()` - Generate case ID and number
- `ChainOfCustody.create_case()` - Create new forensic case
- `ChainOfCustody.seal_evidence()` - Seal evidence with hash
- `ChainOfCustody.log_custody_transfer()` - Log custody transfer
- `ChainOfCustody.export_case_bundle()` - Export read-only bundle

**Usage:**

```bash
# Create case
python3 /home/ransomeye/rebuild/core/forensics/chain_of_custody.py \
    --case-name "Ransomware Incident 2026-01-28" \
    --created-by "admin@ransomeye.tech"

# Export case bundle
python3 /home/ransomeye/rebuild/core/forensics/chain_of_custody.py \
    --case-name "Ransomware Incident 2026-01-28" \
    --created-by "admin@ransomeye.tech" \
    --output /path/to/case_bundle.tar.gz
```

---

## Rules

### Append-Only

- Custody transfer log is append-only
- No updates or deletes allowed
- Historical integrity preserved

### Verifiable Offline

- All hashes verifiable without database
- Bundle contains all necessary data
- Chain of custody provable offline

### Tamper-Evident

- Hash mismatches indicate tampering
- Broken chain indicates missing transfers
- Invalid signatures indicate forgery

---

## Legal Defensibility

### Court Requirements

- Complete chain of custody
- Immutable evidence
- Verifiable integrity
- Custodian accountability

### Audit Requirements

- All transfers logged
- All evidence sealed
- All hashes verifiable
- All timestamps accurate

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Table creation failure → FAIL-CLOSED
3. Evidence sealing failure → FAIL-CLOSED
4. Transfer logging failure → FAIL-CLOSED
5. Bundle export failure → FAIL-CLOSED

---

## Integration

### Upstream Systems

- **Forensic Collection** - Provides evidence for sealing
- **Incident Response** - Creates cases for incidents
- **Audit System** - Provides audit evidence

### Downstream Systems

- **Legal Proceedings** - Uses bundles for court
- **External Auditors** - Uses bundles for audit
- **Regulators** - Uses bundles for compliance

---

## Last Updated

PROMPT-62 Phase 3 Implementation

