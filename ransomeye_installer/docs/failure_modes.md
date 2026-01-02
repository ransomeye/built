# Installer Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_installer/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Documentation of all failure modes and recovery procedures

---

## Overview

All installer failures are fail-closed. No partial installations are allowed.

---

## Failure Modes

### 1. OS Unsupported

**Condition:** OS is not Ubuntu >=22.04 or RHEL >=8

**Error:** "OS check failed: Unsupported distribution"

**Recovery:**
- Upgrade OS to supported version
- Re-run installer

**Prevention:** Check OS before installation

---

### 2. Disk Insufficient

**Condition:** Free space < 10GB OR usage >= 90%

**Error:** "Disk check failed: Insufficient disk space"

**Recovery:**
- Free up disk space
- Expand disk partition
- Re-run installer

**Prevention:** Ensure >= 10GB free before installation

---

### 3. Swap Missing

**Condition:** Swap < 16GB OR < RAM size

**Error:** "Swap check failed: Insufficient swap"

**Recovery:**
- Create swap file/partition
- Ensure swap >= 16GB or equal to RAM
- Re-run installer

**Prevention:** Configure swap before installation

---

### 4. EULA Not Accepted

**Condition:** User declines EULA

**Error:** "EULA not accepted. Installation aborted."

**Recovery:**
- Re-run installer
- Accept EULA when prompted

**Prevention:** Review EULA before installation

---

### 5. Retention Invalid

**Condition:** Retention values outside valid ranges

**Error:** "Retention validation failed: [reason]"

**Recovery:**
- Re-run installer
- Provide valid retention values
- Use defaults if unsure

**Prevention:** Use default values

---

### 6. Identity Generation Failed

**Condition:** Cannot generate cryptographic identity

**Error:** "Error generating identity: [reason]"

**Recovery:**
- Check disk permissions
- Ensure crypto directory is writable
- Re-run installer

**Prevention:** Ensure proper permissions

---

### 7. State Save Failed

**Condition:** Cannot save or sign install state

**Error:** "Error saving state: [reason]"

**Recovery:**
- Check disk space
- Check permissions
- Verify trust infrastructure
- Re-run installer

**Prevention:** Ensure trust infrastructure initialized

---

## Partial Installation Prevention

The installer prevents partial installations by:

1. **Validation Before Actions**
   - All prerequisites validated before any changes
   - No state changes if validation fails

2. **Atomic Operations**
   - State saved only after all steps complete
   - Rollback on any failure

3. **Signed State**
   - State must be valid and signed
   - Services check state before starting

---

## Recovery Procedures

### Complete Reinstall

If installation fails:

1. Remove partial state:
   ```bash
   rm -f /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json
   ```

2. Fix the issue (see failure modes above)

3. Re-run installer:
   ```bash
   python3 -m ransomeye_installer.installer
   ```

### State Corruption

If state is corrupted:

1. Verify state signature:
   ```bash
   python3 -m ransomeye_trust.verify_tool /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json
   ```

2. If invalid, remove and reinstall

---

## Last Updated

Phase 1 Implementation

