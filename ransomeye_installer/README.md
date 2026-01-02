# RansomEye Unified Installer

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_installer/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 1 installer documentation and usage guide

---

## Overview

The RansomEye unified installer is the **ONLY supported mechanism** for installing RansomEye. No component may run outside this installer.

---

## Installation

### Prerequisites

- Ubuntu >=22.04 OR RHEL >=8
- Python 3.10+
- Minimum 10GB free disk space
- Swap >= 16GB OR equal to RAM (for Core/DPI)
- Root/sudo access

### Run Installer

```bash
cd /home/ransomeye/rebuild
python3 -m ransomeye_installer.installer
```

---

## Installation Flow

1. **Validate Prerequisites**
   - OS compatibility
   - Disk space
   - Swap space
   - Clock synchronization

2. **EULA Acceptance**
   - Display EULA
   - Require explicit acceptance
   - Fail-closed if not accepted

3. **Retention Configuration**
   - Prompt for values
   - Apply defaults if skipped
   - Validate all inputs

4. **Generate Identity**
   - Create RSA-4096 key pair
   - Generate unique identity hash

5. **Create Systemd Units**
   - Generate service units for all core modules
   - All units disabled by default

6. **Save State**
   - Create signed `install_state.json`
   - State required for services to start

---

## Components

### Core Modules

- `installer.py` - Main orchestrator
- `state_manager.py` - Install state management

### System Checks

- `system/os_check.py` - OS validation
- `system/disk_check.py` - Disk space validation
- `system/swap_check.py` - Swap space validation
- `system/clock_check.py` - Clock synchronization check

### Retention

- `retention/retention_writer.py` - Writes retention.txt
- `retention/retention_validator.py` - Validates retention config

### Crypto

- `crypto/identity_generator.py` - Generates installation identity
- `crypto/keystore.py` - Secure key storage

### Services

- `services/systemd_writer.py` - Generates systemd units
- `services/lifecycle.py` - Service lifecycle management

---

## Fail-Closed Conditions

Installation ABORTS if:

- OS unsupported
- Disk insufficient
- Swap missing (where required)
- EULA not accepted
- Retention invalid
- Identity generation fails
- State save fails

---

## Service Management

After installation:

```bash
# Install systemd units
sudo cp /home/ransomeye/rebuild/systemd/*.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable services (optional)
sudo systemctl enable ransomeye-*

# Start services
sudo systemctl start ransomeye-*
```

---

## Service Startup Requirements

Services will NOT start unless:

1. `install_state.json` exists and is valid
2. State signature is valid
3. EULA was accepted
4. Retention is configured
5. Identity is generated

---

## Documentation

- `docs/installer_flow.md` - Complete installation flow
- `docs/failure_modes.md` - Failure modes and recovery
- `docs/upgrade_policy.md` - Upgrade procedures

---

## Tests

Run tests:

```bash
python3 -m unittest ransomeye_installer.tests.eula_enforcement_test
python3 -m unittest ransomeye_installer.tests.retention_default_test
python3 -m unittest ransomeye_installer.tests.fail_closed_test
```

---

## Last Updated

Phase 1 Implementation - Complete

