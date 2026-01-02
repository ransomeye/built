# RansomEye Upgrade and Rollback Procedure

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_operations/docs/upgrade_rollback_procedure.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete upgrade and rollback procedures for RansomEye core services

---

## Overview

This document describes the procedures for upgrading and rolling back RansomEye core services. All procedures are **transactional**, **deterministic**, and **fail-closed**.

**Scope:** Core services only (ingestion, correlation, policy, dispatcher, AI advisory)  
**Excluded:** DPI Probe, Linux Agent, Windows Agent (stand-alone modules with their own procedures)

---

## Prerequisites

Before upgrading:

1. **Verify Current Installation**
   ```bash
   ransomeye_operations status
   ```

2. **Backup Current State**
   ```bash
   # Backup install state
   cp /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json \
      /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json.backup
   
   # Backup configuration
   cp -r /home/ransomeye/rebuild/config \
         /home/ransomeye/rebuild/config.backup
   ```

3. **Verify Disk Space**
   - Minimum 10 GB free space required
   - Ensure swap is configured (16 GB minimum for core services)

4. **Verify Version Compatibility**
   - Check version compatibility matrix (below)
   - Verify upgrade path is supported

---

## Version Compatibility Matrix

| Current Version | Target Version | Compatible | Notes |
|----------------|----------------|------------|-------|
| 1.0.0 | 1.1.0 | ✅ Yes | Minor upgrade |
| 1.0.0 | 2.0.0 | ⚠️ Check | Major upgrade - verify compatibility |
| 1.x.x | 1.x.x | ✅ Yes | Patch upgrades always compatible |

**Mixed Versions:**
- Mixed versions are **NOT supported** in production
- All services must run the same version
- Upgrade must be atomic (all services upgraded together)

---

## Upgrade Procedure

### Step 1: Pre-Upgrade Validation

1. **Stop All Services**
   ```bash
   ransomeye_operations stop
   ```

2. **Verify Services Stopped**
   ```bash
   ransomeye_operations status
   # All services should show "inactive"
   ```

3. **Validate Install State**
   ```bash
   # Verify install state is valid and signed
   ransomeye_operations verify-install-state
   ```

4. **Create Upgrade Snapshot**
   ```bash
   # Create snapshot for rollback
   ransomeye_operations create-snapshot --version CURRENT_VERSION
   ```

---

### Step 2: Backup Current Installation

1. **Backup Binaries**
   ```bash
   # Backup current binaries
   mkdir -p /var/lib/ransomeye/upgrades/CURRENT_VERSION/bin
   cp -r /usr/bin/ransomeye_* /var/lib/ransomeye/upgrades/CURRENT_VERSION/bin/
   ```

2. **Backup Configuration**
   ```bash
   # Already done in prerequisites
   # Verify backup exists
   ls -la /home/ransomeye/rebuild/config.backup
   ```

3. **Backup Install State**
   ```bash
   # Already done in prerequisites
   # Verify backup exists
   ls -la /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json.backup
   ```

---

### Step 3: Install New Version

1. **Run Installer with Upgrade Flag**
   ```bash
   cd /home/ransomeye/rebuild
   ./install.sh --upgrade --target-version NEW_VERSION --accept-eula
   ```

2. **Verify Installation**
   ```bash
   ransomeye_operations verify-install-state
   ```

3. **Validate Configuration Compatibility**
   ```bash
   # Verify configuration is compatible with new version
   ransomeye_operations validate-config --version NEW_VERSION
   ```

---

### Step 4: Migrate Configuration (If Required)

1. **Check Configuration Migration Requirements**
   ```bash
   # Review migration guide for new version
   cat /home/ransomeye/rebuild/ransomeye_operations/docs/CHANGELOG.md
   ```

2. **Apply Configuration Migrations**
   ```bash
   # If migration script exists
   ransomeye_operations migrate-config --from CURRENT_VERSION --to NEW_VERSION
   ```

3. **Verify Migrated Configuration**
   ```bash
   ransomeye_operations validate-config --version NEW_VERSION
   ```

---

### Step 5: Start Services

1. **Start Services in Dependency Order**
   ```bash
   ransomeye_operations start
   ```

2. **Verify Services Started**
   ```bash
   ransomeye_operations status
   # All services should show "active"
   ```

3. **Verify Health Checks**
   ```bash
   # Check service health
   systemctl status ransomeye-core
   systemctl status ransomeye-ingestion
   systemctl status ransomeye-correlation
   systemctl status ransomeye-policy
   systemctl status ransomeye-enforcement
   ```

---

### Step 6: Post-Upgrade Validation

1. **Verify Service Functionality**
   ```bash
   # Check logs for errors
   journalctl -u ransomeye-core -n 100
   journalctl -u ransomeye-ingestion -n 100
   # ... repeat for all services
   ```

2. **Verify Data Integrity**
   ```bash
   # Verify data pipelines are working
   ransomeye_operations verify-data-integrity
   ```

3. **Performance Validation**
   ```bash
   # Monitor service performance
   ransomeye_operations monitor --duration 300
   ```

---

## Rollback Procedure

### When to Rollback

Rollback should be performed if:
- Services fail to start after upgrade
- Critical errors detected in logs
- Data integrity issues discovered
- Performance degradation observed
- Configuration migration failures

---

### Step 1: Stop All Services

```bash
ransomeye_operations stop
```

---

### Step 2: Verify Rollback Snapshot

```bash
# List available snapshots
ransomeye_operations list-snapshots

# Verify target snapshot exists
ransomeye_operations verify-snapshot --version TARGET_VERSION
```

---

### Step 3: Restore Previous Version

1. **Restore Binaries**
   ```bash
   # Restore previous binaries
   cp -r /var/lib/ransomeye/upgrades/TARGET_VERSION/bin/* /usr/bin/
   ```

2. **Restore Configuration**
   ```bash
   # Restore configuration
   rm -rf /home/ransomeye/rebuild/config
   cp -r /home/ransomeye/rebuild/config.backup /home/ransomeye/rebuild/config
   ```

3. **Restore Install State**
   ```bash
   # Restore install state
   cp /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json.backup \
      /home/ransomeye/rebuild/ransomeye_installer/config/install_state.json
   ```

---

### Step 4: Verify Rollback

1. **Validate Install State**
   ```bash
   ransomeye_operations verify-install-state
   ```

2. **Validate Configuration**
   ```bash
   ransomeye_operations validate-config --version TARGET_VERSION
   ```

---

### Step 5: Start Services

```bash
ransomeye_operations start
ransomeye_operations status
```

---

### Step 6: Post-Rollback Validation

1. **Verify Service Functionality**
   ```bash
   journalctl -u ransomeye-core -n 100
   # ... repeat for all services
   ```

2. **Verify Data Integrity**
   ```bash
   ransomeye_operations verify-data-integrity
   ```

---

## Transactional Upgrade Validation

### Atomic Upgrade Guarantee

The upgrade process ensures:
- **All-or-Nothing**: Either all services upgrade successfully, or all remain at previous version
- **Consistent State**: No mixed versions in production
- **Rollback Safety**: Previous version always restorable

### Validation Checks

1. **Pre-Upgrade Checks**
   - Install state valid
   - Sufficient disk space
   - Services stopped
   - Backup created

2. **During Upgrade**
   - Signature verification
   - Configuration compatibility
   - Version compatibility

3. **Post-Upgrade Checks**
   - Services started
   - Health checks pass
   - Data integrity verified

---

## Failure Scenarios

### Scenario 1: Upgrade Fails Mid-Process

**Response:**
1. Upgrade process aborts automatically
2. Previous version remains installed
3. Services remain stopped
4. Rollback to previous version required

**Recovery:**
```bash
ransomeye_operations rollback --version PREVIOUS_VERSION
```

---

### Scenario 2: Services Fail to Start After Upgrade

**Response:**
1. Services remain stopped
2. Check logs for errors
3. Perform rollback if critical errors

**Recovery:**
```bash
# Check logs
journalctl -u ransomeye-core -n 100

# Rollback if needed
ransomeye_operations rollback --version PREVIOUS_VERSION
```

---

### Scenario 3: Configuration Migration Fails

**Response:**
1. Configuration migration aborts
2. Previous configuration preserved
3. Upgrade cannot proceed

**Recovery:**
```bash
# Manual configuration migration may be required
# Or rollback to previous version
ransomeye_operations rollback --version PREVIOUS_VERSION
```

---

## Best Practices

1. **Always Create Backups**
   - Never upgrade without backups
   - Verify backups before upgrade

2. **Test in Staging First**
   - Test upgrade procedure in staging environment
   - Validate configuration migrations

3. **Monitor After Upgrade**
   - Monitor services for 24-48 hours after upgrade
   - Watch for errors or performance issues

4. **Keep Snapshots**
   - Maintain snapshots for at least 2 previous versions
   - Archive old snapshots for compliance

5. **Document Issues**
   - Document any issues encountered during upgrade
   - Update procedures based on lessons learned

---

## Automated Upgrade (Optional)

For automated upgrades:

```bash
# Automated upgrade script
ransomeye_operations upgrade \
  --target-version NEW_VERSION \
  --accept-eula \
  --auto-rollback-on-failure
```

**Features:**
- Automatic backup creation
- Automatic rollback on failure
- Health check validation
- Email notifications (if configured)

---

## Support

For upgrade/rollback support:
- **Email:** Gagan@RansomEye.Tech
- **Documentation:** `/home/ransomeye/rebuild/ransomeye_operations/docs/`

---

## Appendix: Version History

| Version | Release Date | Breaking Changes | Configuration Changes |
|---------|--------------|------------------|----------------------|
| 1.0.0 | Initial | N/A | N/A |

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

