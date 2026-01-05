# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/evidence_index.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Evidence Index - Machine-verifiable inventory of all immutable assurances (PROMPT-65-A)

# RansomEye v1.0.0-enterprise-ship Evidence Index

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **COMPLETE**

---

## Overview

This document provides a **machine-verifiable inventory** of all immutable assurances in RansomEye v1.0.0-enterprise-ship. Each entry includes artifact path, hash, verification method, failure behavior, and who can independently verify it.

---

## Evidence Categories

### 1. Ship Seal Enforcement Artifacts

#### 1.1 Ship Seal Enforcer

- **Artifact Path:** `/home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py`
- **Hash:** See `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Runtime binary self-hash check
  - Loads `ARTIFACT_HASHES.txt` and verifies all binaries
  - Self-verification of enforcer and verifier
- **Failure Behavior:** 
  - Writes `SYSTEM_INTEGRITY_VIOLATION` audit entry
  - Exits with non-zero code
  - Blocks service startup
- **Independent Verification:** 
  - ✅ Customer: Run `python3 core/assurance/ship_seal_enforcer.py`
  - ✅ Auditor: Verify hash matches `ARTIFACT_HASHES.txt`
  - ✅ Regulator: Review code and verify enforcement logic

#### 1.2 Ship Seal Hash List

- **Artifact Path:** `/home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt`
- **Hash:** See file itself (contains hashes of all artifacts)
- **Verification Method:** 
  - Read-only file (444 permissions)
  - Contains SHA-256 hashes of all production binaries
  - Parsed by ship seal enforcer at runtime
- **Failure Behavior:** 
  - If missing: Enforcer fails to load, system fails to start
  - If modified: Hash mismatch detected, violation logged
- **Independent Verification:** 
  - ✅ Customer: Verify file exists and is read-only
  - ✅ Auditor: Verify hashes match actual binaries
  - ✅ Regulator: Verify hash list integrity and completeness

#### 1.3 Ship Seal Integration

- **Artifact Path:** `/home/ransomeye/rebuild/core/verifier/verifier.py`
- **Hash:** See `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Contains `check_ship_seal()` function
  - Calls `ShipSealEnforcer` in verification loop
  - Runs every 5 minutes
- **Failure Behavior:** 
  - On violation: Writes audit entry, exits non-zero
  - Continuous verifier stops, system enters fail-closed state
- **Independent Verification:** 
  - ✅ Customer: Run `python3 core/verifier/verifier.py`
  - ✅ Auditor: Review code integration
  - ✅ Regulator: Verify enforcement is fail-closed

---

### 2. Verifier Enforcement Points

#### 2.1 Continuous Verifier

- **Artifact Path:** `/home/ransomeye/rebuild/core/verifier/verifier.py`
- **Hash:** See `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Runs every 5 minutes (systemd timer)
  - Checks all locked invariants:
    - Systemd services active
    - DB tables increasing
    - Audit actions present
    - Model registry valid
    - Threat intel current
    - Ship seal intact
    - Drift detection
- **Failure Behavior:** 
  - Writes `SYSTEM_INTEGRITY_VIOLATION` audit entry
  - Exits with non-zero code
  - System enters fail-closed state
- **Independent Verification:** 
  - ✅ Customer: Check systemd timer status
  - ✅ Auditor: Review verification logic
  - ✅ Regulator: Verify fail-closed enforcement

#### 2.2 Verifier Audit Log

- **Artifact Path:** `/var/log/ransomeye/verifier_audit.log`
- **Hash:** N/A (append-only log)
- **Verification Method:** 
  - Append-only log file
  - Contains all verification results
  - Timestamped entries
- **Failure Behavior:** 
  - If log cannot be written: Verification continues but no audit trail
  - Log entries show pass/fail status
- **Independent Verification:** 
  - ✅ Customer: Read log file
  - ✅ Auditor: Verify log integrity
  - ✅ Regulator: Review verification history

#### 2.3 Verifier Results

- **Artifact Path:** `/var/log/ransomeye/verifier_results.json`
- **Hash:** N/A (updated every 5 minutes)
- **Verification Method:** 
  - JSON file with latest verification results
  - Contains check status, failures, warnings
  - Timestamped
- **Failure Behavior:** 
  - On failure: `overall_healthy: false`
  - Contains detailed failure information
- **Independent Verification:** 
  - ✅ Customer: Read JSON file
  - ✅ Auditor: Verify results match audit log
  - ✅ Regulator: Review verification outcomes

---

### 3. Audit Chain Invariants

#### 3.1 Immutable Audit Log

- **Artifact Path:** PostgreSQL table `ransomeye.immutable_audit_log`
- **Hash:** Chain hash in `chain_hash_sha256` column
- **Verification Method:** 
  - Hash-chained entries
  - Each entry includes previous entry's hash
  - Cryptographic chain integrity
- **Failure Behavior:** 
  - If chain breaks: Integrity violation detected
  - Cannot insert entries with invalid chain hash
- **Independent Verification:** 
  - ✅ Customer: Export audit chain, verify hashes
  - ✅ Auditor: Verify chain integrity
  - ✅ Regulator: Review audit chain samples

#### 3.2 Audit Chain Export

- **Artifact Path:** `/var/lib/ransomeye/customer_verification/audit_chain_export.json`
- **Hash:** N/A (generated on demand)
- **Verification Method:** 
  - JSON export of audit chain entries
  - Includes all chain hashes
  - Verifiable offline
- **Failure Behavior:** 
  - If export fails: Customer cannot verify independently
  - Export is optional, not required for operation
- **Independent Verification:** 
  - ✅ Customer: Verify chain hashes match
  - ✅ Auditor: Verify export integrity
  - ✅ Regulator: Review audit chain samples

---

### 4. Model Governance Proofs

#### 4.1 Model Registry

- **Artifact Path:** PostgreSQL table `ransomeye.model_registry`
- **Hash:** Model hashes stored in registry
- **Verification Method:** 
  - Registry contains model metadata
  - Model versions tracked
  - SHAP explanations required
- **Failure Behavior:** 
  - If model missing: Warning logged
  - If SHAP missing: Warning logged
  - Verifier continues but flags issues
- **Independent Verification:** 
  - ✅ Customer: Export model registry
  - ✅ Auditor: Verify model governance
  - ✅ Regulator: Review model registry

#### 4.2 Model Artifacts

- **Artifact Path:** Various (see `ARTIFACT_HASHES.txt`)
- **Hash:** SHA-256 hashes in `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Ship seal enforcer verifies model hashes
  - Model registry tracks model versions
- **Failure Behavior:** 
  - If hash mismatch: Ship seal violation
  - System fails to start or stops
- **Independent Verification:** 
  - ✅ Customer: Verify model hashes
  - ✅ Auditor: Verify model integrity
  - ✅ Regulator: Review model governance

---

### 5. Customer Verifier Proofs

#### 5.1 Customer Verifier

- **Artifact Path:** `/home/ransomeye/rebuild/core/customer_verifier/customer_verify.py`
- **Hash:** See `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Standalone verifier (no DB credentials required)
  - Verifies binary hashes, model hashes, audit chain
  - Verifies ship finality
  - Produces cryptographically signed result
- **Failure Behavior:** 
  - On failure: `overall_verified: false`
  - `SHIP_FINALITY_VERIFIED` flag set to false
- **Independent Verification:** 
  - ✅ Customer: Run verifier independently
  - ✅ Auditor: Verify verifier logic
  - ✅ Regulator: Review customer verification process

#### 5.2 Customer Verification Results

- **Artifact Path:** `/var/lib/ransomeye/customer_verification/customer_verify_*.json`
- **Hash:** Customer signature in result
- **Verification Method:** 
  - JSON result with all checks
  - Customer-generated signature
  - Timestamped
- **Failure Behavior:** 
  - On failure: `overall_verified: false`
  - Failures listed in result
- **Independent Verification:** 
  - ✅ Customer: Review verification results
  - ✅ Auditor: Verify result integrity
  - ✅ Regulator: Review customer verification outcomes

---

### 6. Vendor Non-Repudiation Proofs

#### 6.1 Vendor Non-Repudiation Scanner

- **Artifact Path:** `/home/ransomeye/rebuild/core/governance/vendor_non_repudiation.py`
- **Hash:** See `ARTIFACT_HASHES.txt`
- **Verification Method:** 
  - Static code scan for backdoor patterns
  - Scans for override flags, recovery mechanisms
  - Checks for bypass code
- **Failure Behavior:** 
  - If critical findings: Exit with non-zero code
  - Findings logged to evidence report
- **Independent Verification:** 
  - ✅ Customer: Run scanner independently
  - ✅ Auditor: Verify scan results
  - ✅ Regulator: Review vendor non-repudiation proof

#### 6.2 Vendor Non-Repudiation Scan Results

- **Artifact Path:** `/var/lib/ransomeye/governance/vendor_non_repudiation_scan.json`
- **Hash:** N/A (generated on demand)
- **Verification Method:** 
  - JSON report of all findings
  - Categorized by severity
  - Filtered for false positives
- **Failure Behavior:** 
  - If critical findings: Report shows findings
  - Scanner exits with non-zero code
- **Independent Verification:** 
  - ✅ Customer: Review scan results
  - ✅ Auditor: Verify scan completeness
  - ✅ Regulator: Review vendor non-repudiation evidence

#### 6.3 Vendor Non-Repudiation Evidence

- **Artifact Path:** `/var/lib/ransomeye/governance/vendor_non_repudiation_evidence.md`
- **Hash:** N/A (generated on demand)
- **Verification Method:** 
  - Markdown evidence report
  - Detailed findings
  - Verification conclusion
- **Failure Behavior:** 
  - If critical findings: Report shows findings
  - Conclusion states vendor non-repudiation not verified
- **Independent Verification:** 
  - ✅ Customer: Review evidence report
  - ✅ Auditor: Verify evidence integrity
  - ✅ Regulator: Review vendor non-repudiation proof

---

## Verification Methods Summary

### Customer Verification

Customers can independently verify:

1. ✅ Ship seal enforcement (run enforcer)
2. ✅ Binary integrity (verify hashes)
3. ✅ Audit chain integrity (export and verify)
4. ✅ Ship finality (run customer verifier)
5. ✅ Vendor non-repudiation (run scanner)

### Auditor Verification

Auditors can independently verify:

1. ✅ All customer verification methods
2. ✅ Code review of enforcement logic
3. ✅ Audit chain integrity
4. ✅ Model governance
5. ✅ Vendor non-repudiation scan results

### Regulator Verification

Regulators can independently verify:

1. ✅ All auditor verification methods
2. ✅ Evidence bundle integrity
3. ✅ Legal non-claims declaration
4. ✅ Regulator walkthrough compliance
5. ✅ Court-defensible evidence

---

## Failure Behavior Summary

### Ship Seal Violation

- **Detection:** Immediate (service startup) or ≤5 minutes (verifier)
- **Response:** 
  - `SYSTEM_INTEGRITY_VIOLATION` audit entry
  - Service fails to start or stops
  - Verifier exits non-zero
- **Evidence:** Audit log, verifier results, violation details

### Verifier Failure

- **Detection:** Every 5 minutes
- **Response:** 
  - `SYSTEM_INTEGRITY_VIOLATION` audit entry
  - Verifier exits non-zero
  - System enters fail-closed state
- **Evidence:** Audit log, verifier results, diagnostic snapshot

### Audit Chain Break

- **Detection:** On chain hash verification
- **Response:** 
  - Chain integrity violation detected
  - Cannot insert entries with invalid chain hash
- **Evidence:** Audit chain export, chain hash verification

---

## Independent Verification Matrix

| Artifact | Customer | Auditor | Regulator |
|----------|----------|---------|-----------|
| Ship Seal Enforcer | ✅ | ✅ | ✅ |
| Ship Seal Hash List | ✅ | ✅ | ✅ |
| Continuous Verifier | ✅ | ✅ | ✅ |
| Audit Chain | ✅ | ✅ | ✅ |
| Model Registry | ✅ | ✅ | ✅ |
| Customer Verifier | ✅ | ✅ | ✅ |
| Vendor Non-Repudiation | ✅ | ✅ | ✅ |

---

## Evidence Bundle Contents

See `/docs/enterprise/evidence_bundle_guide.md` for complete evidence bundle contents.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

