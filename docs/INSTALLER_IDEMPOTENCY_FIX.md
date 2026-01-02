# Installer Idempotency Fix - Production Blocking Bug Resolution

**Date:** 2025-12-28  
**Status:** ✅ RESOLVED  
**Priority:** P0 (Production Blocking)  
**Issue:** Global Validator fails on re-install due to stale systemd units

---

## Problem Statement

### Root Cause
When re-running `install.sh`, the Global Validator would **FAIL** in POST-INSTALL mode because:

1. **Old systemd units** already existed in `/etc/systemd/system/`
2. These old units still referenced: `/home/ransomeye/rebuild`
3. The installer's `install_units()` method did **NOT** overwrite existing units
4. Global Validator detected the path violation and blocked installation

### User Impact
- Re-running `install.sh` would **FAIL** even though it should be idempotent
- Manual cleanup required before re-installation
- No automated remediation path
- Breaks CI/CD and automated deployment workflows

---

## Solution Architecture

### 1. **Idempotent Unit Installation** (`systemd_writer.py`)

**Before:**
```python
def install_units(self) -> bool:
    for unit_file in self.SYSTEMD_DIR.glob("*.service"):
        target = Path(f"/etc/systemd/system/{unit_file.name}")
        shutil.copy2(unit_file, target)  # Would fail or skip if exists
```

**After:**
```python
def install_units(self) -> bool:
    # 1. Detect existing RansomEye units
    existing_units = list(Path("/etc/systemd/system").glob("ransomeye-*.service"))
    
    if existing_units:
        # 2. Stop and disable existing services
        for existing_unit in existing_units:
            systemctl stop {service}
            systemctl disable {service}
    
    # 3. Force-overwrite units
    for unit_file in self.SYSTEMD_DIR.glob("*.service"):
        target = Path(f"/etc/systemd/system/{unit_file.name}")
        shutil.copy2(unit_file, target)  # Always overwrites
        print(f"[INSTALL] Replacing existing systemd unit: {unit_file.name}")
    
    # 4. Reload systemd
    systemctl daemon-reload
```

**Key Changes:**
- **Automatic detection** of existing units
- **Graceful stop/disable** of services before replacement
- **Force-overwrite** mode (idempotent)
- **Clear logging** of replacement actions

### 2. **Pre-Install Sanitation** (`install.sh`)

Added explicit pre-install check that:
1. Scans `/etc/systemd/system/` for existing `ransomeye-*.service` files
2. Checks each unit for `/home/ransomeye/rebuild` references
3. Logs a clear warning with list of stale units
4. Informs user that units will be automatically replaced

**Output Example:**
```
=========================================================================
STALE SYSTEMD UNITS DETECTED
=========================================================================

The following systemd units from a previous installation reference
/home/ransomeye/rebuild and will be automatically replaced:

  • ransomeye-intelligence.service
  • ransomeye-correlation.service
  • ransomeye-policy.service

This installer will:
  1. Stop and disable all stale services
  2. Overwrite stale units with new units (referencing /opt/ransomeye)
  3. Reload systemd daemon

This is normal for re-installation and ensures idempotency.
=========================================================================
```

### 3. **Enhanced Global Validator Messaging** (`systemd_installer.py`)

**Before:**
```
INSTALLED UNIT VIOLATION: systemd unit 'ransomeye-intelligence.service' 
in /etc/systemd/system/ references /home path in WorkingDirectory 
(must use /opt/ransomeye)
```

**After:**
```
INSTALLED UNIT VIOLATION: systemd unit 'ransomeye-intelligence.service' 
in /etc/systemd/system/ references /home path in WorkingDirectory 
(must use /opt/ransomeye). 

REMEDIATION: Re-run installer to replace stale unit with correct 
/opt/ransomeye paths.
```

**Added to violation details:**
```json
{
  "remediation": "Re-run installer with: sudo ./install.sh (installer will automatically replace stale units)"
}
```

### 4. **Regression Test** (`systemd_idempotency_test.py`)

Created comprehensive test suite:

```python
class TestSystemdIdempotency(unittest.TestCase):
    
    def test_install_units_overwrites_stale_unit(self):
        """Test that install_units() overwrites stale units with /home paths."""
        # 1. Create stale unit with /home/ransomeye/rebuild
        # 2. Run install_units()
        # 3. Verify unit is REPLACED (not preserved)
        # 4. Verify /home paths are GONE
        # 5. Verify /opt paths are PRESENT
    
    def test_install_units_without_existing_unit(self):
        """Test clean installation (no pre-existing units)."""
    
    def test_install_units_fails_without_runtime_root(self):
        """Test fail-closed behavior when runtime root is missing."""
```

**Test Coverage:**
- ✅ Stale unit replacement (main regression scenario)
- ✅ Clean installation (no existing units)
- ✅ Fail-closed validation (missing runtime root)

---

## Validation Workflow

### Pre-Install Phase
```
┌─────────────────────────────────────────┐
│  install.sh: Pre-Install Sanitation     │
├─────────────────────────────────────────┤
│  1. Scan /etc/systemd/system/           │
│  2. Detect ransomeye-*.service files    │
│  3. Check for /home/ransomeye/rebuild   │
│  4. Log stale units warning             │
│  5. Continue to installation            │
└─────────────────────────────────────────┘
```

### Installation Phase
```
┌─────────────────────────────────────────┐
│  systemd_writer.install_units()         │
├─────────────────────────────────────────┤
│  1. Detect existing units               │
│  2. Stop existing services              │
│  3. Disable existing services           │
│  4. Force-copy new units (overwrite)    │
│  5. Reload systemd daemon               │
│  6. Log replacement actions             │
└─────────────────────────────────────────┘
```

### Post-Install Phase
```
┌─────────────────────────────────────────┐
│  Global Validator: POST-INSTALL Mode    │
├─────────────────────────────────────────┤
│  1. Scan /etc/systemd/system/           │
│  2. Read installed unit files           │
│  3. Validate paths (/opt/ransomeye)     │
│  4. PASS ✅ (no /home paths detected)   │
└─────────────────────────────────────────┘
```

---

## Security & Safety Guarantees

### 1. **Fail-Closed Validation**
- Installer still requires runtime root (`/opt/ransomeye`) to exist
- Unit installation fails if runtime root is missing
- No weakening of security validation

### 2. **Service Isolation**
- Services are stopped before replacement
- No mixed state between old/new units
- Systemd daemon reload ensures clean state

### 3. **Idempotency**
- Re-running installer **always** converges to correct state
- No manual cleanup required
- Safe for automated deployments

### 4. **Rollback Safety**
- If installation fails after unit replacement, rollback removes all units
- No partial/broken state possible
- Fail-closed error handling throughout

---

## Testing & Validation

### Unit Tests
```bash
# Run regression tests
cd /home/ransomeye/rebuild
python3 -m pytest ransomeye_installer/tests/systemd_idempotency_test.py -v
```

**Expected Output:**
```
test_install_units_overwrites_stale_unit ..................... PASS
test_install_units_without_existing_unit ..................... PASS
test_install_units_fails_without_runtime_root ................ PASS
```

### Integration Test
```bash
# 1. Run installer first time
sudo ./install.sh

# 2. Verify Global Validator passes
python3 /home/ransomeye/rebuild/core/global_validator/validate.py

# 3. Re-run installer (idempotency test)
sudo ./install.sh

# 4. Verify Global Validator still passes
python3 /home/ransomeye/rebuild/core/global_validator/validate.py
```

**Expected Behavior:**
- First install: Clean installation, all units installed
- Second install: Stale units detected and replaced
- Both runs: Global Validator **PASS** ✅

### Manual Regression Test
```bash
# Simulate stale unit from old installation
sudo cp /home/ransomeye/rebuild/systemd/ransomeye-intelligence.service \
        /etc/systemd/system/

# Manually corrupt it (add /home path)
sudo sed -i 's|/opt/ransomeye|/home/ransomeye/rebuild|g' \
        /etc/systemd/system/ransomeye-intelligence.service

# Run Global Validator (should FAIL)
python3 /home/ransomeye/rebuild/core/global_validator/validate.py
# Expected: FAIL with "INSTALLED UNIT VIOLATION"

# Re-run installer to fix
sudo ./install.sh
# Expected: Detects stale unit, replaces it, Global Validator PASS
```

---

## Deployment & Rollout

### Backward Compatibility
- ✅ Safe to deploy to all environments
- ✅ No breaking changes to API or behavior
- ✅ Existing installations will be automatically fixed on next run

### Rollout Plan
1. **Stage 1:** Deploy to dev environment, run regression tests
2. **Stage 2:** Deploy to QA, validate idempotency
3. **Stage 3:** Deploy to production, monitor install logs

### Monitoring
Watch for log messages:
```
[INSTALL] Detected existing RansomEye systemd units - preparing for replacement
[INSTALL] Replacing existing systemd unit: ransomeye-intelligence.service
[INSTALL] Successfully installed 7 systemd units (disabled by default)
```

---

## Files Modified

### Core Changes
1. `/home/ransomeye/rebuild/ransomeye_installer/services/systemd_writer.py`
   - Modified `install_units()` method
   - Added automatic detection and replacement logic
   - Enhanced logging

2. `/home/ransomeye/rebuild/install.sh`
   - Added pre-install sanitation check
   - Enhanced stale unit detection
   - Improved user messaging

3. `/home/ransomeye/rebuild/core/global_validator/systemd_installer.py`
   - Enhanced validation error messages
   - Added remediation instructions

### Test & Documentation
4. `/home/ransomeye/rebuild/ransomeye_installer/tests/systemd_idempotency_test.py`
   - New regression test suite

5. `/home/ransomeye/rebuild/docs/INSTALLER_IDEMPOTENCY_FIX.md`
   - This documentation file

---

## Acceptance Criteria

- [x] `install.sh` is idempotent (can be re-run safely)
- [x] Re-running install **always** converges systemd units to `/opt/ransomeye`
- [x] Global Validator **PASSES** on clean re-run
- [x] Stale units are automatically detected and logged
- [x] No manual cleanup required
- [x] Fail-closed security validation maintained
- [x] Regression test covers main scenario
- [x] Documentation complete

---

## Related Issues & References

- **Issue:** Production-blocking installer idempotency bug
- **Root Cause:** Existing systemd units not overwritten during re-install
- **Security Impact:** None (fix maintains fail-closed validation)
- **User Impact:** High (blocks re-installation workflows)

---

## Contact & Support

**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Support:** Gagan@RansomEye.Tech  
**Project:** RansomEye Enterprise Security Platform  

---

**© RansomEye.Tech | Enterprise-Excellent Cyber Security**

