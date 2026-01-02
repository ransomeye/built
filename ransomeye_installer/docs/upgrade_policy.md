# Upgrade Policy

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_installer/docs/upgrade_policy.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Upgrade policy and procedures

---

## Overview

Upgrades preserve existing configuration while updating components.

---

## Upgrade Process

### 1. Pre-Upgrade Validation

- Verify existing installation state
- Validate state signature
- Check prerequisites (OS, disk, swap)

### 2. Backup Current State

- Backup `install_state.json`
- Backup `config/retention.txt`
- Backup cryptographic identity

### 3. Update Components

- Update codebase
- Regenerate systemd units
- Update trust infrastructure (if needed)

### 4. State Update

- Update state to 'UPGRADED'
- Preserve EULA acceptance
- Preserve retention configuration
- Preserve identity (unless regenerated)

### 5. Service Restart

- Restart services with new code
- Verify services start correctly

---

## State Preservation

Upgrades preserve:

- EULA acceptance (unless EULA changed)
- Retention configuration
- Cryptographic identity
- Service configuration

---

## Breaking Changes

If upgrade includes breaking changes:

1. **EULA Changes**
   - New EULA must be accepted
   - Previous acceptance invalidated

2. **Identity Regeneration**
   - May require new identity
   - Old identity preserved for migration

3. **Configuration Migration**
   - Old config migrated to new format
   - Validation performed

---

## Rollback Procedure

If upgrade fails:

1. Restore backup state:
   ```bash
   cp install_state.json.backup install_state.json
   ```

2. Restore codebase to previous version

3. Restart services

---

## Last Updated

Phase 1 Implementation

