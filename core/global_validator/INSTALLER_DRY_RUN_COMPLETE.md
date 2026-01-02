# Installer Dry-Run & Manifest Audit - Complete (Prompt #12)

**Path:** `/home/ransomeye/rebuild/core/global_validator/INSTALLER_DRY_RUN_COMPLETE.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Date:** 2025-12-28

## Status: ✅ COMPLETE

Prompt #12 (Installer Dry-Run & Manifest Audit) has been successfully implemented.

---

## Implementation Summary

### ✅ 1. DRY_RUN Mode Added

**Environment Variable:** `RANSOMEYE_DRY_RUN=1`

**Behavior:**
- ✅ NO files written (except manifest to project root)
- ✅ NO systemd units installed
- ✅ NO services started
- ✅ All actions logged only
- ✅ Exit code 0 on success

**Location:** `/home/ransomeye/rebuild/install.sh`

**Key Features:**
- Root privilege check skipped in dry-run
- EULA acceptance skipped in dry-run
- All destructive operations skipped (user creation, binary installation, systemd installation)
- Global validator still runs (mandatory)
- Manifest generation still runs (mandatory)

---

### ✅ 2. Install Manifest Generator Enhanced

**File:** `/home/ransomeye/rebuild/ransomeye_installer/manifest_generator.py`

**Manifest Location:**
- Normal mode: `/var/lib/ransomeye/install_manifest.json`
- Dry-run mode: `/home/ransomeye/rebuild/install_manifest.json`

**Manifest Contents:**
- ✅ `binaries`: List of binaries to be installed (name, path, will_install, exists)
- ✅ `systemd_units`: List of systemd units (name, module, source_path, target_path)
- ✅ `config_paths`: List of configuration paths
- ✅ `ports`: Port configuration (CORE_API_PORT, FRONTEND_PORT, BACKEND_API_PORT)
- ✅ `database_migrations`: Database migration list (currently empty)
- ✅ `agent_exclusions`: List of standalone agents excluded from installation
- ✅ `modules`: All installed modules with hashes, phases, types
- ✅ `dry_run`: Boolean flag indicating dry-run mode
- ✅ `guardrails_spec_hash`: Hash of guardrails specification

**Sample Manifest Structure:**
```json
{
  "install_timestamp": "2025-12-28T08:45:12.931325Z",
  "project_root": "/home/ransomeye/rebuild",
  "installer_version": "1.0.0",
  "dry_run": true,
  "modules": { ... },
  "binaries": [
    {
      "name": "ransomeye-guardrails",
      "path": "/usr/bin/ransomeye-guardrails",
      "will_install": true,
      "exists": false
    }
  ],
  "systemd_units": [
    {
      "name": "ransomeye-intelligence.service",
      "module": "ransomeye_intelligence",
      "source_path": "/home/ransomeye/rebuild/systemd/ransomeye-intelligence.service",
      "target_path": "/etc/systemd/system/ransomeye-intelligence.service"
    }
  ],
  "config_paths": [
    "/etc/ransomeye/",
    "/var/lib/ransomeye/",
    "/run/ransomeye/",
    "/home/ransomeye/rebuild/ransomeye_installer/config/"
  ],
  "ports": {
    "CORE_API_PORT": 8443,
    "FRONTEND_PORT": 3000,
    "BACKEND_API_PORT": 8080
  },
  "database_migrations": [],
  "agent_exclusions": [],
  "guardrails_spec_hash": "..."
}
```

---

### ✅ 3. Validator Hook Added

**Location:** `/home/ransomeye/rebuild/install.sh` (Step 0)

**Behavior:**
- ✅ Global validator runs **BEFORE** any installation steps
- ✅ Installation **ABORTS** if validator fails (fail-closed)
- ✅ Runs even in dry-run mode (mandatory)
- ✅ Exit code 1 if validator fails

**Implementation:**
```bash
VALIDATOR_PATH="$PROJECT_ROOT/core/global_validator/validate.py"

if [[ -f "$VALIDATOR_PATH" ]]; then
    log "Running global validator"
    if python3 "$VALIDATOR_PATH" 2>&1 | tee -a "${LOG_FILE:-/dev/stdout}"; then
        VALIDATOR_EXIT_CODE=${PIPESTATUS[0]}
        if [[ $VALIDATOR_EXIT_CODE -eq 0 ]]; then
            success "Global validator PASSED - installation can proceed"
        else
            error "Global validator FAILED (exit code: $VALIDATOR_EXIT_CODE) - installation aborted (fail-closed)"
        fi
    else
        error "Failed to execute global validator"
    fi
else
    error "Global validator not found: $VALIDATOR_PATH (fail-closed)"
fi
```

---

### ✅ 4. Manifest Generation Validation

**Behavior:**
- ✅ Manifest generation runs after global validator
- ✅ Installation **ABORTS** if manifest generation fails
- ✅ Manifest file **MUST** exist after generation
- ✅ Exit code 1 if manifest generation fails

**Implementation:**
```bash
if python3 -c "...manifest generator..." 2>&1 | tee -a "${LOG_FILE:-/dev/stdout}"; then
    MANIFEST_EXIT_CODE=${PIPESTATUS[0]}
    if [[ $MANIFEST_EXIT_CODE -eq 0 ]]; then
        if [[ -f "$MANIFEST_PATH" ]]; then
            success "Install manifest generated: $MANIFEST_PATH"
        else
            error "Install manifest generation reported success but file not found: $MANIFEST_PATH"
        fi
    else
        error "Install manifest generation failed with exit code: $MANIFEST_EXIT_CODE"
    fi
else
    error "Failed to execute manifest generator"
fi
```

---

### ✅ 5. Write Detection (Dry-Run Enforcement)

**Behavior:**
- ✅ All file write operations wrapped in dry-run checks
- ✅ Logs indicate "[DRY-RUN] Would execute: ..." for skipped operations
- ✅ No actual writes occur in dry-run mode (except manifest to project root)

**Operations Skipped in Dry-Run:**
- User/group creation
- Binary installation
- systemd unit installation
- Config file writes
- Service starts

---

## Test Results

### Dry-Run Execution

**Command:**
```bash
RANSOMEYE_DRY_RUN=1 ./install.sh
```

**Results:**
- ✅ Exit code: 0
- ✅ Global validator: PASSED
- ✅ Manifest generated: `/home/ransomeye/rebuild/install_manifest.json`
- ✅ Zero system changes made
- ✅ All operations logged

**Output:**
```
===========================================================================
DRY-RUN MODE ENABLED
===========================================================================
No files will be written, no systemd units installed, no services started.
Actions will be logged only.
===========================================================================

[DRY-RUN] Running Global Forensic Consistency Validator
✓ Global validator PASSED - installation can proceed

[DRY-RUN] Generating install manifest
✓ Install manifest generated: /home/ransomeye/rebuild/install_manifest.json
  Modules: 29
  Binaries: 4
  Systemd units: 2
  Agent exclusions: 0

[DRY-RUN] Installation simulation completed

===========================================================================
DRY-RUN COMPLETE
===========================================================================

Dry-run completed successfully. No system changes were made.

Install manifest: /home/ransomeye/rebuild/install_manifest.json

To perform actual installation (after reviewing manifest):
  sudo ./install.sh

✓ Dry-run completed successfully (exit code: 0)
```

---

## Files Modified

1. `/home/ransomeye/rebuild/install.sh`
   - Added DRY_RUN mode detection
   - Added global validator hook (Step 0)
   - Added manifest generation (Step 3)
   - Added dry-run checks for all write operations
   - Added dry-run completion message

2. `/home/ransomeye/rebuild/ransomeye_installer/manifest_generator.py`
   - Added `dry_run` parameter to constructor
   - Enhanced `generate_manifest()` to include:
     - `binaries` list
     - `systemd_units` list with source/target paths
     - `config_paths` list
     - `ports` dictionary
     - `database_migrations` list
     - `agent_exclusions` list
     - `dry_run` flag
   - Updated `write_manifest()` to write to project root in dry-run mode

---

## Stop Condition Met

✅ **Running `RANSOMEYE_DRY_RUN=1 ./install.sh` must:**
- ✅ Exit code 0
- ✅ Produce `install_manifest.json`
- ✅ Make zero system changes

---

## Design Principles Maintained

✅ **No auto-install** - Dry-run mode prevents all installations  
✅ **No assumptions** - All operations are explicit  
✅ **No interactive prompts** - EULA skipped in dry-run  
✅ **No silent fallback** - All failures abort with error messages  
✅ **Fail-closed** - Validator failures abort installation  
✅ **Deterministic** - Manifest generation is reproducible  

---

## Next Steps

Prompt #12 is complete. Ready for Prompt #13 (Local Install - Controlled).

**Pre-requisites for Prompt #13:**
- ✅ Dry-run mode works correctly
- ✅ Manifest generation works correctly
- ✅ Global validator hook works correctly
- ✅ All validation checks pass

---

## Status

✅ **PROMPT #12 COMPLETE** - Installer Dry-Run & Manifest Audit

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

