# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/production_change_prohibitions.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Production Change Prohibition Register - Formal prohibition register for production changes (PROMPT-66-D)

# Production Change Prohibition Register (PROMPT-66-D)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Overview

This register provides a **formal prohibition list** of actions that operations teams, vendors, and customers are forbidden to perform in production. It also defines actions that customers may perform safely and actions that require lifecycle restart.

**Purpose:** Prevent accidental or intentional weakening of security assurances.

---

## Operations Team Prohibitions

### Core Binary Modifications

**FORBIDDEN Actions:**

1. ❌ **Modify core binaries**
   - Cannot modify any binary listed in `ARTIFACT_HASHES.txt`
   - Cannot replace binaries
   - Cannot patch binaries
   - **Reason:** Ship seal violation

2. ❌ **Modify ARTIFACT_HASHES.txt**
   - Cannot modify hash list
   - Cannot add hashes
   - Cannot remove hashes
   - Cannot change file permissions
   - **Reason:** Ship seal violation

3. ❌ **Modify ship seal enforcer**
   - Cannot modify `/core/assurance/ship_seal_enforcer.py`
   - Cannot bypass ship seal checks
   - Cannot disable ship seal enforcement
   - **Reason:** Ship seal violation

4. ❌ **Modify verifier code**
   - Cannot modify `/core/verifier/verifier.py`
   - Cannot bypass verifier checks
   - Cannot disable verifier timer
   - **Reason:** Ship seal violation

5. ❌ **Modify model artifacts**
   - Cannot modify model files (`.model`, `.pkl`, `.gguf`)
   - Cannot replace models
   - Cannot patch models
   - **Reason:** Ship seal violation

### Audit Log Modifications

**FORBIDDEN Actions:**

1. ❌ **Delete audit log entries**
   - Cannot delete from `ransomeye.immutable_audit_log`
   - Cannot truncate audit log
   - Cannot modify audit entries
   - **Reason:** Immutable audit log, chain break

2. ❌ **Modify audit chain**
   - Cannot modify chain hashes
   - Cannot break chain links
   - Cannot insert entries with invalid chain hash
   - **Reason:** Audit chain integrity violation

3. ❌ **Bypass audit logging**
   - Cannot disable audit logging
   - Cannot skip audit entries
   - Cannot modify audit configuration
   - **Reason:** Audit trail integrity

### Assurance Mechanism Bypass

**FORBIDDEN Actions:**

1. ❌ **Bypass ship seal checks**
   - Cannot skip ship seal verification
   - Cannot disable ship seal enforcer
   - Cannot override ship seal enforcement
   - **Reason:** Ship seal violation

2. ❌ **Bypass verifier checks**
   - Cannot skip verifier execution
   - Cannot disable verifier timer
   - Cannot override verifier results
   - **Reason:** Integrity violation

3. ❌ **Disable assurance mechanisms**
   - Cannot disable assurance mode
   - Cannot remove assurance lock
   - Cannot bypass protections
   - **Reason:** Assurance violation

### Configuration Modifications

**FORBIDDEN Actions:**

1. ❌ **Hardcode credentials**
   - Cannot hardcode passwords
   - Cannot hardcode secrets
   - Cannot hardcode keys
   - **Reason:** Security violation

2. ❌ **Modify systemd service files (if protected)**
   - Cannot modify protected service files
   - Cannot bypass service checks
   - Cannot disable service protection
   - **Reason:** Ship seal violation (if protected)

### Evidence Modifications

**FORBIDDEN Actions:**

1. ❌ **Delete evidence**
   - Cannot delete evidence files
   - Cannot modify evidence
   - Cannot tamper with evidence
   - **Reason:** Evidence integrity violation

2. ❌ **Modify evidence hashes**
   - Cannot modify evidence file hashes
   - Cannot falsify evidence
   - Cannot tamper with evidence chain of custody
   - **Reason:** Evidence integrity violation

---

## Vendor Prohibitions

### All Operations Team Prohibitions Apply

**Vendors are subject to ALL operations team prohibitions.**

### Additional Vendor Prohibitions

**FORBIDDEN Actions:**

1. ❌ **Override ship seal enforcement**
   - Cannot provide override mechanisms
   - Cannot bypass ship seal checks
   - Cannot disable ship seal enforcement
   - **Reason:** Vendor non-repudiation violation

2. ❌ **Provide backdoor access**
   - Cannot provide hidden access
   - Cannot provide vendor-only access
   - Cannot bypass customer controls
   - **Reason:** Vendor non-repudiation violation

3. ❌ **Modify customer systems without authorization**
   - Cannot access customer systems without authorization
   - Cannot modify customer systems without authorization
   - Cannot delete customer evidence
   - **Reason:** Customer trust violation

4. ❌ **Interfere with legal proceedings**
   - Cannot modify evidence during legal proceedings
   - Cannot delete logs during legal proceedings
   - Cannot bypass audit trail during legal proceedings
   - **Reason:** Legal compliance violation

5. ❌ **Interfere with regulatory investigations**
   - Cannot modify systems during regulatory investigations
   - Cannot delete evidence during regulatory investigations
   - Cannot bypass controls during regulatory investigations
   - **Reason:** Regulatory compliance violation

### Vendor Intervention Boundaries

**Vendor intervention is PROHIBITED during:**
- Active security incidents
- Legal proceedings
- Regulatory investigations
- Forensic analysis
- Evidence preservation

**Vendor intervention is ALLOWED only with:**
- Explicit customer authorization
- Written customer consent
- Customer supervision
- Full documentation

---

## Customer Safe Actions

### ✅ Allowed Actions

**Customers may safely perform:**

1. ✅ **Configure environment variables**
   - May set environment variables via `.env` files
   - May configure database connections
   - May configure service parameters
   - **Note:** Must not hardcode credentials

2. ✅ **Manage operational data**
   - May insert operational data into database
   - May update operational data
   - May delete operational data (not audit log)
   - **Note:** Must not modify audit log

3. ✅ **Manage log files**
   - May rotate log files
   - May archive log files
   - May delete old log files (not audit log)
   - **Note:** Must preserve audit log

4. ✅ **Perform backups**
   - May create backups (non-mutating)
   - May restore from backups (with verification)
   - May archive backups
   - **Note:** Must verify restore integrity

5. ✅ **Run verification tools**
   - May run customer verifier
   - May run vendor non-repudiation scanner
   - May run ship seal enforcer
   - May generate evidence bundles
   - **Note:** All verification tools are read-only

6. ✅ **Export data**
   - May export audit chain
   - May export evidence
   - May export compliance reports
   - **Note:** Exports are read-only

7. ✅ **Manage services**
   - May start/stop services (with verification)
   - May restart services (with verification)
   - May check service status
   - **Note:** Services verify ship seal on startup

8. ✅ **Monitor system**
   - May monitor verifier results
   - May review audit logs
   - May check service status
   - May review evidence
   - **Note:** All monitoring is read-only

### ⚠️ Conditional Actions

**Customers may perform with caution:**

1. ⚠️ **Restore from backup**
   - May restore database (verify integrity first)
   - May restore configuration (verify hash first)
   - **Condition:** Must verify ship seal after restore
   - **Condition:** Must verify audit chain integrity
   - **Condition:** Must not restore over production without approval

2. ⚠️ **Modify configuration**
   - May modify non-core configuration
   - **Condition:** Must not modify protected files
   - **Condition:** Must verify ship seal after changes
   - **Condition:** Must not hardcode credentials

---

## Actions Requiring Lifecycle Restart

### Definition

**Lifecycle Restart:** Complete system reinstallation with new shipment version.

### Actions Requiring Lifecycle Restart

**The following actions require lifecycle restart:**

1. 🔄 **Ship seal changes**
   - Any modification to `ARTIFACT_HASHES.txt`
   - Any addition of new binaries to ship seal
   - Any change to ship seal format
   - **Reason:** Ship seal is immutable

2. 🔄 **Core binary updates**
   - Any update to core binaries
   - Any replacement of core binaries
   - Any patching of core binaries
   - **Reason:** Ship seal violation

3. 🔄 **Model updates**
   - Any update to model artifacts
   - Any replacement of models
   - Any new model versions
   - **Reason:** Ship seal violation

4. 🔄 **Verifier updates**
   - Any update to verifier code
   - Any change to verifier logic
   - Any modification to verifier checks
   - **Reason:** Ship seal violation

5. 🔄 **Ship seal enforcer updates**
   - Any update to ship seal enforcer
   - Any change to enforcement logic
   - Any modification to verification process
   - **Reason:** Ship seal violation

6. 🔄 **Assurance mechanism changes**
   - Any change to assurance mechanisms
   - Any modification to protection logic
   - Any update to enforcement rules
   - **Reason:** Assurance violation

### Lifecycle Restart Procedure

**Required steps:**

1. **Preserve Evidence:**
   - Export complete audit chain
   - Generate evidence bundle
   - Preserve all evidence
   - Document current state

2. **Backup System:**
   - Backup database (non-mutating)
   - Backup configuration
   - Backup evidence
   - Verify backup integrity

3. **Install New Version:**
   - Install new shipment version
   - Verify new ship seal
   - Verify new binaries
   - Verify new configuration

4. **Restore Data:**
   - Restore database (with verification)
   - Restore configuration (with verification)
   - Verify audit chain integrity
   - Verify ship seal integrity

5. **Verify System:**
   - Run customer verifier
   - Run vendor non-repudiation scanner
   - Verify all services
   - Verify audit chain

6. **Document Process:**
   - Document lifecycle restart
   - Document evidence preserved
   - Document verification results
   - Update compliance records

---

## Prohibition Enforcement

### Technical Enforcement

**Enforced by:**

1. **Ship Seal Enforcer:**
   - Detects binary modifications
   - Blocks service startup on violation
   - Generates SYSTEM_INTEGRITY_VIOLATION

2. **Continuous Verifier:**
   - Detects drift and violations
   - Enters fail-closed state on violation
   - Generates SYSTEM_INTEGRITY_VIOLATION

3. **Immutable Audit Log:**
   - Prevents audit log modification
   - Maintains chain integrity
   - Detects chain breaks

4. **Vendor Non-Repudiation:**
   - Scans for backdoor patterns
   - Detects override mechanisms
   - Verifies no bypass paths

### Procedural Enforcement

**Enforced by:**

1. **Operations Playbook:**
   - Defines allowed actions
   - Defines forbidden actions
   - Provides procedures

2. **Incident Response Matrix:**
   - Defines violation response
   - Defines escalation procedures
   - Defines evidence preservation

3. **Compliance Procedures:**
   - Defines compliance requirements
   - Defines regulatory obligations
   - Defines legal requirements

---

## Prohibition Violation Consequences

### Technical Consequences

**On Violation:**

1. **Ship Seal Violation:**
   - Service fails to start
   - SYSTEM_INTEGRITY_VIOLATION audit entry
   - Verifier enters fail-closed state
   - System blocks operation

2. **Audit Log Violation:**
   - Chain break detected
   - Integrity violation logged
   - System enters fail-closed state
   - Evidence preserved

3. **Verifier Violation:**
   - Verifier fails
   - SYSTEM_INTEGRITY_VIOLATION audit entry
   - System enters fail-closed state
   - Evidence preserved

### Procedural Consequences

**On Violation:**

1. **Incident Response:**
   - Incident declared
   - Evidence preserved
   - Compliance team notified
   - Legal team notified (if required)

2. **Regulatory Consequences:**
   - Regulatory notification (if required)
   - Compliance violation
   - Potential regulatory action
   - Evidence provided to regulators

3. **Legal Consequences:**
   - Legal documentation
   - Potential legal action
   - Court proceedings (if required)
   - Evidence provided to courts

---

## Exception Process

### Exception Request

**Exception requests must include:**

1. **Justification:**
   - Business justification
   - Technical justification
   - Risk assessment
   - Mitigation plan

2. **Approval:**
   - Security team approval
   - Compliance team approval
   - Legal team approval (if required)
   - Executive approval (if required)

3. **Documentation:**
   - Exception request
   - Approval documentation
   - Risk assessment
   - Mitigation plan

### Exception Limitations

**Exceptions are NOT granted for:**

- Ship seal modifications
- Audit log modifications
- Core binary modifications
- Bypass mechanisms
- Override mechanisms

**Exceptions may be granted for:**

- Non-core configuration changes
- Operational data management
- Backup and restore procedures
- Evidence export procedures

---

## Conclusion

This register ensures **operations cannot weaken security** and **customers cannot accidentally break assurances**. All prohibitions are enforced technically and procedurally, with clear consequences for violations.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

