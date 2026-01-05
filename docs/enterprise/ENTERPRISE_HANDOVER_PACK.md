# RansomEye Enterprise Handover Pack

**Version**: 1.0.0-enterprise-ship  
**Generated**: 2026-01-05T10:40:44.752451+00:00  
**Purpose**: Complete operational handover documentation for RansomEye enterprise deployment

---

## Table of Contents

1. [Operator Runbook](#operator-runbook)
2. [Security Architecture Summary](#security-architecture-summary)
3. [Compliance Mapping](#compliance-mapping)
4. [Upgrade Policy](#upgrade-policy)
5. [Support Escalation Path](#support-escalation-path)

---

## Operator Runbook

# RansomEye Operator Runbook

## Overview

This runbook provides operational procedures for RansomEye v1.0.0-enterprise-ship.

## Service Management

### Start Services

```bash
sudo systemctl start ransomeye-core
sudo systemctl start ransomeye-ingestion
sudo systemctl start ransomeye-normalization
sudo systemctl start ransomeye-ui
```

### Stop Services

```bash
sudo systemctl stop ransomeye-ui
sudo systemctl stop ransomeye-normalization
sudo systemctl stop ransomeye-ingestion
sudo systemctl stop ransomeye-core
```

### Check Service Status

```bash
sudo systemctl status ransomeye-core
sudo systemctl status ransomeye-ingestion
sudo systemctl status ransomeye-normalization
sudo systemctl status ransomeye-ui
```

### View Logs

```bash
sudo journalctl -u ransomeye-core -f
sudo journalctl -u ransomeye-ingestion -f
sudo journalctl -u ransomeye-normalization -f
sudo journalctl -u ransomeye-ui -f
```

## Database Management

### Connect to Database

```bash
psql -h localhost -U gagan -d ransomeye
```

### Backup Database

```bash
pg_dump -h localhost -U gagan ransomeye > /var/backups/ransomeye_$(date +%Y%m%d_%H%M%S).sql
```

### Restore Database

```bash
psql -h localhost -U gagan -d ransomeye < /var/backups/ransomeye_backup.sql
```

## Monitoring

### Verifier Status

```bash
cat /var/log/ransomeye/verifier_results.json
```

### System Health

```bash
python3 /home/ransomeye/rebuild/core/verifier/verifier.py
```

## Troubleshooting

### Service Won't Start

1. Check logs: `sudo journalctl -u <service-name> -n 100`
2. Check database connection: `psql -h localhost -U gagan -d ransomeye -c "SELECT 1;"`
3. Check file permissions: `ls -la /opt/ransomeye/`
4. Check systemd status: `sudo systemctl status <service-name>`

### Database Connection Issues

1. Verify PostgreSQL is running: `sudo systemctl status postgresql`
2. Check credentials in `/etc/ransomeye/db.env`
3. Test connection: `psql -h localhost -U gagan -d ransomeye`

### High Disk Usage

1. Check retention policy: `cat /etc/ransomeye/retention.yaml`
2. Run retention enforcer: `sudo systemctl start ransomeye-retention-enforcer.service`
3. Check disk usage: `df -h /var/lib/ransomeye`

## Emergency Procedures

### Complete System Restart

```bash
sudo systemctl stop ransomeye-*
sudo systemctl start ransomeye-core
sudo systemctl start ransomeye-ingestion
sudo systemctl start ransomeye-normalization
sudo systemctl start ransomeye-ui
```

### Data Recovery

1. Stop all services
2. Restore database from backup
3. Verify data integrity
4. Restart services

## Maintenance Windows

### Scheduled Maintenance

- Weekly: Database backup
- Monthly: Compliance report generation
- Quarterly: System health audit

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


---

## Security Architecture Summary

# RansomEye Security Architecture Summary

## Overview

RansomEye implements defense-in-depth security architecture with zero-trust principles.

## Security Layers

### 1. Network Security

- **mTLS Transport**: All agent-to-core communication uses mutual TLS
- **Certificate-Based Identity**: Each agent has unique Ed25519 keypair
- **Network Isolation**: Agents operate in untrusted data plane
- **DPI Probe**: Passive network inspection (no packet modification)

### 2. Data Security

- **Encryption at Rest**: Database encryption for PII fields
- **Encryption in Transit**: All communications encrypted
- **Audit Chain**: Immutable audit log with cryptographic chain hashing
- **Data Retention**: 7-year retention policy with automatic cleanup

### 3. Access Control

- **Rootless Operation**: All services run as `ransomeye` user
- **Least Privilege**: Minimal required permissions
- **No Hardcoded Secrets**: All credentials via environment variables
- **Certificate-Based Auth**: No password-based authentication

### 4. Integrity Protection

- **Artifact Verification**: SHA256 hashes for all binaries/models
- **SHAP Explainability**: All AI decisions explainable
- **Drift Detection**: Continuous monitoring for unauthorized changes
- **Change Control**: Pre-update gates with staging requirements

### 5. Availability

- **Fail-Closed Design**: System fails securely on errors
- **Auto-Restart**: systemd Restart=always for all services
- **Health Monitoring**: Continuous verifier checks every 5 minutes
- **Offline Operation**: Fully air-gapped capable

## Threat Model

### Protected Against

- Ransomware attacks
- Data exfiltration
- Unauthorized access
- System tampering
- Audit log manipulation

### Security Boundaries

- **Data Plane**: Untrusted, high-volume, non-authoritative
- **Control Plane**: Trusted, authoritative, policy enforcement
- **Management Plane**: Trusted, configuration and monitoring

## Compliance

- **GDPR**: Data retention and encryption
- **SOC 2**: Audit trails and access controls
- **NIST**: Security controls and monitoring
- **CIS**: Hardening guidelines

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


---

## Compliance Mapping

# RansomEye Compliance Mapping

## Regulatory Compliance

### GDPR (General Data Protection Regulation)

| Requirement | RansomEye Implementation |
|------------|---------------------------|
| Data Minimization | Only collect necessary telemetry |
| Right to Erasure | Retention policy with automatic cleanup |
| Data Encryption | AES-256 encryption for PII fields |
| Audit Trails | Immutable audit log with chain hashing |
| Data Portability | Export capabilities (CSV/HTML/PDF) |

### SOC 2 Type II

| Control | RansomEye Implementation |
|---------|--------------------------|
| Access Controls | Rootless operation, certificate-based auth |
| Audit Logging | Immutable audit log |
| Change Management | Change control guard with staging gates |
| Monitoring | Continuous verifier (5-minute intervals) |
| Incident Response | Incident drill procedures |

### NIST Cybersecurity Framework

| Function | RansomEye Implementation |
|----------|--------------------------|
| Identify | Threat intelligence, asset discovery |
| Protect | Encryption, access controls, hardening |
| Detect | DPI probe, agent telemetry, correlation |
| Respond | Incident response playbooks |
| Recover | Forensic exports, data recovery procedures |

### CIS Benchmarks

| Benchmark | RansomEye Implementation |
|-----------|--------------------------|
| Linux Hardening | Rootless services, minimal permissions |
| Network Security | mTLS, certificate-based identity |
| Logging | Centralized audit logging |
| Monitoring | Continuous health verification |

## Compliance Evidence

### Monthly Reports

Location: `/home/ransomeye/rebuild/docs/enterprise/compliance/monthly/YYYY-MM/`

Contents:
- Audit retention proof
- Data lineage proof
- AI explainability samples (SHAP)

### Audit Trail

Location: Database table `ransomeye.immutable_audit_log`

Features:
- Cryptographic chain hashing
- Immutable (append-only)
- 7-year retention

### Data Retention

Policy: 7 years maximum
Enforcement: Automatic cleanup when >80% disk usage
Evidence: Monthly compliance reports

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


---

## Upgrade Policy

# RansomEye Upgrade Policy

## Overview

All upgrades must follow strict change control procedures to maintain system integrity.

## Pre-Upgrade Requirements

### 1. Version Bump

- New version must be incremented from current version
- Version format: `MAJOR.MINOR.PATCH[-LABEL]`
- Current version: `1.0.0-enterprise-ship`

### 2. Staging Execution

- All changes must be executed in staging environment
- Staging marker required: `/var/lib/ransomeye/staging/staging_execution.json`
- Minimum staging duration: 24 hours

### 3. Verifier Green Status

- Verifier must be green for ≥24 hours before production
- Check: `/var/log/ransomeye/verifier_results.json`
- All checks must pass: services, DB, audit, models, threat intel

### 4. Change Control Approval

- Run change guard: `python3 /home/ransomeye/rebuild/core/change_control/change_guard.py <type> <version>`
- Change guard validates all requirements
- Audit entry created on approval/violation

## Upgrade Procedure

### Step 1: Validate Change

```bash
python3 /home/ransomeye/rebuild/core/change_control/change_guard.py binary 1.0.1
```

### Step 2: Backup

```bash
# Backup database
pg_dump -h localhost -U gagan ransomeye > /var/backups/ransomeye_pre_upgrade_$(date +%Y%m%d_%H%M%S).sql

# Backup configuration
tar -czf /var/backups/ransomeye_config_$(date +%Y%m%d_%H%M%S).tar.gz /etc/ransomeye/
```

### Step 3: Stop Services

```bash
sudo systemctl stop ransomeye-ui
sudo systemctl stop ransomeye-normalization
sudo systemctl stop ransomeye-ingestion
sudo systemctl stop ransomeye-core
```

### Step 4: Apply Changes

- Install new binaries/models
- Update configuration files
- Run database migrations (if any)

### Step 5: Verify

```bash
# Run verifier
python3 /home/ransomeye/rebuild/core/verifier/verifier.py

# Check service status
sudo systemctl status ransomeye-core
```

### Step 6: Start Services

```bash
sudo systemctl start ransomeye-core
sudo systemctl start ransomeye-ingestion
sudo systemctl start ransomeye-normalization
sudo systemctl start ransomeye-ui
```

### Step 7: Post-Upgrade Validation

- Verify all services running
- Check verifier status
- Validate data integrity
- Test critical workflows

## Rollback Procedure

### If Upgrade Fails

1. Stop all services
2. Restore database from backup
3. Restore configuration files
4. Restore binaries (if needed)
5. Restart services
6. Verify system health

## Prohibited Actions

- **No Hot Changes**: Never modify running binaries/configs
- **No Bypass**: Never bypass change control guard
- **No Direct DB Modifications**: Use approved migration scripts only
- **No Unauthorized Updates**: All updates must go through staging

## Change Types

### Binary Changes

- New service binaries
- Updated models (.pkl, .gguf)
- Updated libraries

### Schema Changes

- Database table modifications
- Index changes
- Constraint modifications

### Model Changes

- New model versions
- Model retraining
- SHAP explainability updates

### Config Changes

- Environment variable changes
- Policy file updates
- Systemd unit modifications

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


---

## Support Escalation Path

# RansomEye Support Escalation Path

## Support Contacts

### Primary Support

- **Email**: Gagan@RansomEye.Tech
- **Response Time**: 24 hours (business days)
- **Severity Levels**: P1 (Critical), P2 (High), P3 (Medium), P4 (Low)

## Severity Definitions

### P1 - Critical

- System completely down
- Data loss or corruption
- Security breach
- **Response Time**: 4 hours

### P2 - High

- Major feature unavailable
- Performance degradation >50%
- Service instability
- **Response Time**: 8 hours

### P3 - Medium

- Minor feature unavailable
- Performance degradation <50%
- Non-critical errors
- **Response Time**: 24 hours

### P4 - Low

- Documentation issues
- Feature requests
- General questions
- **Response Time**: 48 hours

## Escalation Process

### Step 1: Self-Service

1. Check operator runbook
2. Review logs: `sudo journalctl -u <service-name> -n 100`
3. Run verifier: `python3 /home/ransomeye/rebuild/core/verifier/verifier.py`
4. Check documentation: `/home/ransomeye/rebuild/docs/`

### Step 2: Collect Information

Before contacting support, collect:

- Service status: `sudo systemctl status ransomeye-*`
- Verifier results: `cat /var/log/ransomeye/verifier_results.json`
- Recent logs: `sudo journalctl -u <service-name> --since "1 hour ago"`
- System information: `uname -a`, `cat /etc/os-release`
- Database status: `psql -h localhost -U gagan -d ransomeye -c "SELECT version();"`

### Step 3: Contact Support

Email: Gagan@RansomEye.Tech

Include:
- Severity level (P1-P4)
- Description of issue
- Steps to reproduce
- Collected information (logs, status, etc.)
- Expected vs actual behavior

### Step 4: Escalation

If no response within SLA:
- P1: Escalate immediately
- P2: Escalate after 8 hours
- P3: Escalate after 24 hours
- P4: Escalate after 48 hours

## Emergency Procedures

### Complete System Failure

1. Stop all services
2. Collect diagnostic information
3. Contact support immediately (P1)
4. Do not attempt manual fixes without support guidance

### Data Corruption

1. Stop all services immediately
2. Do not restart services
3. Contact support (P1)
4. Preserve current state (no modifications)

### Security Incident

1. Isolate affected systems
2. Preserve all logs and evidence
3. Contact support immediately (P1)
4. Follow incident response procedures

## Support Resources

### Documentation

- Operator Runbook: This document
- Architecture: `/home/ransomeye/rebuild/docs/`
- Compliance: `/home/ransomeye/rebuild/docs/enterprise/compliance/`

### Diagnostic Tools

- Verifier: `/home/ransomeye/rebuild/core/verifier/verifier.py`
- Baseline Capture: `/home/ransomeye/rebuild/core/baseline/golden_baseline_capture.py`
- Incident Drill: `/home/ransomeye/rebuild/core/incident/incident_drill.py`

### Log Locations

- Service logs: `sudo journalctl -u <service-name>`
- Verifier logs: `/var/log/ransomeye/verifier_audit.log`
- Application logs: `/var/log/ransomeye/`

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


---

## Document Control

- **Version**: 1.0.0
- **Last Updated**: 2026-01-05T10:40:44.752476+00:00
- **Next Review**: Quarterly
- **Owner**: RansomEye Operations Team

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
