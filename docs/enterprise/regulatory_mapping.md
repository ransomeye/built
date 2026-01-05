# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/regulatory_mapping.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regulatory mapping documentation - automated mapping of internal controls to regulations with evidence auto-linking

# Regulatory Mapping (PROMPT-62 Phase 2)

**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

Automated mapping of internal controls to regulations with evidence auto-linking. No manual spreadsheets required.

---

## Supported Regulations

### ISO 27001:2022

- A.9.1.1 - Access control policy
- A.9.2.1 - User registration and de-registration
- A.12.4.1 - Event logging
- A.12.6.1 - Management of technical vulnerabilities
- A.18.1.1 - Identification of applicable legislation

### SOC 2 Type II

- CC6.1 - Logical and physical access controls
- CC7.2 - System monitoring
- CC7.4 - System monitoring (vulnerability management)

### NIST SP 800-53 Rev. 5

- AC-1 - Access Control Policy
- AC-2 - Account Management
- AU-2 - Audit Events
- SI-2 - Flaw Remediation
- PM-1 - Information Security Program Plan

### GDPR (General Data Protection Regulation)

- Article 30 - Records of processing activities
- Article 32 - Security of processing

### RBI Cyber Security Framework (India)

- Control 4.1 - Access Management
- Control 5.1 - Logging and Monitoring
- Control 6.1 - Vulnerability Management

---

## Internal Controls

### Access Control

- **Description:** Access control policy and enforcement
- **Evidence Sources:** `immutable_audit_log`, `components`, `agents`
- **Mappings:** A.9.1.1, CC6.1, AC-1, GDPR-32, RBI-4.1

### Audit Logging

- **Description:** Immutable audit logging with chain hashing
- **Evidence Sources:** `immutable_audit_log`
- **Mappings:** A.12.4.1, CC7.2, AU-2, GDPR-30, RBI-5.1

### Vulnerability Management

- **Description:** Technical vulnerability management
- **Evidence Sources:** `verifier_results`, `drift_snapshot`
- **Mappings:** A.12.6.1, CC7.4, SI-2, GDPR-32, RBI-6.1

### Data Encryption

- **Description:** Data encryption at rest and in transit
- **Evidence Sources:** `immutable_audit_log`, `model_registry`
- **Mappings:** A.10.1.1, CC6.7, SC-28, GDPR-32, RBI-3.1

### Change Control

- **Description:** Change control and management
- **Evidence Sources:** `immutable_audit_log`, `drift_snapshot`
- **Mappings:** A.12.5.1, CC7.3, CM-3, GDPR-32, RBI-7.1

---

## Evidence Auto-Linking

### Evidence Sources

1. **immutable_audit_log** - Audit entries related to control
2. **verifier_results** - System health and compliance checks
3. **drift_snapshot** - System change tracking
4. **model_registry** - Model compliance status
5. **components** - Component configuration

### Evidence Collection

- Evidence automatically collected from database
- Evidence linked to controls via mappings
- Evidence samples limited to 5 per control (for performance)

---

## Implementation

### Module: `core/compliance/regulatory_mapper.py`

**Functions:**

- `RegulatoryMapper.map_control_to_regulations()` - Map control to regulations
- `RegulatoryMapper.get_control_evidence()` - Get evidence for control
- `RegulatoryMapper.generate_regulatory_report()` - Generate compliance report
- `RegulatoryMapper.save_mapping()` - Save mapping to file

**Usage:**

```bash
# Generate report for all regulations
python3 /home/ransomeye/rebuild/core/compliance/regulatory_mapper.py

# Generate report for specific regulation
python3 /home/ransomeye/rebuild/core/compliance/regulatory_mapper.py \
    --regulation iso27001

# Custom output path
python3 /home/ransomeye/rebuild/core/compliance/regulatory_mapper.py \
    --output /path/to/report.json
```

---

## Report Format

### Regulatory Report Structure

```json
{
  "generated_at": "2026-01-28T12:00:00Z",
  "regulation": "all",
  "controls": {
    "access_control": {
      "description": "Access control policy and enforcement",
      "mappings": {
        "iso27001": [{"control_id": "A.9.1.1", "control_name": "..."}],
        "soc2": [{"control_id": "CC6.1", "control_name": "..."}],
        ...
      },
      "evidence_count": 10,
      "evidence": [...]
    },
    ...
  }
}
```

---

## Data-Driven Mapping

### No Manual Spreadsheets

- All mappings defined in code
- Mappings version-controlled
- Mappings auditable
- Mappings reproducible

### Mapping Updates

- Mappings can be updated in code
- New regulations can be added
- New controls can be added
- Evidence sources can be extended

---

## Fail-Closed Enforcement

### Failure Conditions

1. Database connection failure → FAIL-CLOSED
2. Evidence collection failure → WARNING (partial evidence)
3. Report generation failure → FAIL-CLOSED
4. Mapping save failure → WARNING (report still generated)

---

## Integration

### Upstream Systems

- **Audit System** - Provides audit evidence
- **Verifier** - Provides compliance checks
- **Model Registry** - Provides model compliance status

### Downstream Systems

- **Compliance Reports** - Used in regulatory submissions
- **Auditor Envelopes** - Included in audit packages
- **Customer Reports** - Included in security reviews

---

## Last Updated

PROMPT-62 Phase 2 Implementation

