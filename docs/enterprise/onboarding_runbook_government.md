# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/onboarding_runbook_government.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Government/Sovereign Onboarding Runbook - Customer-facing runbook for government deployments (PROMPT-66-B)

# Government / Sovereign Onboarding Runbook (PROMPT-66-B)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This runbook provides **customer-facing procedures** for onboarding RansomEye v1.0.0-enterprise-ship in **government and sovereign environments**.

**Security Context:**
- Classified information handling
- Sovereign data protection
- National security requirements
- Air-gapped environments
- Highest security classification

---

## Pre-Deployment Checklist

### Security Clearance

- [ ] Personnel security clearances verified
- [ ] Facility security clearances verified
- [ ] System security clearances verified
- [ ] Access controls configured (mandatory)
- [ ] Security classification verified

### Environment Verification

- [ ] Air-gapped network confirmed (mandatory)
- [ ] Network isolation verified (no external connections)
- [ ] Physical security measures in place (SCIF/secure facility)
- [ ] Access controls configured (multi-factor authentication)
- [ ] Encryption at rest enabled (FIPS 140-2 compliant)
- [ ] Encryption in transit enabled (TLS 1.3+)
- [ ] Backup systems available (air-gapped)
- [ ] Evidence storage configured (classified storage)

### System Requirements

- [ ] Operating system: Linux (RHEL 8+, Ubuntu 20.04+, Debian 11+)
- [ ] Database: PostgreSQL 12+ (with FIPS 140-2 encryption)
- [ ] Python 3.8+ installed
- [ ] Systemd available
- [ ] Sufficient disk space (>1TB recommended for long-term retention)
- [ ] Sufficient memory (>64GB recommended)

### Security Requirements

- [ ] Ship seal hash list present
- [ ] No hardcoded credentials
- [ ] Environment variables configured
- [ ] Access controls configured (RBAC, mandatory)
- [ ] Audit logging enabled (mandatory)
- [ ] Encryption configured (FIPS 140-2)
- [ ] Key management system configured (HSM if required)
- [ ] Security classification labels applied

---

## Installation Verification Steps

### Step 1: Security Pre-Installation Review

**Required reviews:**
1. Security team review (mandatory)
2. Classification authority review
3. Facility security review
4. Risk assessment review
5. Legal/regulatory review

**Documentation required:**
- Installation plan (classified)
- Security controls documentation
- Evidence retention plan
- Incident response procedures
- Classification guide

### Step 2: Verify Installation Media (Secure Transfer)

```bash
# Verify media integrity (secure transfer)
sha256sum -c media_checksums.txt

# Verify ARTIFACT_HASHES.txt present
ls -la /media/ransomeye/docs/ARTIFACT_HASHES.txt

# Verify file is read-only
stat -c "%a %n" /media/ransomeye/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)

# Verify media chain of custody
cat /media/ransomeye/chain_of_custody.txt
```

### Step 3: Install RansomEye (Secure Environment)

```bash
# Extract installation media (in secure facility)
tar -xzf ransomeye-v1.0.0-enterprise-ship.tar.gz -C /home/ransomeye/

# Verify installation
ls -la /home/ransomeye/rebuild/

# Apply security classification labels
chattr +i /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
```

### Step 4: Verify Ship Seal (Mandatory)

```bash
# Run ship seal enforcer (mandatory)
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py

# Expected: Exit code 0, "✓ Ship seal verified - all binaries intact"
```

**If ship seal verification fails:**
- **DO NOT PROCEED** with installation
- Document failure details (classified)
- Notify security team immediately
- Contact vendor support (via secure channel only)
- Do not attempt to bypass or modify ship seal
- Preserve all evidence (classified)

### Step 5: Configure Environment (FIPS 140-2)

```bash
# Set environment variables (no hardcoded values)
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=ransomeye
export DB_USER=gagan
export DB_PASS=gagan
# ... other required variables

# Configure FIPS 140-2 encryption
export DB_ENCRYPTION_KEY_PATH=/etc/ransomeye/encryption.key
export ENCRYPTION_ENABLED=true
export FIPS_MODE=true
```

### Step 6: Initialize Database (FIPS 140-2 Encrypted)

```bash
# Initialize database with FIPS 140-2 encryption
# Verify database encryption enabled
# Ensure immutable_audit_log table exists
# Verify encryption at rest (FIPS 140-2)
# Verify key management (HSM if required)
```

### Step 7: Start Services (Secure)

```bash
# Start systemd services
systemctl start ransomeye-ingestion
systemctl start ransomeye-normalization
systemctl start ransomeye-ui

# Verify services are active
systemctl status ransomeye-ingestion
systemctl status ransomeye-normalization
systemctl status ransomeye-ui
```

### Step 8: Enable Continuous Verifier (Mandatory)

```bash
# Enable and start verifier timer (mandatory)
systemctl enable ransomeye-verifier.timer
systemctl start ransomeye-verifier.timer

# Verify timer is active
systemctl status ransomeye-verifier.timer
```

---

## Customer Ship Finality Verification

### Step 1: Run Customer Verifier (Mandatory)

```bash
# Run customer verifier (mandatory)
cd /home/ransomeye/rebuild
python3 core/customer_verifier/customer_verify.py

# Expected output:
# SHIP_FINALITY_VERIFIED = true
# overall_verified = true
```

### Step 2: Security Verification (Mandatory)

**Required for government/sovereign deployments:**
1. Security team verification (mandatory)
2. Classification authority verification
3. Facility security verification
4. Legal/regulatory verification

**Verification checklist:**
- [ ] Ship seal verified
- [ ] Customer verifier passed
- [ ] Vendor non-repudiation verified
- [ ] Audit log integrity verified
- [ ] Encryption verified (FIPS 140-2)
- [ ] Access controls verified
- [ ] Security classification verified
- [ ] Chain of custody verified

### Step 3: Generate Evidence Bundle (Classified)

```bash
# Generate evidence bundle for security review
sudo /home/ransomeye/rebuild/scripts/generate_evidence_bundle.sh

# Verify bundle integrity
sha256sum -c /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz.sha256

# Apply security classification
chattr +i /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz
```

---

## Operational Do's and Don'ts

### ✅ DO's

- **DO** run daily ship seal verification (mandatory)
- **DO** monitor verifier results (mandatory)
- **DO** preserve all evidence (permanent retention)
- **DO** follow incident response procedures (mandatory)
- **DO** maintain audit log custody (mandatory)
- **DO** perform regular backups (encrypted, non-mutating, air-gapped)
- **DO** document all operational activities (classified)
- **DO** notify security team of violations (immediate)
- **DO** maintain security classification
- **DO** follow national security requirements
- **DO** maintain chain of custody

### ❌ DON'Ts

- **DON'T** modify core binaries
- **DON'T** modify ARTIFACT_HASHES.txt
- **DON'T** bypass verifier checks
- **DON'T** delete audit log entries
- **DON'T** modify ship seal enforcer
- **DON'T** attempt to override protections
- **DON'T** hardcode credentials
- **DON'T** skip verification steps
- **DON'T** violate security classification
- **DON'T** bypass encryption requirements
- **DON'T** connect to external networks
- **DON'T** bypass air-gap isolation
- **DON'T** share classified information

---

## Evidence Retention Guidance

### Security Requirements

**Government/sovereign deployments must retain:**
- **Audit logs:** Permanent retention
- **Evidence artifacts:** Permanent retention
- **Compliance reports:** Permanent retention
- **Incident reports:** Permanent retention
- **Violation evidence:** Permanent retention
- **Classification records:** Permanent retention

### Initial Evidence Collection

**Collect at installation (classified):**
1. Ship seal verification output
2. Customer verifier output
3. Vendor non-repudiation scan results
4. Service startup logs
5. Initial verifier results
6. Database schema verification
7. Encryption verification (FIPS 140-2)
8. Security team approval
9. Classification authority approval
10. Chain of custody documentation

**Store in:** `/var/lib/ransomeye/evidence/onboarding/` (encrypted, classified)

### Ongoing Evidence Collection

**Collect daily (mandatory):**
- Verifier results
- Ship seal verification output
- Service status reports
- Security checks

**Collect weekly (mandatory):**
- Complete audit chain export (encrypted, classified)
- Customer verifier output
- Evidence bundle (if generated)

**Collect monthly (mandatory):**
- Complete evidence bundle (encrypted, classified)
- Security reports
- Audit log summary
- Classification review

### Evidence Storage

**Requirements:**
- Encrypted storage (FIPS 140-2)
- Permanent retention
- Chain of custody documentation
- Security access controls
- Audit trail of access
- Classification labels
- Air-gapped storage

---

## Security Classification

### Classification Levels

**Classification requirements:**
- Unclassified
- Confidential
- Secret
- Top Secret

**Handling requirements:**
- Apply classification labels
- Maintain classification records
- Follow classification procedures
- Protect classified information

### Classification Procedures

**For all evidence:**
1. Determine classification level
2. Apply classification labels
3. Document classification decision
4. Maintain classification records
5. Follow declassification procedures (if applicable)

---

## National Security Requirements

### Security Controls

**Required controls:**
- Multi-factor authentication
- Role-based access control
- Encryption (FIPS 140-2)
- Audit logging (mandatory)
- Incident response
- Chain of custody

### Security Verification

**Required verification:**
- Security team review
- Classification authority review
- Facility security review
- Risk assessment review
- Compliance verification

---

## Incident Response (National Security)

### Security Notification

**Required notifications:**
- Immediate: Security incidents
- Immediate: Classification violations
- Immediate: National security threats
- 24 hours: Data breaches
- Weekly: Security status

**Notification procedures:**
1. Document incident details (classified)
2. Preserve all evidence (classified)
3. Notify security team (immediate)
4. Notify classification authority (if required)
5. Notify national security authorities (if required)
6. Follow incident response matrix

---

## Post-Onboarding Verification

### Week 1 Verification

- [ ] All services running
- [ ] Verifier passing
- [ ] Ship seal intact
- [ ] Audit log growing
- [ ] No violations detected
- [ ] Security team approval
- [ ] Classification authority approval

### Month 1 Verification

- [ ] Complete evidence bundle generated
- [ ] Customer verifier passing
- [ ] Vendor non-repudiation verified
- [ ] Audit log integrity verified
- [ ] Security reports generated
- [ ] Classification review completed
- [ ] National security approval (if required)

### Ongoing Verification

- [ ] Daily ship seal verification (mandatory)
- [ ] Daily verifier monitoring (mandatory)
- [ ] Weekly evidence collection (mandatory)
- [ ] Monthly security review (mandatory)
- [ ] Quarterly classification review
- [ ] Annual comprehensive security audit

---

## Conclusion

This runbook ensures **controlled, repeatable, auditable onboarding** for government and sovereign environments. All procedures preserve immutable assurances while maintaining national security requirements.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

