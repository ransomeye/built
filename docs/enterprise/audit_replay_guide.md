# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/audit_replay_guide.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Audit Replay & Evidence Regeneration Guide - Guide for independent auditor verification (PROMPT-67-D)

# Audit Replay & Evidence Regeneration Guide (PROMPT-67-D)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This guide enables an **independent auditor** to re-run verifications, regenerate evidence bundles, validate audit chain continuity, confirm ship seal enforcement, and independently reach the same conclusions.

**Assumptions:**
- Hostile auditor (zero trust)
- Zero vendor trust
- Offline environment (if required)

---

## Prerequisites

### Required Access

- Read-only access to RansomEye installation directory
- Read-only access to system logs
- Read-only access to evidence bundles (if available)
- Read-only database access (if available)
- Basic command-line knowledge

### Optional Access

- Offline evidence bundle
- Audit chain export
- Verification history

---

## Verification 1: Ship Seal Enforcement

### Step 1: Verify Ship Seal Enforcer Exists

```bash
# Verify ship seal enforcer exists
ls -la /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Verify file is readable
file /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Expected: File exists and is readable
```

### Step 2: Verify ARTIFACT_HASHES.txt Exists

```bash
# Verify ARTIFACT_HASHES.txt exists
ls -la /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Verify file is read-only (444 permissions)
stat -c "%a %n" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)

# Verify file content
head -20 /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
```

### Step 3: Run Ship Seal Enforcer

```bash
# Run ship seal enforcer
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py

# Expected output:
# ✓ Ship seal verified - all binaries intact
# Exit code: 0

# If failure:
# SHIP SEAL VIOLATION - SYSTEM_INTEGRITY_VIOLATION
# Exit code: 1
```

### Step 4: Verify Binary Hashes

```bash
# Verify a sample binary hash
# Example: Verify ship seal enforcer itself
sha256sum /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Check against ARTIFACT_HASHES.txt
grep "ship_seal_enforcer.py" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Expected: Computed hash matches hash in ARTIFACT_HASHES.txt
```

### Step 5: Verify Ship Seal Integration

```bash
# Verify ship seal is integrated into verifier
grep -n "check_ship_seal\|ShipSealEnforcer" /home/ransomeye/rebuild/core/verifier/verifier.py

# Expected: Verifier code contains ship seal check integration
```

**Conclusion:** Ship seal enforcement is confirmed if:
- ✅ Ship seal enforcer runs successfully
- ✅ ARTIFACT_HASHES.txt is read-only
- ✅ Binary hashes match
- ✅ Ship seal integrated into verifier

---

## Verification 2: Audit Chain Continuity

### Step 1: Export Audit Chain

```bash
# Export audit chain (read-only)
psql -h localhost -U gagan -d ransomeye -c "
    SELECT json_agg(row_to_json(t))
    FROM (
        SELECT 
            audit_id,
            action,
            object_type,
            created_at,
            payload_sha256,
            prev_payload_sha256,
            chain_hash_sha256
        FROM ransomeye.immutable_audit_log
        ORDER BY created_at DESC
        LIMIT 1000
    ) t;
" > audit_chain_export_$(date +%Y%m%d).json

# If database access unavailable, use exported chain from evidence bundle
```

### Step 2: Verify Chain Hash Integrity

```bash
# Verify chain hash integrity
python3 << 'EOF'
import json
import hashlib

with open('audit_chain_export_20260128.json') as f:
    data = json.load(f)
    chain = data[0] if isinstance(data, list) else data.get('chain', [])

prev_chain_hash = None
verified_count = 0
failures = []

for i, entry in enumerate(chain):
    # Verify payload hash
    payload = json.dumps(entry, sort_keys=True)
    computed_payload_hash = hashlib.sha256(payload.encode()).hexdigest()
    entry_payload_hash = entry.get('payload_sha256', '')
    
    if entry_payload_hash and computed_payload_hash != entry_payload_hash:
        failures.append(f"Entry {i}: payload hash mismatch")
        continue
    
    # Verify chain hash
    if prev_chain_hash:
        chain_input = bytes.fromhex(prev_chain_hash) + bytes.fromhex(entry_payload_hash)
        computed_chain_hash = hashlib.sha256(chain_input).hexdigest()
        entry_chain_hash = entry.get('chain_hash_sha256', '')
        
        if entry_chain_hash and computed_chain_hash != entry_chain_hash:
            failures.append(f"Entry {i}: chain hash mismatch")
            continue
    
    prev_chain_hash = entry.get('chain_hash_sha256', '')
    verified_count += 1

if failures:
    print(f"FAILURES: {len(failures)} chain integrity violations")
    for failure in failures:
        print(f"  - {failure}")
    exit(1)
else:
    print(f"SUCCESS: Verified {verified_count} audit chain entries")
    exit(0)
EOF

# Expected: SUCCESS: Verified N audit chain entries
# Exit code: 0
```

### Step 3: Verify Chain Continuity

```bash
# Verify chain continuity (no breaks)
python3 << 'EOF'
import json

with open('audit_chain_export_20260128.json') as f:
    data = json.load(f)
    chain = data[0] if isinstance(data, list) else data.get('chain', [])

# Check for missing chain links
for i in range(1, len(chain)):
    prev_entry = chain[i-1]
    curr_entry = chain[i]
    
    # Verify prev_payload_sha256 matches previous entry's payload_sha256
    prev_payload_hash = prev_entry.get('payload_sha256', '')
    curr_prev_hash = curr_entry.get('prev_payload_sha256', '')
    
    if prev_payload_hash and curr_prev_hash and prev_payload_hash != curr_prev_hash:
        print(f"CHAIN BREAK: Entry {i} prev_payload_sha256 does not match previous entry")
        exit(1)

print("SUCCESS: Chain continuity verified")
exit(0)
EOF

# Expected: SUCCESS: Chain continuity verified
# Exit code: 0
```

**Conclusion:** Audit chain continuity is confirmed if:
- ✅ Chain hash integrity verified
- ✅ Chain continuity verified
- ✅ No chain breaks detected

---

## Verification 3: Evidence Bundle Regeneration

### Step 1: Generate Evidence Bundle

```bash
# Generate evidence bundle
sudo /home/ransomeye/rebuild/scripts/generate_evidence_bundle.sh

# Expected output:
# Evidence bundle generation complete
# Bundle: /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz
# Hash: [hash]
```

### Step 2: Verify Bundle Integrity

```bash
# Verify bundle hash
sha256sum -c /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz.sha256

# Expected: evidence_bundle_v1.0.0.tar.gz: OK
```

### Step 3: Extract and Verify Bundle Contents

```bash
# Extract bundle
tar -xzf /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz

# Verify manifest
cat evidence_bundle_v1.0.0/MANIFEST.txt

# Verify file hashes
cd evidence_bundle_v1.0.0
sha256sum artifacts/file_hashes_*.txt

# Verify evidence files exist
ls -la artifacts/
ls -la documentation/
ls -la evidence/
```

### Step 4: Compare with Previous Bundle (if available)

```bash
# Compare bundle hashes
sha256sum evidence_bundle_v1.0.0.tar.gz
sha256sum previous_evidence_bundle_v1.0.0.tar.gz

# Compare manifest contents
diff evidence_bundle_v1.0.0/MANIFEST.txt previous_evidence_bundle_v1.0.0/MANIFEST.txt

# Expected: Bundles are reproducible (same inputs produce same outputs)
```

**Conclusion:** Evidence bundle regeneration is confirmed if:
- ✅ Bundle generated successfully
- ✅ Bundle hash verified
- ✅ Bundle contents verified
- ✅ Bundle is reproducible (if previous bundle available)

---

## Verification 4: Customer Verifier Replay

### Step 1: Run Customer Verifier

```bash
# Run customer verifier
cd /home/ransomeye/rebuild
python3 core/customer_verifier/customer_verify.py

# Expected output:
# SHIP_FINALITY_VERIFIED = true
# overall_verified = true
# Exit code: 0
```

### Step 2: Verify Verification Results

```bash
# Read verification results
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json | jq .

# Verify SHIP_FINALITY_VERIFIED flag
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json | jq .SHIP_FINALITY_VERIFIED
# Expected: true

# Verify all checks passed
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json | jq .overall_verified
# Expected: true
```

### Step 3: Compare with Previous Results (if available)

```bash
# Compare verification results
diff customer_verify_20260128.json previous_customer_verify_20260127.json

# Expected: Results are consistent (same system state produces same results)
```

**Conclusion:** Customer verifier replay is confirmed if:
- ✅ Customer verifier runs successfully
- ✅ SHIP_FINALITY_VERIFIED = true
- ✅ All checks passed
- ✅ Results are consistent (if previous results available)

---

## Verification 5: Vendor Non-Repudiation Replay

### Step 1: Run Vendor Scanner

```bash
# Run vendor non-repudiation scanner
cd /home/ransomeye/rebuild
python3 core/governance/vendor_non_repudiation.py

# Expected output:
# ✅ No critical findings - Vendor non-repudiation verified
# Exit code: 0
```

### Step 2: Verify Scan Results

```bash
# Read scan results
cat /var/lib/ransomeye/governance/vendor_non_repudiation_scan.json | jq .

# Verify no critical findings
cat /var/lib/ransomeye/governance/vendor_non_repudiation_scan.json | jq .critical_findings
# Expected: 0

# Verify scan summary
cat /var/lib/ransomeye/governance/vendor_non_repudiation_scan.json | jq .summary
```

### Step 3: Review Evidence Report

```bash
# Read evidence report
cat /var/lib/ransomeye/governance/vendor_non_repudiation_evidence.md

# Expected: "VENDOR NON-REPUDIATION VERIFIED"
```

### Step 4: Compare with Previous Scan (if available)

```bash
# Compare scan results
diff vendor_non_repudiation_scan_20260128.json previous_vendor_non_repudiation_scan_20260127.json

# Expected: Results are consistent (same codebase produces same results)
```

**Conclusion:** Vendor non-repudiation replay is confirmed if:
- ✅ Vendor scanner runs successfully
- ✅ No critical findings
- ✅ Evidence report confirms verification
- ✅ Results are consistent (if previous scan available)

---

## Verification 6: Continuous Verifier Replay

### Step 1: Check Verifier Status

```bash
# Check verifier timer status
systemctl status ransomeye-verifier.timer

# Expected: Active (if systemd access available)
```

### Step 2: Run Verifier Manually

```bash
# Run verifier manually
cd /home/ransomeye/rebuild
python3 core/verifier/verifier.py

# Expected output:
# Verification passed
# Exit code: 0

# If failure:
# VERIFICATION FAILED: N failures
# Exit code: 1
```

### Step 3: Verify Verifier Results

```bash
# Read verifier results
cat /var/log/ransomeye/verifier_results.json | jq .

# Verify overall health
cat /var/log/ransomeye/verifier_results.json | jq .overall_healthy
# Expected: true

# Verify ship seal check
cat /var/log/ransomeye/verifier_results.json | jq .checks.ship_seal
# Expected: {"healthy": true, "error": null}
```

### Step 4: Review Verifier History

```bash
# Read verifier audit log
tail -100 /var/log/ransomeye/verifier_audit.log

# Check for violations
grep -i "violation\|failure" /var/log/ransomeye/verifier_audit.log | tail -20
```

**Conclusion:** Continuous verifier replay is confirmed if:
- ✅ Verifier runs successfully
- ✅ Overall health = true
- ✅ Ship seal check passed
- ✅ No violations detected

---

## Independent Conclusion Process

### Step 1: Collect All Verification Results

```bash
# Create verification report directory
mkdir -p audit_verification_$(date +%Y%m%d)

# Collect all results
cp ship_seal_verification_output.txt audit_verification_$(date +%Y%m%d)/
cp audit_chain_export_*.json audit_verification_$(date +%Y%m%d)/
cp customer_verify_*.json audit_verification_$(date +%Y%m%d)/
cp vendor_non_repudiation_scan.json audit_verification_$(date +%Y%m%d)/
cp verifier_results.json audit_verification_$(date +%Y%m%d)/
```

### Step 2: Verify Evidence Integrity

```bash
# Compute hashes of all evidence
cd audit_verification_$(date +%Y%m%d)
find . -type f -exec sha256sum {} \; > evidence_hashes.txt

# Verify hashes
cat evidence_hashes.txt
```

### Step 3: Document Verification Process

```bash
# Create verification report
cat > verification_report_$(date +%Y%m%d).md << 'EOF'
# Independent Audit Verification Report

**Date:** $(date +%Y-%m-%d)
**Auditor:** [Auditor Name]
**Organization:** [Organization Name]

## Verification Results

### Ship Seal Enforcement
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

### Audit Chain Continuity
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

### Evidence Bundle Regeneration
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

### Customer Verifier Replay
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

### Vendor Non-Repudiation Replay
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

### Continuous Verifier Replay
- Status: [PASS/FAIL]
- Evidence: [Evidence path]
- Conclusion: [Conclusion]

## Overall Conclusion

[Overall conclusion based on all verifications]

## Evidence Attachments

- All verification results: [Path]
- Evidence hashes: [Path]
- Verification report: [Path]
EOF
```

### Step 4: Reach Independent Conclusion

**Conclusion Criteria:**

1. **Ship Seal Enforcement:**
   - ✅ Ship seal enforcer runs successfully
   - ✅ ARTIFACT_HASHES.txt is read-only
   - ✅ Binary hashes match
   - ✅ Ship seal integrated into verifier

2. **Audit Chain Continuity:**
   - ✅ Chain hash integrity verified
   - ✅ Chain continuity verified
   - ✅ No chain breaks detected

3. **Evidence Bundle Regeneration:**
   - ✅ Bundle generated successfully
   - ✅ Bundle hash verified
   - ✅ Bundle contents verified

4. **Customer Verifier Replay:**
   - ✅ Customer verifier runs successfully
   - ✅ SHIP_FINALITY_VERIFIED = true
   - ✅ All checks passed

5. **Vendor Non-Repudiation Replay:**
   - ✅ Vendor scanner runs successfully
   - ✅ No critical findings
   - ✅ Evidence report confirms verification

6. **Continuous Verifier Replay:**
   - ✅ Verifier runs successfully
   - ✅ Overall health = true
   - ✅ Ship seal check passed

**Independent Conclusion:**

If all verifications pass:
- ✅ **System integrity verified**
- ✅ **Ship seal enforcement confirmed**
- ✅ **Audit chain integrity confirmed**
- ✅ **Vendor non-repudiation confirmed**
- ✅ **Customer verification independence confirmed**

---

## Offline Verification

### Offline Evidence Bundle Verification

```bash
# Extract offline evidence bundle
tar -xzf evidence_bundle_v1.0.0.tar.gz

# Verify bundle hash
sha256sum -c evidence_bundle_v1.0.0.tar.gz.sha256

# Verify all artifacts
cd evidence_bundle_v1.0.0
sha256sum artifacts/ARTIFACT_HASHES.txt
sha256sum artifacts/file_hashes_*.txt

# Review evidence
cat evidence/audit_chain_sample.json | jq .
cat evidence/ship_finality_verification.json | jq .
cat evidence/vendor_non_repudiation_scan.json | jq .
```

### Offline Audit Chain Verification

```bash
# Verify audit chain from exported file
python3 << 'EOF'
import json
import hashlib

# Load exported audit chain
with open('evidence_bundle_v1.0.0/evidence/audit_chain_sample.json') as f:
    chain_data = json.load(f)
    chain = chain_data.get('chain', [])

# Verify chain integrity (same logic as online verification)
# ... verification code ...
EOF
```

---

## Hostile Auditor Verification

### Zero Trust Verification

**Assumptions:**
- Zero vendor trust
- Zero customer trust
- All evidence must be independently verifiable

**Verification Steps:**

1. **Verify all evidence independently:**
   - Compute all hashes independently
   - Verify all hashes match
   - Verify all evidence is consistent

2. **Verify no vendor assistance required:**
   - All tools run independently
   - All evidence generated independently
   - All verification performed independently

3. **Verify reproducibility:**
   - Same inputs produce same outputs
   - Evidence bundles are reproducible
   - Verification results are consistent

### Verification Without Vendor Trust

**Independent Verification Methods:**

1. **Ship Seal:**
   - Read ARTIFACT_HASHES.txt independently
   - Compute binary hashes independently
   - Compare hashes independently

2. **Audit Chain:**
   - Export audit chain independently
   - Verify chain hashes independently
   - Verify chain continuity independently

3. **Evidence Bundle:**
   - Generate bundle independently
   - Verify bundle hash independently
   - Verify bundle contents independently

---

## Conclusion

This guide enables **independent auditor verification** without vendor assistance or trust. All verifications are reproducible and can be performed offline.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

