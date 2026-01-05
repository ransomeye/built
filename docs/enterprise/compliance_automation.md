# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/compliance_automation.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Compliance Automation Documentation (PROMPT-58-B)

# Compliance Automation (PROMPT-58-B)

## Overview

Automated monthly compliance evidence generation ensures continuous legal and audit readiness without manual intervention.

## Purpose

- **Legal Compliance**: Generate retention and lineage proofs
- **Audit Readiness**: Provide evidence for security audits
- **AI Explainability**: Document SHAP explanations
- **Automated**: Zero manual intervention required

## Execution

### Manual Execution

```bash
python3 /home/ransomeye/rebuild/core/compliance/compliance_automation.py
```

### Scheduled Execution

Monthly via systemd timer (see `ransomeye-compliance-automation.timer`)

## Generated Evidence

### 1. Audit Retention Proof

**Location**: `compliance_report.json` → `audit_retention_proof`

**Contents**:
- Retention policy (years/days)
- Oldest and newest audit entries
- Total entry count
- Expired entries count (must be 0)
- Compliance status

**Purpose**: Prove audit log retention policy is enforced

### 2. Data Lineage Proof

**Location**: `compliance_report.json` → `data_lineage_proof`

**Contents**:
- Raw events statistics
- Normalized events statistics
- Audit log chain integrity
- Chain continuity samples

**Purpose**: Prove data flow from ingestion to storage with integrity

### 3. AI Explainability Samples

**Location**: `compliance_report.json` → `ai_explainability_samples`

**Contents**:
- Total SHAP explanations
- Model count
- Sample explanations (latest 10)
- Model registry information

**Purpose**: Prove AI decisions are explainable (SHAP)

## Storage

### Directory Structure

```
/docs/enterprise/compliance/monthly/
  YYYY-MM/
    compliance_report.json
```

### File Format

JSON with:
- `report_timestamp`: ISO 8601 timestamp
- `report_period`: YYYY-MM format
- `audit_retention_proof`: Retention evidence
- `data_lineage_proof`: Lineage evidence
- `ai_explainability_samples`: SHAP evidence

### Protection

- File permissions: `444` (read-only)
- Immutable after creation
- Never modified, only appended

## Acceptance Criteria

- [x] Job runs without manual input
- [x] Evidence immutable and timestamped
- [x] All three proof types generated
- [x] Stored in monthly directory structure
- [x] Read-only permissions set

## Compliance Mapping

### GDPR

- **Data Retention**: Audit retention proof
- **Data Lineage**: Data lineage proof
- **Right to Explanation**: AI explainability samples

### SOC 2

- **Audit Logging**: Audit retention proof
- **Data Integrity**: Data lineage proof
- **Change Management**: Evidence of compliance processes

### NIST

- **Audit Controls**: Audit retention proof
- **System Integrity**: Data lineage proof
- **AI Transparency**: AI explainability samples

## Maintenance

### Monthly Execution

Automated via systemd timer on 1st of each month at 00:00 UTC.

### Manual Trigger

For immediate compliance evidence:

```bash
sudo systemctl start ransomeye-compliance-automation.service
```

### Review

Monthly reports should be reviewed quarterly for:
- Retention policy compliance
- Data integrity
- SHAP coverage

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

