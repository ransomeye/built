# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/disclosure_control_register.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Controlled Disclosure Register - Disclosure boundary register defining what can be shown publicly, what requires NDA, and what must never be disclosed (PROMPT-68-D)

# Controlled Disclosure Register (PROMPT-68-D)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **COMPLETE**

---

## Overview

This document defines **disclosure boundaries** for RansomEye v1.0.0-enterprise-ship, specifying what can be shown publicly, what requires NDA, what must never be disclosed, and how disclosures are audited.

**Purpose:** Enable controlled disclosure for customers, regulators, investors, and auditors without weakening security posture, governance, or legal boundaries.

**Constraint:** Proof-based disclosure only, no weakening of zero-trust posture, no leakage of sensitive internals.

---

## 1. Public Disclosure (No Restrictions)

### 1.1 Public Documentation

**What Can Be Shown Publicly:**
- Product overview and architecture (high-level)
- Feature descriptions (functional capabilities)
- Deployment options (air-gapped, network-connected)
- Onboarding procedures (general steps)
- Customer verification procedures (public commands)

**Examples:**
- `/docs/enterprise/regulator_briefing_note.md` (public version)
- `/docs/enterprise/customer_proof_demo_pack.md` (public version)
- `/docs/enterprise/legal_non_claims.md` (public version)

**Constraints:**
- ❌ No internal file paths
- ❌ No database credentials
- ❌ No customer-specific configuration
- ❌ No threat intelligence feeds
- ❌ No full audit chain with customer data

---

### 1.2 Public Evidence Artifacts

**What Can Be Shown Publicly:**
- Ship seal verification outputs (hashes only, no paths)
- Customer finality verification results (anonymized)
- Evidence bundle index (non-sensitive artifacts)
- Governance artifact list (documentation references)

**Examples:**
- `/artifacts/customer_demo_evidence/ship_seal_verification.json` (public version)
- `/artifacts/customer_demo_evidence/evidence_bundle_index.json` (public version)
- `/artifacts/customer_demo_evidence/governance_artifacts.json` (public version)

**Constraints:**
- ❌ No full file paths
- ❌ No customer-specific data
- ❌ No threat intelligence feeds
- ❌ No full audit chain with customer data

---

### 1.3 Public Verification Procedures

**What Can Be Shown Publicly:**
- Customer verification commands (public commands)
- Evidence bundle generation procedures (general steps)
- Ship seal verification procedures (general steps)

**Examples:**
- `python3 core/customer_verifier/customer_verify.py --check ship_finality`
- `bash scripts/generate_evidence_bundle.sh`
- `systemctl status ransomeye-verifier.timer`

**Constraints:**
- ❌ No internal file paths
- ❌ No database credentials
- ❌ No customer-specific configuration

---

## 2. NDA-Required Disclosure

### 2.1 Technical Architecture Details

**What Requires NDA:**
- Detailed technical architecture (component interactions)
- Internal file paths (beyond customer-visible paths)
- Database schema details (table structures, relationships)
- Systemd service configurations (detailed settings)
- Evidence bundle generation scripts (full implementation)

**Examples:**
- `/core/assurance/ship_seal_enforcer.py` (full code)
- `/core/customer_verifier/customer_verify.py` (full code)
- `/core/governance/vendor_non_repudiation.py` (full code)
- `/scripts/generate_evidence_bundle.sh` (full script)

**NDA Requirements:**
- ✅ Mutual NDA required
- ✅ Technical review only
- ✅ No redistribution permitted
- ✅ Audit trail required

---

### 2.2 Evidence Artifacts (Detailed)

**What Requires NDA:**
- Full evidence bundle contents (detailed artifacts)
- Audit chain samples (with anonymized data)
- Verifier failure demonstrations (detailed outputs)
- Vendor non-repudiation scan results (detailed findings)

**Examples:**
- `/artifacts/evidence_bundle_v1.0.0.tar.gz` (full bundle)
- `/artifacts/customer_demo_evidence/verifier_failure_demo_redacted.json` (detailed version)
- `/var/lib/ransomeye/governance/vendor_non_repudiation_scan.json` (full results)

**NDA Requirements:**
- ✅ Mutual NDA required
- ✅ Technical review only
- ✅ No redistribution permitted
- ✅ Audit trail required

---

### 2.3 Operational Procedures (Detailed)

**What Requires NDA:**
- Detailed production operations playbook (full procedures)
- Detailed onboarding runbooks (full steps)
- Detailed incident response procedures (full matrix)
- Detailed change prohibition register (full prohibitions)

**Examples:**
- `/docs/enterprise/production_operations_playbook.md` (full version)
- `/docs/enterprise/onboarding_runbook_airgap.md` (full version)
- `/docs/enterprise/incident_response_matrix.md` (full version)
- `/docs/enterprise/production_change_prohibitions.md` (full version)

**NDA Requirements:**
- ✅ Mutual NDA required
- ✅ Technical review only
- ✅ No redistribution permitted
- ✅ Audit trail required

---

### 2.4 Certification and Compliance Details

**What Requires NDA:**
- Detailed certification mapping matrix (full mappings)
- Detailed auditor access model (full access model)
- Detailed audit replay guide (full procedures)
- Detailed executive attestation templates (full templates)

**Examples:**
- `/docs/enterprise/certification_mapping_matrix.md` (full version)
- `/docs/enterprise/auditor_access_model.md` (full version)
- `/docs/enterprise/audit_replay_guide.md` (full version)
- `/docs/enterprise/executive_attestation_template.md` (full version)

**NDA Requirements:**
- ✅ Mutual NDA required
- ✅ Technical review only
- ✅ No redistribution permitted
- ✅ Audit trail required

---

## 3. Never Disclose (Strict Prohibition)

### 3.1 Secrets and Credentials

**What Must Never Be Disclosed:**
- Database credentials (DB_USER, DB_PASS, DB_HOST, DB_PORT)
- API keys (MISP_KEY, OTX_KEY, TALOS_KEY, THREATFOX_KEY)
- Encryption keys (DB_ENCRYPTION_KEY_PATH)
- Certificate private keys (AGENT_CERT_PATH, PROBE_CERT_PATH)
- Any hardcoded secrets or credentials

**Examples:**
- ❌ `.env` files with credentials
- ❌ Configuration files with secrets
- ❌ Certificate private keys
- ❌ API key files

**Enforcement:**
- ✅ Static code scan for secrets
- ✅ Configuration sanity check
- ✅ Customer verifier check
- ✅ Audit trail for any disclosure attempts

---

### 3.2 Customer-Specific Data

**What Must Never Be Disclosed:**
- Customer-specific configuration
- Customer-specific audit chain entries
- Customer-specific threat intelligence
- Customer-specific incident data
- Customer-specific evidence bundles

**Examples:**
- ❌ Full audit chain with customer data
- ❌ Customer-specific configuration files
- ❌ Customer-specific threat intelligence feeds
- ❌ Customer-specific incident response data

**Enforcement:**
- ✅ Evidence bundle anonymization
- ✅ Audit chain sample anonymization
- ✅ Customer data redaction
- ✅ Audit trail for any disclosure attempts

---

### 3.3 Internal Network Topology

**What Must Never Be Disclosed:**
- Internal network paths
- Internal IP addresses
- Internal service endpoints
- Internal network topology
- Internal security boundaries

**Examples:**
- ❌ Internal file paths (beyond customer-visible paths)
- ❌ Internal IP addresses
- ❌ Internal service endpoints
- ❌ Internal network topology diagrams

**Enforcement:**
- ✅ Path redaction in evidence bundles
- ✅ IP address anonymization
- ✅ Service endpoint redaction
- ✅ Audit trail for any disclosure attempts

---

### 3.4 Threat Intelligence Feeds

**What Must Never Be Disclosed:**
- Threat intelligence feed sources
- Threat intelligence feed contents
- Threat intelligence feed API keys
- Threat intelligence feed configurations
- Threat intelligence feed internal processing

**Examples:**
- ❌ MISP feed contents
- ❌ OTX feed contents
- ❌ TALOS feed contents
- ❌ THREATFOX feed contents

**Enforcement:**
- ✅ Threat intelligence feed redaction
- ✅ API key redaction
- ✅ Feed source redaction
- ✅ Audit trail for any disclosure attempts

---

### 3.5 Model Training Data

**What Must Never Be Disclosed:**
- Model training data
- Model training procedures
- Model training configurations
- Model training internal processing
- Model training customer-specific data

**Examples:**
- ❌ Training dataset contents
- ❌ Training procedure details
- ❌ Training configuration files
- ❌ Training customer-specific data

**Enforcement:**
- ✅ Training data redaction
- ✅ Training procedure redaction
- ✅ Training configuration redaction
- ✅ Audit trail for any disclosure attempts

---

## 4. Disclosure Audit Trail

### 4.1 Disclosure Logging

**What Is Logged:**
- Disclosure recipient (customer, regulator, investor, auditor)
- Disclosure type (public, NDA-required, never disclose)
- Disclosure content (artifact path, document name)
- Disclosure timestamp (date, time, timezone)
- Disclosure approval (approver, approval timestamp)

**Log Format:**
```json
{
  "disclosure_id": "UUID",
  "recipient": "Customer/Regulator/Investor/Auditor",
  "disclosure_type": "PUBLIC/NDA_REQUIRED/NEVER_DISCLOSE",
  "content": {
    "artifact_path": "/path/to/artifact",
    "document_name": "document_name.md",
    "version": "1.0.0"
  },
  "timestamp": "2026-01-28T12:00:00Z",
  "approver": "approver_name",
  "approval_timestamp": "2026-01-28T12:00:00Z"
}
```

**Log Location:**
- `/var/log/ransomeye/disclosure_audit.log`
- Append-only log file
- Immutable audit trail entry

---

### 4.2 Disclosure Approval Process

**Approval Requirements:**
- **Public Disclosure:** No approval required (pre-approved artifacts)
- **NDA-Required Disclosure:** Mutual NDA required, technical review approval
- **Never Disclose:** Strict prohibition, no exceptions

**Approval Workflow:**
1. Disclosure request submitted
2. Disclosure type determined (public, NDA-required, never disclose)
3. Approval obtained (if required)
4. Disclosure executed
5. Disclosure logged to audit trail

**Approval Authority:**
- **Public Disclosure:** Pre-approved artifacts only
- **NDA-Required Disclosure:** Technical review team approval
- **Never Disclose:** No approval possible (strict prohibition)

---

### 4.3 Disclosure Violation Detection

**Violation Detection:**
- Static code scan for secrets in public artifacts
- Configuration sanity check for hardcoded credentials
- Customer verifier check for sensitive paths
- Audit trail review for unauthorized disclosures

**Violation Response:**
- Violation logged to immutable audit trail
- Violation triggers fail-closed behavior (if applicable)
- Violation reported to security team
- Violation remediation (if possible)

**Violation Evidence:**
- Disclosure audit log entry
- Static code scan results
- Configuration sanity check results
- Customer verifier check results

---

## 5. Disclosure Boundaries by Audience

### 5.1 Customer Disclosure

**Public Disclosure:**
- Product overview and architecture (high-level)
- Feature descriptions (functional capabilities)
- Customer verification procedures (public commands)
- Evidence bundle index (non-sensitive artifacts)

**NDA-Required Disclosure:**
- Detailed technical architecture (component interactions)
- Full evidence bundle contents (detailed artifacts)
- Detailed operational procedures (full playbooks)
- Detailed onboarding runbooks (full steps)

**Never Disclose:**
- Database credentials
- Customer-specific data
- Internal network topology
- Threat intelligence feeds

---

### 5.2 Regulator Disclosure

**Public Disclosure:**
- Regulator briefing note (public version)
- Legal non-claims declaration (public version)
- Evidence bundle index (non-sensitive artifacts)
- Governance artifact list (documentation references)

**NDA-Required Disclosure:**
- Detailed regulator walkthrough (full procedures)
- Detailed certification mapping matrix (full mappings)
- Detailed auditor access model (full access model)
- Detailed audit replay guide (full procedures)

**Never Disclose:**
- Database credentials
- Customer-specific data
- Internal network topology
- Threat intelligence feeds

---

### 5.3 Investor Disclosure

**Public Disclosure:**
- Product overview and architecture (high-level)
- Feature descriptions (functional capabilities)
- Customer verification procedures (public commands)
- Evidence bundle index (non-sensitive artifacts)

**NDA-Required Disclosure:**
- Detailed technical diligence pack (full pack)
- Detailed technical architecture (component interactions)
- Detailed evidence artifacts (detailed artifacts)
- Detailed operational procedures (full playbooks)

**Never Disclose:**
- Database credentials
- Customer-specific data
- Internal network topology
- Threat intelligence feeds

---

### 5.4 Auditor Disclosure

**Public Disclosure:**
- Auditor access model (public version)
- Evidence bundle index (non-sensitive artifacts)
- Governance artifact list (documentation references)
- Customer verification procedures (public commands)

**NDA-Required Disclosure:**
- Detailed auditor access model (full access model)
- Detailed audit replay guide (full procedures)
- Detailed evidence bundle contents (detailed artifacts)
- Detailed certification mapping matrix (full mappings)

**Never Disclose:**
- Database credentials
- Customer-specific data
- Internal network topology
- Threat intelligence feeds

---

## 6. Disclosure Control Enforcement

### 6.1 Technical Enforcement

**Static Code Scan:**
- Scan public artifacts for secrets
- Scan NDA-required artifacts for sensitive paths
- Scan never-disclose artifacts for violations

**Configuration Sanity Check:**
- Check for hardcoded credentials
- Check for sensitive paths
- Check for customer-specific data

**Customer Verifier Check:**
- Verify no secrets in customer-visible paths
- Verify no sensitive paths in evidence bundles
- Verify no customer-specific data in public artifacts

---

### 6.2 Procedural Enforcement

**Disclosure Approval:**
- Public disclosure: Pre-approved artifacts only
- NDA-required disclosure: Mutual NDA required, technical review approval
- Never disclose: Strict prohibition, no exceptions

**Disclosure Logging:**
- All disclosures logged to audit trail
- Disclosure recipient, type, content, timestamp
- Disclosure approval (if required)

**Disclosure Violation Response:**
- Violation logged to immutable audit trail
- Violation triggers fail-closed behavior (if applicable)
- Violation reported to security team
- Violation remediation (if possible)

---

### 6.3 Audit Trail Enforcement

**Disclosure Audit Log:**
- Append-only log file
- Immutable audit trail entry
- Cryptographically verifiable

**Disclosure Audit Review:**
- Regular audit review (monthly)
- Violation detection and response
- Compliance verification

**Disclosure Audit Evidence:**
- Disclosure audit log entries
- Static code scan results
- Configuration sanity check results
- Customer verifier check results

---

## 7. Disclosure Control Matrix

| Artifact | Public | NDA-Required | Never Disclose |
|----------|--------|--------------|----------------|
| Product overview | ✅ | ❌ | ❌ |
| Technical architecture (high-level) | ✅ | ❌ | ❌ |
| Technical architecture (detailed) | ❌ | ✅ | ❌ |
| Customer verification procedures | ✅ | ❌ | ❌ |
| Evidence bundle index (non-sensitive) | ✅ | ❌ | ❌ |
| Evidence bundle contents (detailed) | ❌ | ✅ | ❌ |
| Ship seal verification outputs | ✅ | ❌ | ❌ |
| Audit chain samples (anonymized) | ✅ | ❌ | ❌ |
| Audit chain samples (detailed) | ❌ | ✅ | ❌ |
| Vendor non-repudiation scan summary | ✅ | ❌ | ❌ |
| Vendor non-repudiation scan results (detailed) | ❌ | ✅ | ❌ |
| Operational procedures (general) | ✅ | ❌ | ❌ |
| Operational procedures (detailed) | ❌ | ✅ | ❌ |
| Database credentials | ❌ | ❌ | ✅ |
| Customer-specific data | ❌ | ❌ | ✅ |
| Internal network topology | ❌ | ❌ | ✅ |
| Threat intelligence feeds | ❌ | ❌ | ✅ |
| Model training data | ❌ | ❌ | ✅ |

---

## 8. Disclosure Control Compliance

### 8.1 Compliance Verification

**Regular Verification:**
- Monthly disclosure audit review
- Quarterly static code scan for secrets
- Quarterly configuration sanity check
- Quarterly customer verifier check

**Compliance Evidence:**
- Disclosure audit log entries
- Static code scan results
- Configuration sanity check results
- Customer verifier check results

**Compliance Reporting:**
- Monthly disclosure audit report
- Quarterly compliance verification report
- Annual disclosure control assessment

---

### 8.2 Compliance Violation Response

**Violation Detection:**
- Static code scan for secrets
- Configuration sanity check for hardcoded credentials
- Customer verifier check for sensitive paths
- Audit trail review for unauthorized disclosures

**Violation Response:**
- Violation logged to immutable audit trail
- Violation triggers fail-closed behavior (if applicable)
- Violation reported to security team
- Violation remediation (if possible)

**Violation Evidence:**
- Disclosure audit log entry
- Static code scan results
- Configuration sanity check results
- Customer verifier check results

---

## Conclusion

**PROMPT-68-D COMPLETE**

RansomEye v1.0.0-enterprise-ship now provides a **controlled disclosure register** that enables:

- ✅ Public disclosure boundaries (no restrictions)
- ✅ NDA-required disclosure boundaries (mutual NDA required)
- ✅ Never-disclose boundaries (strict prohibition)
- ✅ Disclosure audit trail (all disclosures logged)
- ✅ Disclosure control enforcement (technical and procedural)

All disclosures are **proof-based, evidence-backed, and auditable** without weakening security posture, governance, or legal boundaries.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

