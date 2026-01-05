# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/onboarding_runbook_airgap.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Air-Gapped Enterprise Onboarding Runbook - Customer-facing runbook for air-gapped deployments (PROMPT-66-B)

# Air-Gapped Enterprise Onboarding Runbook (PROMPT-66-B)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This runbook provides **customer-facing procedures** for onboarding RansomEye v1.0.0-enterprise-ship in **air-gapped enterprise environments**.

**Assumptions:**
- No internet connectivity
- No external network access
- Complete network isolation
- Physical media transfer only

---

## Pre-Deployment Checklist

### Environment Verification

- [ ] Air-gapped network confirmed (no internet connectivity)
- [ ] Network isolation verified (no external connections)
- [ ] Physical security measures in place
- [ ] Access controls configured
- [ ] Backup systems available
- [ ] Evidence storage configured

### System Requirements

- [ ] Operating system: Linux (RHEL 8+, Ubuntu 20.04+, Debian 11+)
- [ ] Database: PostgreSQL 12+ (local installation)
- [ ] Python 3.8+ installed
- [ ] Systemd available
- [ ] Sufficient disk space (>100GB recommended)
- [ ] Sufficient memory (>16GB recommended)

### Media Transfer

- [ ] RansomEye installation media verified
- [ ] ARTIFACT_HASHES.txt present
- [ ] Installation scripts present
- [ ] Documentation included
- [ ] Media integrity verified (checksums)

### Security Requirements

- [ ] Ship seal hash list present
- [ ] No hardcoded credentials
- [ ] Environment variables configured
- [ ] Access controls configured
- [ ] Audit logging enabled

---

## Installation Verification Steps

### Step 1: Verify Installation Media

```bash
# Verify media integrity
sha256sum -c media_checksums.txt

# Verify ARTIFACT_HASHES.txt present
ls -la /media/ransomeye/docs/ARTIFACT_HASHES.txt

# Verify file is read-only
stat -c "%a %n" /media/ransomeye/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)
```

### Step 2: Install RansomEye

```bash
# Extract installation media
tar -xzf ransomeye-v1.0.0-enterprise-ship.tar.gz -C /home/ransomeye/

# Verify installation
ls -la /home/ransomeye/rebuild/
```

### Step 3: Verify Ship Seal

```bash
# Run ship seal enforcer
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py

# Expected: Exit code 0, "✓ Ship seal verified - all binaries intact"
```

**If ship seal verification fails:**
- **DO NOT PROCEED** with installation
- Document failure details
- Contact vendor support (via secure channel)
- Do not attempt to bypass or modify ship seal

### Step 4: Configure Environment

```bash
# Set environment variables (no hardcoded values)
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=ransomeye
export DB_USER=gagan
export DB_PASS=gagan
# ... other required variables

# Create .env file (if using)
cat > /home/ransomeye/rebuild/.env <<EOF
DB_HOST=localhost
DB_PORT=5432
DB_NAME=ransomeye
DB_USER=gagan
DB_PASS=gagan
EOF
```

### Step 5: Initialize Database

```bash
# Initialize database (if required)
# Verify database schema
# Ensure immutable_audit_log table exists
```

### Step 6: Start Services

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

### Step 7: Enable Continuous Verifier

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

### Step 2: Verify Ship Finality Checks

```bash
# Review verification results
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json | jq .

# Verify SHIP_FINALITY_VERIFIED flag
cat /var/lib/ransomeye/customer_verification/customer_verify_*.json | jq .SHIP_FINALITY_VERIFIED
# Expected: true
```

### Step 3: Verify Ship Seal Components

```bash
# Verify ship seal enforcer exists
ls -la /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Verify ARTIFACT_HASHES.txt exists
ls -la /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Verify ship seal integration
grep -n "check_ship_seal\|ShipSealEnforcer" /home/ransomeye/rebuild/core/verifier/verifier.py
```

### Step 4: Verify Vendor Non-Repudiation

```bash
# Run vendor non-repudiation scanner
python3 /home/ransomeye/rebuild/core/governance/vendor_non_repudiation.py

# Expected: Exit code 0, "✅ No critical findings - Vendor non-repudiation verified"

# Review scan results
cat /var/lib/ransomeye/governance/vendor_non_repudiation_scan.json | jq .critical_findings
# Expected: 0
```

---

## Operational Do's and Don'ts

### ✅ DO's

- **DO** run daily ship seal verification
- **DO** monitor verifier results
- **DO** preserve all evidence
- **DO** follow incident response procedures
- **DO** maintain audit log custody
- **DO** perform regular backups (non-mutating)
- **DO** document all operational activities
- **DO** verify media integrity before installation

### ❌ DON'Ts

- **DON'T** modify core binaries
- **DON'T** modify ARTIFACT_HASHES.txt
- **DON'T** bypass verifier checks
- **DON'T** delete audit log entries
- **DON'T** modify ship seal enforcer
- **DON'T** attempt to override protections
- **DON'T** hardcode credentials
- **DON'T** skip verification steps
- **DON'T** connect to external networks
- **DON'T** bypass air-gap isolation

---

## Evidence Retention Guidance

### Initial Evidence Collection

**Collect at installation:**
1. Ship seal verification output
2. Customer verifier output
3. Vendor non-repudiation scan results
4. Service startup logs
5. Initial verifier results
6. Database schema verification

**Store in:** `/var/lib/ransomeye/evidence/onboarding/`

### Ongoing Evidence Collection

**Collect daily:**
- Verifier results
- Ship seal verification output
- Service status reports

**Collect weekly:**
- Complete audit chain export
- Customer verifier output
- Evidence bundle (if generated)

**Collect monthly:**
- Complete evidence bundle
- Compliance reports
- Audit log summary

### Evidence Retention

- **Daily evidence:** Retain for 90 days
- **Weekly evidence:** Retain for 1 year
- **Monthly evidence:** Retain for 7 years (compliance requirement)
- **Violation evidence:** Retain permanently
- **Onboarding evidence:** Retain permanently

### Evidence Storage

**Requirements:**
- Store evidence on air-gapped systems only
- Compute SHA-256 hashes for all evidence
- Maintain evidence chain of custody
- Document evidence collection process

---

## Air-Gap Specific Considerations

### Network Isolation

**Requirements:**
- No internet connectivity
- No external network access
- Complete network isolation
- Physical media transfer only

**Verification:**
```bash
# Verify no internet connectivity
ping -c 1 8.8.8.8
# Expected: No response (timeout)

# Verify no external DNS
nslookup google.com
# Expected: No response
```

### Media Transfer

**Requirements:**
- Physical media only (USB, DVD, etc.)
- Media integrity verification
- Secure transfer procedures
- Chain of custody documentation

**Verification:**
```bash
# Verify media checksums
sha256sum -c media_checksums.txt

# Verify media integrity
md5sum -c media_checksums.md5
```

### Update Procedures

**Requirements:**
- Updates via physical media only
- Update integrity verification
- Ship seal verification after update
- Evidence collection after update

**Note:** Updates may require new shipment version if ship seal changes.

---

## Troubleshooting

### Issue: Ship Seal Verification Fails

**Possible causes:**
- Binary hash mismatch
- Missing ARTIFACT_HASHES.txt
- File permission issues
- Media corruption

**Resolution:**
- Check violation details in output
- Verify ARTIFACT_HASHES.txt exists and is read-only
- Verify media integrity
- Contact vendor support (via secure channel)

### Issue: Service Startup Failure

**Possible causes:**
- Ship seal violation
- Database connection failure
- Configuration issues
- Missing dependencies

**Resolution:**
- Review service logs: `journalctl -u <service-name>`
- Verify ship seal integrity
- Verify database connectivity
- Check configuration

### Issue: Verifier Failure

**Possible causes:**
- Ship seal violation
- Service failures
- Database issues
- Configuration drift

**Resolution:**
- Review verifier results: `/var/log/ransomeye/verifier_results.json`
- Verify ship seal integrity
- Check service status
- Follow incident response procedures

---

## Post-Onboarding Verification

### Week 1 Verification

- [ ] All services running
- [ ] Verifier passing
- [ ] Ship seal intact
- [ ] Audit log growing
- [ ] No violations detected

### Month 1 Verification

- [ ] Complete evidence bundle generated
- [ ] Customer verifier passing
- [ ] Vendor non-repudiation verified
- [ ] Audit log integrity verified
- [ ] Compliance reports generated

### Ongoing Verification

- [ ] Daily ship seal verification
- [ ] Daily verifier monitoring
- [ ] Weekly evidence collection
- [ ] Monthly compliance review
- [ ] Annual comprehensive review

---

## Conclusion

This runbook ensures **controlled, repeatable, auditable onboarding** for air-gapped enterprise environments. All procedures preserve immutable assurances while enabling necessary operational activities.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

