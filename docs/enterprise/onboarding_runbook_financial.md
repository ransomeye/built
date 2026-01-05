# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/onboarding_runbook_financial.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regulated Financial Institution Onboarding Runbook - Customer-facing runbook for financial deployments (PROMPT-66-B)

# Regulated Financial Institution Onboarding Runbook (PROMPT-66-B)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This runbook provides **customer-facing procedures** for onboarding RansomEye v1.0.0-enterprise-ship in **regulated financial institution environments**.

**Regulatory Context:**
- SOC 2 compliance
- PCI DSS compliance
- FFIEC guidelines
- GDPR compliance
- Financial services regulations

---

## Pre-Deployment Checklist

### Regulatory Compliance

- [ ] Regulatory approval obtained (if required)
- [ ] Compliance team notified
- [ ] Audit team engaged
- [ ] Legal team review completed
- [ ] Risk assessment completed
- [ ] Data protection impact assessment (DPIA) completed

### Environment Verification

- [ ] Network segmentation configured
- [ ] Access controls configured
- [ ] Encryption at rest enabled
- [ ] Encryption in transit enabled
- [ ] Backup systems available
- [ ] Disaster recovery procedures documented
- [ ] Evidence storage configured (7-year retention)

### System Requirements

- [ ] Operating system: Linux (RHEL 8+, Ubuntu 20.04+, Debian 11+)
- [ ] Database: PostgreSQL 12+ (with encryption)
- [ ] Python 3.8+ installed
- [ ] Systemd available
- [ ] Sufficient disk space (>500GB recommended for 7-year retention)
- [ ] Sufficient memory (>32GB recommended)

### Security Requirements

- [ ] Ship seal hash list present
- [ ] No hardcoded credentials
- [ ] Environment variables configured
- [ ] Access controls configured (RBAC)
- [ ] Audit logging enabled
- [ ] Encryption configured
- [ ] Key management system configured

---

## Installation Verification Steps

### Step 1: Regulatory Pre-Installation Review

**Required reviews:**
1. Compliance team review
2. Security team review
3. Legal team review
4. Risk assessment review

**Documentation required:**
- Installation plan
- Security controls documentation
- Evidence retention plan
- Incident response procedures

### Step 2: Verify Installation Media

```bash
# Verify media integrity
sha256sum -c media_checksums.txt

# Verify ARTIFACT_HASHES.txt present
ls -la /media/ransomeye/docs/ARTIFACT_HASHES.txt

# Verify file is read-only
stat -c "%a %n" /media/ransomeye/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)
```

### Step 3: Install RansomEye

```bash
# Extract installation media
tar -xzf ransomeye-v1.0.0-enterprise-ship.tar.gz -C /home/ransomeye/

# Verify installation
ls -la /home/ransomeye/rebuild/
```

### Step 4: Verify Ship Seal

```bash
# Run ship seal enforcer
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py

# Expected: Exit code 0, "✓ Ship seal verified - all binaries intact"
```

**If ship seal verification fails:**
- **DO NOT PROCEED** with installation
- Document failure details
- Notify compliance team immediately
- Contact vendor support (via secure channel)
- Do not attempt to bypass or modify ship seal

### Step 5: Configure Environment (Encrypted)

```bash
# Set environment variables (no hardcoded values)
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=ransomeye
export DB_USER=gagan
export DB_PASS=gagan
# ... other required variables

# Configure encryption
export DB_ENCRYPTION_KEY_PATH=/etc/ransomeye/encryption.key
export ENCRYPTION_ENABLED=true
```

### Step 6: Initialize Database (Encrypted)

```bash
# Initialize database with encryption
# Verify database encryption enabled
# Ensure immutable_audit_log table exists
# Verify encryption at rest
```

### Step 7: Start Services

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

### Step 8: Enable Continuous Verifier

```bash
# Enable and start verifier timer
systemctl enable ransomeye-verifier.timer
systemctl start ransomeye-verifier.timer

# Verify timer is active
systemctl status ransomeye-verifier.timer
```

---

## Customer Ship Finality Verification

### Step 1: Run Customer Verifier

```bash
# Run customer verifier
cd /home/ransomeye/rebuild
python3 core/customer_verifier/customer_verify.py

# Expected output:
# SHIP_FINALITY_VERIFIED = true
# overall_verified = true
```

### Step 2: Regulatory Verification

**Required for financial institutions:**
1. Compliance team verification
2. Security team verification
3. Audit team verification
4. Legal team verification

**Verification checklist:**
- [ ] Ship seal verified
- [ ] Customer verifier passed
- [ ] Vendor non-repudiation verified
- [ ] Audit log integrity verified
- [ ] Encryption verified
- [ ] Access controls verified

### Step 3: Generate Evidence Bundle

```bash
# Generate evidence bundle for regulators
sudo /home/ransomeye/rebuild/scripts/generate_evidence_bundle.sh

# Verify bundle integrity
sha256sum -c /home/ransomeye/rebuild/artifacts/evidence_bundle_v1.0.0.tar.gz.sha256
```

---

## Operational Do's and Don'ts

### ✅ DO's

- **DO** run daily ship seal verification
- **DO** monitor verifier results
- **DO** preserve all evidence (7-year retention)
- **DO** follow incident response procedures
- **DO** maintain audit log custody
- **DO** perform regular backups (encrypted, non-mutating)
- **DO** document all operational activities
- **DO** notify compliance team of violations
- **DO** maintain regulatory compliance
- **DO** follow data protection regulations

### ❌ DON'Ts

- **DON'T** modify core binaries
- **DON'T** modify ARTIFACT_HASHES.txt
- **DON'T** bypass verifier checks
- **DON'T** delete audit log entries
- **DON'T** modify ship seal enforcer
- **DON'T** attempt to override protections
- **DON'T** hardcode credentials
- **DON'T** skip verification steps
- **DON'T** violate data protection regulations
- **DON'T** bypass encryption requirements

---

## Evidence Retention Guidance

### Regulatory Requirements

**Financial institutions must retain:**
- **Audit logs:** 7 years minimum
- **Evidence artifacts:** 7 years minimum
- **Compliance reports:** 7 years minimum
- **Incident reports:** 7 years minimum
- **Violation evidence:** Permanently

### Initial Evidence Collection

**Collect at installation:**
1. Ship seal verification output
2. Customer verifier output
3. Vendor non-repudiation scan results
4. Service startup logs
5. Initial verifier results
6. Database schema verification
7. Encryption verification
8. Compliance team approval

**Store in:** `/var/lib/ransomeye/evidence/onboarding/` (encrypted)

### Ongoing Evidence Collection

**Collect daily:**
- Verifier results
- Ship seal verification output
- Service status reports
- Compliance checks

**Collect weekly:**
- Complete audit chain export (encrypted)
- Customer verifier output
- Evidence bundle (if generated)

**Collect monthly:**
- Complete evidence bundle (encrypted)
- Compliance reports
- Audit log summary
- Regulatory reports

### Evidence Storage

**Requirements:**
- Encrypted storage
- 7-year retention minimum
- Chain of custody documentation
- Regulatory access controls
- Audit trail of access

---

## Regulatory Compliance

### SOC 2 Compliance

**Requirements:**
- Change detection and integrity monitoring
- Audit trail maintenance
- Evidence retention
- Incident response procedures

**Verification:**
- Regular compliance audits
- Evidence bundle generation
- Audit log review
- Compliance reports

### PCI DSS Compliance

**Requirements:**
- Data encryption
- Access controls
- Audit logging
- Incident response

**Verification:**
- Encryption verification
- Access control review
- Audit log review
- Compliance reports

### FFIEC Guidelines

**Requirements:**
- Security controls
- Audit trail
- Evidence retention
- Incident response

**Verification:**
- Security control review
- Audit trail verification
- Evidence retention verification
- Compliance reports

### GDPR Compliance

**Requirements:**
- Data protection
- Audit trail
- Evidence retention
- Incident notification

**Verification:**
- Data protection review
- Audit trail verification
- Evidence retention verification
- Compliance reports

---

## Regulatory Reporting

### Required Reports

**Monthly reports:**
- Compliance status
- Audit log summary
- Violation summary
- Evidence retention status

**Quarterly reports:**
- Comprehensive compliance review
- Evidence bundle
- Regulatory updates
- Risk assessment

**Annual reports:**
- Complete compliance audit
- Evidence bundle
- Regulatory compliance summary
- Risk assessment update

### Report Generation

```bash
# Generate compliance report
# Include:
# - Audit log summary
# - Violation summary
# - Evidence retention status
# - Compliance checklist
```

---

## Incident Response (Regulatory)

### Regulatory Notification

**Required notifications:**
- Immediate: Security incidents
- 24 hours: Data breaches
- 72 hours: Regulatory violations
- Weekly: Compliance status

**Notification procedures:**
1. Document incident details
2. Preserve all evidence
3. Notify compliance team
4. Notify legal team
5. Notify regulators (if required)
6. Follow incident response matrix

---

## Post-Onboarding Verification

### Week 1 Verification

- [ ] All services running
- [ ] Verifier passing
- [ ] Ship seal intact
- [ ] Audit log growing
- [ ] No violations detected
- [ ] Compliance team approval

### Month 1 Verification

- [ ] Complete evidence bundle generated
- [ ] Customer verifier passing
- [ ] Vendor non-repudiation verified
- [ ] Audit log integrity verified
- [ ] Compliance reports generated
- [ ] Regulatory approval (if required)

### Ongoing Verification

- [ ] Daily ship seal verification
- [ ] Daily verifier monitoring
- [ ] Weekly evidence collection
- [ ] Monthly compliance review
- [ ] Quarterly regulatory reporting
- [ ] Annual comprehensive audit

---

## Conclusion

This runbook ensures **controlled, repeatable, auditable onboarding** for regulated financial institutions. All procedures preserve immutable assurances while maintaining regulatory compliance.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

