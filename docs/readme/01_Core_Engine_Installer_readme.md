# Phase 1: Core Engine & Installer

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/01_Core_Engine_Installer_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 1 - Core Engine & Installer

---

## WHAT EXISTS

### Implementation Location
- **Python Installer:** `/home/ransomeye/rebuild/ransomeye_installer/`
- **Rust Operations:** `/home/ransomeye/rebuild/ransomeye_operations/` (if exists)
- **Root Installer:** `/home/ransomeye/rebuild/install.sh`
- **Root Uninstaller:** `/home/ransomeye/rebuild/uninstall.sh`

### Core Components

1. **Python Installer** (`ransomeye_installer/installer.py`)
   - Main orchestrator for installation
   - Validates prerequisites (OS, disk, swap, clock)
   - Enforces EULA acceptance (no bypass)
   - Configures retention policies
   - Generates cryptographic identities
   - Creates systemd service files
   - Writes install state

2. **State Manager** (`ransomeye_installer/state_manager.py`)
   - Tracks installation state
   - Records EULA acceptance
   - Records retention configuration
   - Records identity generation
   - State file: `ransomeye_installer/config/install_state.json`

3. **System Checks** (`ransomeye_installer/system/`)
   - OS check (`os_check.py`)
   - Disk check (`disk_check.py`)
   - Swap check (`swap_check.py`)
   - Clock check (`clock_check.py`)

4. **Retention Writer** (`ransomeye_installer/retention/retention_writer.py`)
   - Writes retention configuration
   - Validates retention limits (<= 7 years)
   - Output: `config/retention.txt`

5. **Identity Generator** (`ransomeye_installer/crypto/identity_generator.py`)
   - Generates Ed25519 keypairs
   - Creates trust store
   - Stores keys securely

6. **Runtime Deployer** (`ransomeye_installer/runtime/runtime_deployer.py`)
   - Deploys modules to `/opt/ransomeye` for production runtime
   - Creates canonical runtime layout (bin/, lib/, modules/, config/, logs/)
   - Sets proper ownership (ransomeye:ransomeye) and permissions
   - Generates launcher scripts in `/opt/ransomeye/bin`
   - Validates runtime layout after deployment

6. **Systemd Writer** (`ransomeye_installer/services/systemd_writer.py`)
   - Generates systemd service files
   - Validates module existence (no phantom modules)
   - Outputs to unified `/home/ransomeye/rebuild/systemd/`
   - All services run as `User=ransomeye` (rootless)
   - **SOURCE OF TRUTH:** `/home/ransomeye/rebuild/systemd/*.service` templates are canonical and runtime-clean
   - **RUNTIME PATHS:** All units reference `/opt/ransomeye` (not `/home/ransomeye/rebuild`)
   - **LAUNCHER SCRIPTS:** All services use `/opt/ransomeye/bin/<service-launcher>` executables
   - **NO HOME PATHS:** Templates contain zero references to `/home/ransomeye` or developer directories

7. **Module Resolver** (`ransomeye_installer/module_resolver.py`)
   - Resolves module names to directories
   - Detects phantom modules (fail-closed)
   - Validates module existence

8. **Manifest Generator** (`ransomeye_installer/manifest_generator.py`)
   - Generates installation manifest
   - Records installed components
   - Records versions

9. **Root Installer** (`install.sh`)
   - Root privilege check
   - EULA enforcement
   - Creates ransomeye user/group
   - Builds guardrails binary
   - Enforces guardrails
   - Builds core binaries
   - Calls Python installer
   - Post-install validation

10. **Post-Install Validator** (`post_install_validator.py`)
    - Validates installation consistency
    - Checks service definitions
    - Validates rootless runtime
    - Validates services disabled by default
    - Validates trust store
    - Validates identities
    - Validates retention config
    - Validates service-to-binary integrity
    - Validates module references exist
    - Invokes release gate

---

## WHAT DOES NOT EXIST

1. **No separate uninstaller module** - Uninstall handled by root `uninstall.sh`
2. **No database initialization** - Database setup handled by DB Core (Phase 10)
3. **No AI/ML model installation** - Models handled by Intelligence (Phase 3)
4. **No UI installation** - UI handled separately (Phase 11)
5. **No standalone agent installation** - Agents have separate installers

---

## DATABASE SCHEMAS

**NONE** - Phase 1 does not create database schemas. Database initialization is handled by Phase 10 (DB Core).

**Install State:**
- File: `ransomeye_installer/config/install_state.json`
- Format: JSON
- Fields: `state`, `timestamp`, `version`, `eula_accepted`, `retention_configured`, `identity_generated`

---

## RUNTIME SERVICES

**NONE** - Phase 1 is an installer, not a runtime service.

**Service Creation:**
- Phase 1 creates systemd service files in `/home/ransomeye/rebuild/systemd/`
- Services are created but **DISABLED by default**
- Services must be manually enabled: `sudo systemctl enable ransomeye-*`

**Systemd Source Canonicalization (v1):**
- **SOURCE OF TRUTH:** `/home/ransomeye/rebuild/systemd/*.service` templates are runtime-clean
- **CANONICAL RUNTIME ROOT:** `/opt/ransomeye` (all WorkingDirectory, ExecStart, ConditionPathExists)
- **CANONICAL STATE ROOT:** `/var/lib/ransomeye` (state and runtime data)
- **CANONICAL LOG ROOT:** `/var/log/ransomeye` (all logging output)
- **LAUNCHER PATHS:** All ExecStart point to `/opt/ransomeye/bin/<service-launcher>`
- **NO HOME PATHS:** Zero references to `/home/ransomeye` or `/home/ransomeye/rebuild` in runtime configuration
- **NO PYTHON -M:** All services use launcher scripts, not Python module invocation
- **INSTALL STATE:** ConditionPathExists checks `/opt/ransomeye/config/install_state.json`
- **VALIDATION:** Pre-start validation ensures `/opt/ransomeye/bin/<service-launcher>` exists
- All units are installable on clean hosts without source tree presence

**Service Files Created:**
- `ransomeye-ingestion.service`
- `ransomeye-correlation.service`
- `ransomeye-policy.service`
- `ransomeye-enforcement.service`
- `ransomeye-intelligence.service`
- `ransomeye-reporting.service`
- `ransomeye-network-scanner.service`
- `ransomeye-playbook-engine.service`
- `ransomeye-posture-engine.service`
- `ransomeye-feed-fetcher.service` (timer)
- `ransomeye-feed-retraining.service` (timer)

**All services configured:**
- `User=ransomeye`
- `Group=ransomeye`
- `Restart=always`
- `WantedBy=multi-user.target`
- `StandardOutput=journal`
- `StandardError=journal`

---

## GUARDRAILS ALIGNMENT

Phase 1 enforces guardrails:

1. **EULA Enforcement** - No bypass, must accept
2. **Retention Limits** - <= 7 years enforced
3. **Rootless Runtime** - All services run as `ransomeye` user
4. **Phantom Module Detection** - Fail-closed if phantom modules referenced
5. **Service-to-Binary Integrity** - All services must reference valid binaries
6. **Unified Systemd** - All services in `/home/ransomeye/rebuild/systemd/`

**Fail-Closed Behavior:**
- EULA not accepted → Installation aborted
- Phantom modules detected → Installation aborted
- Invalid service binaries → Installation aborted
- Guardrails enforcement fails → Installation aborted

---

## INSTALLER BEHAVIOR

**Installation Flow:**

1. **Root Check** - Must run as root
2. **EULA Display** - Shows EULA, requires acceptance
3. **User/Group Creation** - Creates `ransomeye` user and group
4. **Guardrails Enforcement** - Builds and runs guardrails (fail-closed)
5. **Binary Building** - Builds Rust binaries (operations, playbooks, network scanner)
6. **Python Installer** - Calls `ransomeye_installer.installer`
7. **Prerequisites Check** - OS, disk, swap, clock
8. **Retention Configuration** - Writes retention config
9. **Identity Generation** - Generates Ed25519 keys
10. **Systemd Creation** - Creates service files
11. **Post-Install Validation** - Validates installation
12. **Release Gate** - Final validation

**Uninstallation:**
- Handled by `uninstall.sh`
- Removes services
- Removes binaries
- Removes user/group (optional)
- Preserves data (logs, evidence)

---

## SYSTEMD INTEGRATION

**Unified Systemd Directory:**
- All service files in `/home/ransomeye/rebuild/systemd/`
- No per-module systemd directories (except standalone agents)

**Service Configuration:**
- All services run as `User=ransomeye` (rootless)
- All services have `Restart=always`
- All services disabled by default
- All services use `StandardOutput=journal` and `StandardError=journal`

**Service Dependencies:**
- Services depend on `network.target`
- Services depend on previous services in pipeline
- Services check for `install_state.json` existence

**Runtime vs Development Separation:**
- **Development:** `/home/ransomeye/rebuild/` (source code, build artifacts)
- **Runtime:** `/opt/ransomeye/` (deployed modules, launchers, production files)
- Services MUST run from `/opt/ransomeye`, NOT from `/home/ransomeye/rebuild`
- Installer deploys modules to `/opt/ransomeye/modules/` during installation
- Launcher scripts created in `/opt/ransomeye/bin/` for each service
- Runtime layout:
  - `/opt/ransomeye/bin/` - Service launcher scripts (550 permissions)
  - `/opt/ransomeye/modules/` - Python modules (750 dirs, 640 files)
  - `/opt/ransomeye/config/` - Runtime configuration
  - `/opt/ransomeye/logs/` - Runtime logs (symlink to /var/log/ransomeye)
- All runtime files owned by `ransomeye:ransomeye`
- systemd `WorkingDirectory` set to `/opt/ransomeye`
- systemd `ExecStart` points to `/opt/ransomeye/bin/<service-launcher>`
- Pre-start validation ensures `/opt/ransomeye` exists and is accessible

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 1 does not train or install AI/ML models.

**Model Installation:**
- Models installed by Phase 3 (Intelligence)
- Baseline Intelligence Pack validated at startup
- Models must be signed and verified

---

## COPILOT REALITY

**NONE** - Phase 1 does not provide copilot functionality.

---

## UI REALITY

**NONE** - Phase 1 does not install UI.

**UI Installation:**
- UI handled separately (Phase 11)
- UI may be installed after core installation

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **EULA Not Accepted** → Installation aborted
2. **Guardrails Fail** → Installation aborted
3. **Phantom Modules Detected** → Installation aborted
4. **Invalid Service Binaries** → Installation aborted
5. **Prerequisites Fail** → Installation aborted
6. **Post-Install Validation Fails** → Installation marked failed
7. **Release Gate Blocks** → Installation cannot proceed

**No Bypass Mechanisms:**
- No `--skip-eula` flag
- No `--skip-guardrails` flag
- No `--force` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 1 is fully implemented and production-ready:

✅ **Complete Implementation**
- Python installer fully functional
- Root installer wrapper complete
- All system checks implemented
- Retention configuration working
- Identity generation working
- Systemd service creation working
- Post-install validation complete

✅ **Guardrails Alignment**
- EULA enforcement (no bypass)
- Retention limits enforced
- Rootless runtime enforced
- Phantom module detection
- Service-to-binary integrity

✅ **Fail-Closed Behavior**
- All critical checks fail-closed
- No bypass mechanisms
- Release gate integration

**Limitations:**
- No database initialization (handled by Phase 10)
- No model installation (handled by Phase 3)
- No UI installation (handled by Phase 11)

**Recommendation:** Deploy as-is. Phase 1 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
