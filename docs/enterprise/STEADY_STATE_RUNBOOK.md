# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/STEADY_STATE_RUNBOOK.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Steady-State Operations Runbook - Automated cadence for Enterprise-Complete maintenance

# RansomEye Steady-State Operations Runbook

## Overview

This runbook defines the automated cadence for maintaining **Enterprise-Complete** status with **zero drift**, **provable compliance**, and **scheduled re-attestation**.

## Operational Cadence

### Every 5 Minutes: Continuous Verification

**Service**: `ransomeye-verifier`  
**Frequency**: Every 5 minutes  
**Purpose**: Verify all system invariants

**Checks**:
- Systemd services active (no restart loops)
- Database tables increasing
- Audit actions present
- Model registry active versions
- Threat intel IOC count > 0
- DPI Probe L7 protocol detection
- Linux Agent heartbeat
- UI reachable
- Artifact hashes match
- Drift detection (no unauthorized changes)

**Enforcement**: Fail-closed on any violation

**Status**: ✅ ACTIVE (systemd timer)

---

### Daily: Integrity Snapshot Diff

**Script**: `/home/ransomeye/rebuild/core/baseline/daily_integrity_diff.py`  
**Frequency**: Daily at 00:00 UTC  
**Purpose**: Compare current state against golden baseline

**Actions**:
1. Capture current system state snapshot
2. Compare against golden baseline (`/var/lib/ransomeye/baselines/golden_baseline.json`)
3. Generate diff report
4. Alert on any drift detected

**Output**: `/var/lib/ransomeye/integrity_diffs/daily_YYYYMMDD.json`

**Enforcement**: Drift detection triggers integrity violation

**Status**: ⚠️ TO BE IMPLEMENTED

---

### Weekly: Full Pipeline Sanity Replay

**Script**: `/home/ransomeye/rebuild/core/validation/weekly_pipeline_replay.py`  
**Frequency**: Weekly (Sunday 02:00 UTC)  
**Purpose**: Synthetic end-to-end pipeline validation

**Actions**:
1. Generate synthetic events
2. Replay through full pipeline:
   - Ingestion → Normalization → Correlation → Response
3. Verify all stages complete successfully
4. Validate audit trail integrity
5. Check export capabilities (CSV/HTML/PDF)

**Output**: `/var/lib/ransomeye/pipeline_replays/weekly_YYYYMMDD.json`

**Enforcement**: Pipeline failures trigger investigation

**Status**: ⚠️ TO BE IMPLEMENTED

---

### Monthly: Compliance Automation

**Service**: `ransomeye-compliance-automation`  
**Frequency**: Monthly (1st of each month at 00:00 UTC)  
**Purpose**: Generate compliance evidence

**Actions**:
1. Audit retention proof
2. Data lineage proof
3. AI explainability samples (SHAP)

**Output**: `/docs/enterprise/compliance/monthly/YYYY-MM/compliance_report.json`

**Enforcement**: Immutable, timestamped evidence

**Status**: ✅ ACTIVE (systemd timer)

---

### Quarterly: Executive Re-Attestation Refresh

**Script**: `/home/ransomeye/rebuild/core/attestation/quarterly_re_attestation.py`  
**Frequency**: Quarterly (Q1: Jan 1, Q2: Apr 1, Q3: Jul 1, Q4: Oct 1)  
**Purpose**: Non-destructive executive re-attestation refresh

**Actions**:
1. Generate drift summary (must be zero)
2. Capture verifier uptime
3. Generate audit growth proof
4. Capture SHAP sample
5. Include incident drill delta (if any)
6. Generate `EXECUTIVE_ATTESTATION_Q{N}_YYYY.md`

**Output**: `/docs/enterprise/attestations/EXECUTIVE_ATTESTATION_Q{N}_YYYY.md`

**Enforcement**: Immutable, read-only after generation

**Status**: ⚠️ TO BE IMPLEMENTED

---

## Systemd Timers

### Active Timers

1. **ransomeye-verifier.timer**
   - Frequency: Every 5 minutes
   - Status: ✅ ACTIVE

2. **ransomeye-compliance-automation.timer**
   - Frequency: Monthly (1st of month)
   - Status: ✅ ACTIVE

### To Be Created

1. **ransomeye-daily-integrity-diff.timer**
   - Frequency: Daily at 00:00 UTC
   - Status: ⚠️ TO BE CREATED

2. **ransomeye-weekly-pipeline-replay.timer**
   - Frequency: Weekly (Sunday 02:00 UTC)
   - Status: ⚠️ TO BE CREATED

3. **ransomeye-quarterly-re-attestation.timer**
   - Frequency: Quarterly (Q1/Q2/Q3/Q4 start)
   - Status: ⚠️ TO BE CREATED

---

## Monitoring and Alerts

### Verifier Status

Check verifier results:
```bash
cat /var/log/ransomeye/verifier_results.json
```

### Integrity Diff Status

Check daily integrity diff:
```bash
ls -la /var/lib/ransomeye/integrity_diffs/
```

### Pipeline Replay Status

Check weekly pipeline replay:
```bash
ls -la /var/lib/ransomeye/pipeline_replays/
```

### Compliance Status

Check monthly compliance reports:
```bash
ls -la /home/ransomeye/rebuild/docs/enterprise/compliance/monthly/
```

### Re-Attestation Status

Check quarterly attestations:
```bash
ls -la /home/ransomeye/rebuild/docs/enterprise/attestations/
```

---

## Failure Handling

### Verifier Failure

- **Action**: System enters failure state
- **Audit**: `SYSTEM_INTEGRITY_VIOLATION` written
- **Recovery**: Manual intervention required

### Integrity Diff Drift

- **Action**: Integrity violation triggered
- **Audit**: Drift details logged
- **Recovery**: Investigate and resolve drift

### Pipeline Replay Failure

- **Action**: Investigation triggered
- **Audit**: Pipeline failure logged
- **Recovery**: Fix pipeline issue

### Compliance Generation Failure

- **Action**: Retry next month
- **Audit**: Failure logged
- **Recovery**: Manual generation if needed

### Re-Attestation Failure

- **Action**: Retry next quarter
- **Audit**: Failure logged
- **Recovery**: Manual generation if needed

---

## Maintenance Windows

### Scheduled Maintenance

- **Weekly**: Review verifier results
- **Monthly**: Review compliance reports
- **Quarterly**: Review re-attestations
- **Annually**: Full system audit

### Emergency Maintenance

- **Trigger**: Verifier failure
- **Procedure**: Follow incident response playbook
- **Documentation**: Update incident log

---

## Acceptance Criteria

- [x] Verifier active (every 5 minutes)
- [ ] Daily integrity diff implemented
- [ ] Weekly pipeline replay implemented
- [x] Monthly compliance automation active
- [ ] Quarterly re-attestation implemented
- [ ] All timers configured
- [ ] Monitoring in place

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

