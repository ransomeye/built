# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/production_operations_playbook.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Production Operations Playbook - Strict operations procedures from Day-0 to Day-365 (PROMPT-66-A)

# Production Operations Playbook (PROMPT-66-A)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This playbook provides **strict operations procedures** for RansomEye v1.0.0-enterprise-ship from Day-0 deployment through Day-365 steady-state operations.

**Hard Rule:** Operations must **never** be able to mutate sealed core state.

---

## Day-0: Initial Deployment

### Pre-Deployment Checklist

- [ ] Air-gapped environment verified
- [ ] Network isolation confirmed
- [ ] Database credentials configured (via environment variables)
- [ ] Ship seal hash list present (`/home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt`)
- [ ] All systemd service files present
- [ ] Required directories created:
  - `/var/log/ransomeye/`
  - `/var/lib/ransomeye/`
  - `/etc/ransomeye/`

### Single Node Deployment (Air-Gapped First)

#### Step 1: Verify Ship Seal

```bash
# Verify ship seal enforcer exists
ls -la /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py

# Verify ARTIFACT_HASHES.txt exists and is read-only
stat -c "%a %n" /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
# Expected: 444 (read-only)

# Run ship seal enforcer
cd /home/ransomeye/rebuild
python3 core/assurance/ship_seal_enforcer.py
# Expected: Exit code 0, "✓ Ship seal verified - all binaries intact"
```

**If ship seal verification fails:**
- **DO NOT PROCEED** with deployment
- Document failure details
- Contact vendor support (if authorized)
- Do not attempt to bypass or modify ship seal

#### Step 2: Configure Environment Variables

```bash
# Set required environment variables (no hardcoded values)
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=ransomeye
export DB_USER=gagan
export DB_PASS=gagan
# ... other required variables
```

**Hard Rule:** Never hardcode credentials in configuration files.

#### Step 3: Initialize Database

```bash
# Run database initialization (if required)
# Verify database schema matches requirements
# Ensure immutable_audit_log table exists
```

#### Step 4: Start Services

```bash
# Start systemd services in order
systemctl start ransomeye-ingestion
systemctl start ransomeye-normalization
systemctl start ransomeye-ui

# Verify services are active
systemctl status ransomeye-ingestion
systemctl status ransomeye-normalization
systemctl status ransomeye-ui
```

**If service fails to start:**
- Check service logs: `journalctl -u <service-name>`
- Verify ship seal enforcer passed
- Do not attempt to bypass service startup checks

#### Step 5: Enable Continuous Verifier

```bash
# Enable and start verifier timer
systemctl enable ransomeye-verifier.timer
systemctl start ransomeye-verifier.timer

# Verify timer is active
systemctl status ransomeye-verifier.timer
```

#### Step 6: Verify Initial State

```bash
# Run customer verifier
python3 /home/ransomeye/rebuild/core/customer_verifier/customer_verify.py

# Expected: SHIP_FINALITY_VERIFIED = true
# Expected: overall_verified = true
```

### Day-0 Evidence Collection

**Mandatory evidence to collect:**
1. Ship seal verification output
2. Customer verifier output
3. Service startup logs
4. Initial verifier results
5. Database schema verification

**Store evidence in:** `/var/lib/ransomeye/evidence/day0/`

---

## Day-1: Steady State Operations

### Daily Operations Checklist

#### Morning Checks (08:00)

- [ ] Verify all services are active
- [ ] Check verifier results from overnight
- [ ] Review audit log for violations
- [ ] Verify ship seal integrity (run enforcer)
- [ ] Check disk space (must be <80% for retention)

#### Service Status Verification

```bash
# Check all required services
systemctl is-active ransomeye-ingestion
systemctl is-active ransomeye-normalization
systemctl is-active ransomeye-ui
systemctl is-active ransomeye-verifier.timer

# Check verifier results
cat /var/log/ransomeye/verifier_results.json | jq .overall_healthy
# Expected: true
```

#### Ship Seal Verification

```bash
# Daily ship seal check
python3 /home/ransomeye/rebuild/core/assurance/ship_seal_enforcer.py
# Expected: Exit code 0
```

**If ship seal verification fails:**
- **IMMEDIATE INCIDENT** - Follow incident response procedures
- Do not attempt to fix or bypass
- Preserve all evidence

#### Audit Log Review

```bash
# Check for violations in last 24 hours
psql -h localhost -U gagan -d ransomeye -c "
    SELECT COUNT(*) 
    FROM ransomeye.immutable_audit_log 
    WHERE action = 'SYSTEM_INTEGRITY_VIOLATION' 
    AND created_at > NOW() - INTERVAL '24 hours';
"
```

**If violations found:**
- Document violation details
- Follow incident response procedures
- Do not attempt to delete or modify audit entries

### Operational Do's and Don'ts

#### ✅ DO's

- **DO** run daily ship seal verification
- **DO** monitor verifier results
- **DO** preserve all evidence
- **DO** follow incident response procedures
- **DO** maintain audit log custody
- **DO** perform regular backups (non-mutating)

#### ❌ DON'Ts

- **DON'T** modify core binaries
- **DON'T** modify ARTIFACT_HASHES.txt
- **DON'T** bypass verifier checks
- **DON'T** delete audit log entries
- **DON'T** modify ship seal enforcer
- **DON'T** attempt to override protections
- **DON'T** hardcode credentials
- **DON'T** skip verification steps

---

## Incident Handling

### Verifier-Triggered Failure Handling

#### Scenario: Verifier Failure

**Symptoms:**
- Verifier exits with non-zero code
- `overall_healthy: false` in verifier results
- SYSTEM_INTEGRITY_VIOLATION audit entry

**Mandatory Actions:**

1. **DO NOT** attempt to fix or bypass
2. **DO** preserve all evidence:
   - Verifier results: `/var/log/ransomeye/verifier_results.json`
   - Verifier audit log: `/var/log/ransomeye/verifier_audit.log`
   - Audit chain entries: Export from database
3. **DO** document failure details
4. **DO** follow incident response matrix
5. **DO** notify compliance team (if required)

**Forbidden Actions:**

- ❌ Do not modify verifier code
- ❌ Do not disable verifier timer
- ❌ Do not delete violation entries
- ❌ Do not attempt to "fix" violations
- ❌ Do not bypass ship seal checks

#### Scenario: Ship Seal Violation

**Symptoms:**
- Ship seal enforcer fails
- Binary hash mismatch detected
- SYSTEM_INTEGRITY_VIOLATION audit entry

**Mandatory Actions:**

1. **DO NOT** attempt to fix or bypass
2. **DO** preserve all evidence:
   - Ship seal enforcer output
   - Violation details from audit log
   - Binary hash verification results
3. **DO** document violation details
4. **DO** follow incident response matrix
5. **DO** notify compliance team immediately

**Forbidden Actions:**

- ❌ Do not modify ARTIFACT_HASHES.txt
- ❌ Do not replace binaries
- ❌ Do not bypass ship seal enforcer
- ❌ Do not delete violation entries

#### Scenario: Service Startup Failure

**Symptoms:**
- Service fails to start
- Service logs show ship seal violation
- Systemd shows service in failed state

**Mandatory Actions:**

1. **DO NOT** attempt to bypass startup checks
2. **DO** review service logs: `journalctl -u <service-name>`
3. **DO** verify ship seal integrity
4. **DO** preserve all evidence
5. **DO** follow incident response matrix

**Forbidden Actions:**

- ❌ Do not modify service files to skip checks
- ❌ Do not disable ship seal enforcement
- ❌ Do not start services manually without checks

---

## Evidence Preservation Procedures

### Daily Evidence Collection

**Collect daily:**
1. Verifier results: `/var/log/ransomeye/verifier_results.json`
2. Verifier audit log: `/var/log/ransomeye/verifier_audit.log`
3. Ship seal verification output
4. Service status reports

**Store in:** `/var/lib/ransomeye/evidence/daily/YYYY-MM-DD/`

### Weekly Evidence Collection

**Collect weekly:**
1. Complete audit chain export
2. Customer verifier output
3. Vendor non-repudiation scan results
4. Evidence bundle (if generated)

**Store in:** `/var/lib/ransomeye/evidence/weekly/YYYY-WW/`

### Monthly Evidence Collection

**Collect monthly:**
1. Complete evidence bundle
2. Compliance reports
3. Audit log summary
4. Verification history

**Store in:** `/var/lib/ransomeye/evidence/monthly/YYYY-MM/`

### Evidence Retention

- **Daily evidence:** Retain for 90 days
- **Weekly evidence:** Retain for 1 year
- **Monthly evidence:** Retain for 7 years (compliance requirement)
- **Violation evidence:** Retain permanently

### Evidence Chain of Custody

**For all evidence:**
1. Document collection timestamp
2. Document collector identity
3. Compute SHA-256 hash of evidence
4. Store hash with evidence
5. Maintain evidence log

**Evidence log format:**
```json
{
  "evidence_id": "uuid",
  "collection_timestamp": "2026-01-28T12:00:00Z",
  "collector": "operator-name",
  "evidence_path": "/path/to/evidence",
  "evidence_hash": "sha256:...",
  "description": "Evidence description"
}
```

---

## Audit Log Custody & Chain-of-Evidence

### Audit Log Access

**Authorized access:**
- Read-only access for operators
- Read-only access for auditors
- Read-only access for compliance teams
- No write access except by system

**Unauthorized access:**
- ❌ No manual modification of audit entries
- ❌ No deletion of audit entries
- ❌ No bypassing of audit chain

### Audit Chain Verification

**Daily verification:**
```bash
# Verify audit chain integrity
psql -h localhost -U gagan -d ransomeye -c "
    SELECT 
        audit_id,
        action,
        created_at,
        chain_hash_sha256,
        prev_payload_sha256
    FROM ransomeye.immutable_audit_log
    ORDER BY created_at DESC
    LIMIT 10;
"
```

**If chain break detected:**
- **IMMEDIATE INCIDENT**
- Document chain break details
- Preserve all evidence
- Follow incident response procedures

### Audit Log Export

**For regulatory compliance:**
```bash
# Export audit chain sample
psql -h localhost -U gagan -d ransomeye -c "
    SELECT json_agg(row_to_json(t))
    FROM (
        SELECT *
        FROM ransomeye.immutable_audit_log
        ORDER BY created_at DESC
        LIMIT 1000
    ) t;
" > audit_chain_export_$(date +%Y%m%d).json
```

**Export requirements:**
- Include chain hashes
- Include timestamps
- Include violation details
- Compute export hash
- Document export process

---

## Backup & Restore (Non-Mutating Only)

### Backup Procedures

#### Database Backup

```bash
# Non-mutating database backup
pg_dump -h localhost -U gagan -d ransomeye \
    --format=custom \
    --file=/backup/ransomeye_$(date +%Y%m%d).dump

# Verify backup integrity
pg_restore --list /backup/ransomeye_$(date +%Y%m%d).dump | head -20
```

**Backup requirements:**
- Non-mutating (read-only)
- Include audit log
- Include all tables
- Compute backup hash
- Store backup securely

#### Configuration Backup

```bash
# Backup configuration files (non-mutating)
tar -czf /backup/config_$(date +%Y%m%d).tar.gz \
    /etc/ransomeye/ \
    /home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt

# Verify backup integrity
sha256sum /backup/config_$(date +%Y%m%d).tar.gz
```

#### Evidence Backup

```bash
# Backup evidence directory
tar -czf /backup/evidence_$(date +%Y%m%d).tar.gz \
    /var/lib/ransomeye/evidence/

# Verify backup integrity
sha256sum /backup/evidence_$(date +%Y%m%d).tar.gz
```

### Restore Procedures

#### Database Restore

**Restore requirements:**
- Restore to separate environment first
- Verify restore integrity
- Verify audit chain integrity
- Do not restore over production without approval

```bash
# Restore database (test environment)
pg_restore -h localhost -U gagan -d ransomeye_test \
    --clean \
    /backup/ransomeye_20260128.dump

# Verify restore
psql -h localhost -U gagan -d ransomeye_test -c "
    SELECT COUNT(*) FROM ransomeye.immutable_audit_log;
"
```

**Hard Rule:** Never restore over production without:
1. Approval from compliance team
2. Evidence preservation
3. Audit trail documentation

#### Configuration Restore

**Restore requirements:**
- Verify ARTIFACT_HASHES.txt matches original
- Verify ship seal integrity
- Do not restore modified configurations

```bash
# Restore configuration (verify first)
tar -xzf /backup/config_20260128.tar.gz -C /tmp/restore

# Verify ARTIFACT_HASHES.txt
sha256sum /tmp/restore/home/ransomeye/rebuild/docs/ARTIFACT_HASHES.txt
# Compare with original hash

# Only restore if hash matches
```

---

## Disaster Recovery Boundaries

### What is Allowed

#### ✅ Allowed Actions

- **Database restore** (to separate environment first)
- **Configuration restore** (with hash verification)
- **Service restart** (after verification)
- **Evidence collection** (read-only)
- **Audit log export** (read-only)

#### ✅ Allowed Modifications

- **Environment variables** (via .env files)
- **Database data** (operational data only, not audit log)
- **Log files** (rotation, archival)
- **Backup files** (creation, archival)

### What is Forbidden

#### ❌ Forbidden Actions

- **Modify core binaries** (ship seal violation)
- **Modify ARTIFACT_HASHES.txt** (ship seal violation)
- **Modify ship seal enforcer** (ship seal violation)
- **Modify verifier code** (ship seal violation)
- **Delete audit log entries** (chain break)
- **Bypass verification checks** (integrity violation)
- **Disable assurance mechanisms** (integrity violation)

#### ❌ Forbidden Modifications

- **Core code files** (ship seal protected)
- **Model artifacts** (ship seal protected)
- **Systemd service files** (if ship seal protected)
- **Audit log entries** (immutable)
- **Ship seal hash list** (read-only)

### Disaster Recovery Procedures

#### Scenario: Complete System Failure

**Allowed recovery:**
1. Restore from backup (non-mutating)
2. Verify ship seal integrity
3. Verify audit chain integrity
4. Restart services (with verification)
5. Run customer verifier

**Forbidden recovery:**
- ❌ Do not skip verification steps
- ❌ Do not restore modified binaries
- ❌ Do not bypass ship seal checks
- ❌ Do not modify audit log

#### Scenario: Database Corruption

**Allowed recovery:**
1. Restore database from backup
2. Verify audit chain integrity
3. Verify no chain breaks
4. Document recovery process

**Forbidden recovery:**
- ❌ Do not modify audit log entries
- ❌ Do not delete violation entries
- ❌ Do not bypass chain verification

---

## Day-365: Annual Review

### Annual Operations Review

**Review items:**
1. Evidence retention compliance
2. Audit log custody procedures
3. Incident response effectiveness
4. Backup and restore procedures
5. Ship seal integrity history
6. Verifier failure history
7. Compliance reports

### Annual Evidence Collection

**Collect annually:**
1. Complete evidence bundle
2. Annual compliance report
3. Audit log summary
4. Verification history
5. Incident reports

**Store in:** `/var/lib/ransomeye/evidence/annual/YYYY/`

### Annual Verification

**Annual verification steps:**
1. Run customer verifier
2. Run vendor non-repudiation scanner
3. Verify ship seal integrity
4. Verify audit chain integrity
5. Review all evidence

---

## Operational Safety Guarantees

### Operations Cannot Weaken Security

**Guaranteed by:**
- Ship seal enforcement (fail-closed)
- Verifier enforcement (fail-closed)
- Immutable audit log
- Vendor non-repudiation

### Operations Cannot Mutate Core State

**Guaranteed by:**
- Read-only ARTIFACT_HASHES.txt
- Ship seal enforcer verification
- Verifier checks
- Immutable audit log

### Operations Cannot Bypass Protections

**Guaranteed by:**
- Fail-closed enforcement
- No override mechanisms
- No bypass paths
- Vendor non-repudiation

---

## Conclusion

This playbook ensures **controlled, repeatable, auditable production operations** without weakening security or mutating core state. All operations are designed to preserve immutable assurances while enabling necessary operational activities.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

