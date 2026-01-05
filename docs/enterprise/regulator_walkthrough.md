# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/regulator_walkthrough.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regulator Walkthrough - Step-by-step verification guide for non-vendor third parties (PROMPT-65-C)

# Regulator Walkthrough (PROMPT-65-C)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This walkthrough provides a **step-by-step guide** for non-vendor third parties (regulators, auditors, courts) to independently verify RansomEye v1.0.0-enterprise-ship's immutable assurances.

**Assumptions:**
- Zero trust in vendor
- No internet access required
- No prior RansomEye knowledge required

---

## Prerequisites

### Required Access

- Read access to RansomEye installation directory (`/home/ransomeye/rebuild`)
- Read access to system logs (`/var/log/ransomeye/`)
- Read access to evidence bundle (if provided)
- Basic command-line knowledge

### Optional Access

- Database read access (for audit chain verification)
- Systemd status access (for service verification)

---

## Verification Steps

### Step 1: Verify System is Sealed

**Objective:** Verify that the system has an immutable ship seal that prevents silent modification.

#### 1.1 Verify Ship Seal Enforcer Exists

```bash
# Check if ship seal enforcer exists
ls -la /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Verify file permissions (should be readable)
file /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py
```

**Expected Result:** File exists and is readable.

#### 1.2 Verify Ship Seal Hash List Exists

```bash
# Check if ARTIFACT_HASHES.txt exists
ls -la /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Verify file is read-only (444 permissions)
stat -c "%a %n" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)
```

**Expected Result:** File exists and is read-only (444 permissions).

#### 1.3 Verify Ship Seal Hash List Content

```bash
# View hash list content
head -20 /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Count number of artifacts
grep -c "SHA256:" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
```

**Expected Result:** Hash list contains multiple artifacts with SHA-256 hashes.

#### 1.4 Run Ship Seal Enforcer

```bash
# Run ship seal enforcer
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py
```

**Expected Result:** 
- Exit code: 0 (success)
- Output: "✓ Ship seal verified - all binaries intact"

**If Failure:**
- Exit code: 1 (failure)
- Output: "SHIP SEAL VIOLATION - SYSTEM_INTEGRITY_VIOLATION"
- Check violation details in output

#### 1.5 Verify Binary Hashes Match

```bash
# Verify a sample binary hash
# Example: Verify ship seal enforcer itself
sha256sum /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Check against ARTIFACT_HASHES.txt
grep "ship_seal_enforcer.py" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
```

**Expected Result:** Computed hash matches hash in `ARTIFACT_HASHES.txt`.

---

### Step 2: Verify System Cannot Be Silently Modified

**Objective:** Verify that any modification to the system is detected and logged.

#### 2.1 Verify Verifier Integration

```bash
# Check if verifier includes ship seal check
grep -n "check_ship_seal\|ShipSealEnforcer" /home/ransomeye/rebuild/core/verifier/verifier.py
```

**Expected Result:** Verifier code contains ship seal check integration.

#### 2.2 Verify Continuous Verifier

```bash
# Check if verifier timer is active
systemctl status ransomeye-verifier.timer 2>/dev/null || echo "Timer status unavailable"

# Check verifier results
cat /var/log/ransomeye/verifier_results.json 2>/dev/null | jq . 2>/dev/null || echo "Results file unavailable"
```

**Expected Result:** 
- Timer is active (if systemd access available)
- Results show `overall_healthy: true` (if file exists)

#### 2.3 Verify Detection Time

```bash
# Check verifier audit log for recent entries
tail -20 /var/log/ransomeye/verifier_audit.log 2>/dev/null || echo "Audit log unavailable"

# Check for violation entries
grep -i "violation\|failure" /var/log/ransomeye/verifier_audit.log 2>/dev/null | tail -10 || echo "No violations found"
```

**Expected Result:** 
- Verifier runs regularly (every 5 minutes)
- No recent violations (if system is healthy)

#### 2.4 Review Tamper Simulation Evidence

```bash
# Check if tamper simulation script exists
ls -la /home/ransomeye/rebuild/tests/post_ship_tamper_simulation.sh

# Review tamper evidence documentation
cat /home/ransomeye/rebuild/docs/enterprise/post_ship_tamper_evidence.md | head -50
```

**Expected Result:** 
- Tamper simulation script exists
- Documentation describes detection within ≤5 minutes

---

### Step 3: Verify Vendor Cannot Override Controls

**Objective:** Verify that vendor engineers cannot override protections.

#### 3.1 Verify Vendor Non-Repudiation Scanner

```bash
# Check if vendor scanner exists
ls -la /home/ransomeye/rebuild/core/governance/vendor_non_repudiation.py

# Run vendor scanner
cd /home/ransomeye/rebuild
python3 core/governance/vendor_non_repudiation.py
```

**Expected Result:** 
- Exit code: 0 (success)
- Output: "✅ No critical findings - Vendor non-repudiation verified"

**If Failure:**
- Exit code: 1 (failure)
- Output: Critical findings detected
- Review findings in evidence report

#### 3.2 Review Vendor Scan Results

```bash
# Check scan results
cat /var/lib/ransomeye/governance/vendor_non_repudiation_scan.json 2>/dev/null | jq . 2>/dev/null || echo "Scan results unavailable"

# Check evidence report
cat /var/lib/ransomeye/governance/vendor_non_repudiation_evidence.md 2>/dev/null | head -50 || echo "Evidence report unavailable"
```

**Expected Result:** 
- `critical_findings: 0`
- Evidence report states "VENDOR NON-REPUDIATION VERIFIED"

#### 3.3 Verify No Backdoor Patterns

```bash
# Manually check for common backdoor patterns
grep -ri "backdoor\|vendor.*override\|engineer.*bypass" /home/ransomeye/rebuild/core/ --include="*.py" | head -10 || echo "No backdoor patterns found"
```

**Expected Result:** No backdoor patterns found (or only in comments/documentation).

#### 3.4 Verify No Override Flags

```bash
# Check for override flags in code
grep -ri "OVERRIDE\|BYPASS\|DISABLE.*PROTECTION" /home/ransomeye/rebuild/core/ --include="*.py" | grep -v "#" | head -10 || echo "No override flags found"
```

**Expected Result:** No override flags found (or only in comments/documentation).

---

### Step 4: Verify Customer Verification is Independent

**Objective:** Verify that customers can verify system integrity independently without vendor assistance.

#### 4.1 Verify Customer Verifier Exists

```bash
# Check if customer verifier exists
ls -la /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py

# Review customer verifier code
head -50 /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py
```

**Expected Result:** Customer verifier exists and is standalone (no DB credentials required).

#### 4.2 Run Customer Verifier

```bash
# Run customer verifier
cd /home/ransomeye/rebuild
python3 core/customer_verifier/customer_verify.py
```

**Expected Result:** 
- Exit code: 0 (success)
- Output: `SHIP_FINALITY_VERIFIED: true`
- Output: `overall_verified: true`

**If Failure:**
- Exit code: 1 (failure)
- Output: `SHIP_FINALITY_VERIFIED: false`
- Review failure details

#### 4.3 Verify Customer Verification Results

```bash
# Check customer verification results
ls -la /var/lib/ransomeye/customer_verification/ 2>/dev/null || echo "Results directory unavailable"

# View latest result
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json 2>/dev/null | jq . 2>/dev/null | head -50 || echo "Results unavailable"
```

**Expected Result:** 
- Results file exists
- `SHIP_FINALITY_VERIFIED: true`
- All checks passed

#### 4.4 Verify No Vendor Dependencies

```bash
# Check customer verifier for vendor dependencies
grep -i "vendor\|ransomeye.*operator\|support.*team" /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py | head -10 || echo "No vendor dependencies found"
```

**Expected Result:** No vendor dependencies (customer verifier is fully independent).

---

### Step 5: Verify Violations are Detected and Logged

**Objective:** Verify that all violations are detected and logged to immutable audit trail.

#### 5.1 Verify Audit Log Exists

```bash
# Check if audit log table exists (if database access available)
psql -h localhost -U gagan -d ransomeye -c "\d ransomeye.immutable_audit_log" 2>/dev/null || echo "Database access unavailable"
```

**Expected Result:** Audit log table exists (if database access available).

#### 5.2 Verify Audit Chain Integrity

```bash
# Export audit chain sample (if database access available)
psql -h localhost -U gagan -d ransomeye -c "
    SELECT audit_id, action, created_at, chain_hash_sha256
    FROM ransomeye.immutable_audit_log
    ORDER BY created_at DESC
    LIMIT 10;
" 2>/dev/null || echo "Database access unavailable"
```

**Expected Result:** 
- Audit chain entries exist
- Chain hashes are present
- Entries are timestamped

#### 5.3 Verify Violation Logging

```bash
# Check for violation entries in audit log
psql -h localhost -U gagan -d ransomeye -c "
    SELECT audit_id, action, created_at, payload_json->>'violation_type' as violation_type
    FROM ransomeye.immutable_audit_log
    WHERE action = 'SYSTEM_INTEGRITY_VIOLATION'
    ORDER BY created_at DESC
    LIMIT 10;
" 2>/dev/null || echo "Database access unavailable"
```

**Expected Result:** 
- Violation entries exist (if violations occurred)
- Violations are timestamped
- Violation details are logged

#### 5.4 Verify Verifier Audit Log

```bash
# Check verifier audit log
tail -50 /var/log/ransomeye/verifier_audit.log 2>/dev/null || echo "Verifier audit log unavailable"

# Check for violation entries
grep -i "violation\|failure" /var/log/ransomeye/verifier_audit.log 2>/dev/null | tail -10 || echo "No violations found"
```

**Expected Result:** 
- Verifier audit log exists
- Violations are logged (if violations occurred)
- Log entries are timestamped

---

## Verification Summary

### Checklist

- [ ] **Step 1:** System is sealed
  - [ ] Ship seal enforcer exists
  - [ ] Ship seal hash list exists and is read-only
  - [ ] Ship seal enforcer runs successfully
  - [ ] Binary hashes match

- [ ] **Step 2:** System cannot be silently modified
  - [ ] Verifier includes ship seal check
  - [ ] Continuous verifier is active
  - [ ] Detection time is ≤5 minutes
  - [ ] Tamper simulation evidence exists

- [ ] **Step 3:** Vendor cannot override controls
  - [ ] Vendor scanner runs successfully
  - [ ] No critical findings in scan
  - [ ] No backdoor patterns found
  - [ ] No override flags found

- [ ] **Step 4:** Customer verification is independent
  - [ ] Customer verifier exists
  - [ ] Customer verifier runs successfully
  - [ ] `SHIP_FINALITY_VERIFIED: true`
  - [ ] No vendor dependencies

- [ ] **Step 5:** Violations are detected and logged
  - [ ] Audit log exists
  - [ ] Audit chain integrity verified
  - [ ] Violations are logged
  - [ ] Verifier audit log exists

---

## Evidence Collection

### Required Evidence

1. **Ship Seal Verification:**
   - Ship seal enforcer output
   - Binary hash verification results
   - Ship seal hash list copy

2. **Modification Detection:**
   - Verifier integration code review
   - Verifier results
   - Tamper simulation evidence

3. **Vendor Non-Repudiation:**
   - Vendor scanner output
   - Scan results
   - Evidence report

4. **Customer Verification:**
   - Customer verifier output
   - Verification results
   - Finality flag status

5. **Violation Logging:**
   - Audit chain sample
   - Violation entries
   - Verifier audit log

---

## Troubleshooting

### Common Issues

#### Issue: Ship Seal Enforcer Fails

**Possible Causes:**
- Binary hash mismatch
- Missing ARTIFACT_HASHES.txt
- File permission issues

**Resolution:**
- Check violation details in output
- Verify ARTIFACT_HASHES.txt exists
- Check file permissions

#### Issue: Vendor Scanner Finds Critical Findings

**Possible Causes:**
- False positives in code
- Actual backdoor patterns

**Resolution:**
- Review findings in evidence report
- Verify findings are not false positives
- Check if findings are in comments/documentation

#### Issue: Customer Verifier Fails

**Possible Causes:**
- Missing ship seal components
- Hash mismatches
- Configuration issues

**Resolution:**
- Check failure details in output
- Verify all components exist
- Review verification results

---

## Conclusion

This walkthrough provides **independent verification** of RansomEye v1.0.0-enterprise-ship's immutable assurances. All steps can be performed without vendor assistance or internet access.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

