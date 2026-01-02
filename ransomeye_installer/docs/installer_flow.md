# RansomEye Installer Flow

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_installer/docs/installer_flow.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete installer flow documentation

---

## Overview

The RansomEye unified installer is the ONLY supported mechanism for installing RansomEye. No component may run outside this installer.

---

## Installation Flow

### Step 1: Prerequisites Validation

1. **OS Check**
   - Validates Ubuntu >=22.04 OR RHEL >=8
   - Fails-closed if unsupported OS

2. **Disk Check**
   - Validates minimum 10GB free space
   - Warns if usage >= 80%
   - Fails if usage >= 90%

3. **Swap Check**
   - Validates swap >= 16GB OR equal to RAM
   - Required for Core and DPI modules
   - Fails-closed if insufficient

4. **Clock Check**
   - Validates NTP synchronization
   - Warns if not synchronized (does not fail)

### Step 2: EULA Acceptance

1. **Display EULA**
   - Reads from `/home/ransomeye/rebuild/ransomeye_installer/eula/EULA.txt`
   - Displays full EULA text

2. **Get Acceptance**
   - Prompts user: "Do you accept the EULA? (yes/no)"
   - Fails-closed if not accepted
   - No silent install
   - No environment bypass

3. **Log Acceptance**
   - Acceptance logged in install state
   - State is signed cryptographically

### Step 3: Retention Configuration

1. **Prompt for Values**
   - Telemetry retention (months) [default: 6]
   - Forensic retention (days) [default: 10]
   - Disk max usage percent [default: 80]

2. **Apply Defaults**
   - If user skips (presses Enter), defaults are applied
   - Defaults are validated

3. **Validate Configuration**
   - All values validated against min/max ranges
   - Fails-closed if invalid

4. **Write Configuration**
   - Writes to `/home/ransomeye/rebuild/config/retention.txt`
   - Includes mandatory header

### Step 4: Cryptographic Identity Generation

1. **Generate Identity**
   - Creates RSA-4096 key pair
   - Generates unique identity hash
   - Stores in `/home/ransomeye/rebuild/ransomeye_installer/crypto/`

2. **Verify Identity**
   - Checks if identity already exists
   - Reuses existing identity if present

### Step 5: Systemd Units Creation

1. **Generate Units**
   - Creates service units for all core modules
   - Writes to `/home/ransomeye/rebuild/systemd/`
   - All units disabled by default

2. **Unit Dependencies**
   - All units require `install_state.json` to exist
   - All units require network.target
   - All units restart on failure

### Step 6: Installation State

1. **Create State**
   - Creates signed `install_state.json`
   - Includes: version, EULA acceptance, retention config, identity
   - State is cryptographically signed

2. **Validate State**
   - State must be valid for services to start
   - Tampered state prevents service startup

### Step 7: Completion

1. **Summary**
   - Displays installation summary
   - Provides next steps

2. **Services Not Started**
   - Services are NOT auto-started
   - User must manually enable/start services

---

## Fail-Closed Conditions

Installation ABORTS if:

- OS unsupported
- Disk insufficient (< 10GB free OR >= 90% used)
- Swap missing (where required)
- EULA not accepted
- Retention invalid
- Identity generation fails
- State save fails

---

## Service Startup Requirements

Services will NOT start unless:

1. `install_state.json` exists and is valid
2. State signature is valid
3. EULA was accepted
4. Retention is configured
5. Identity is generated

---

## Manual Service Management

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

## Upgrade Flow

Upgrade process:

1. Validate existing state
2. Run prerequisite checks
3. Display EULA (if changed)
4. Update retention (if needed)
5. Regenerate identity (if needed)
6. Update systemd units
7. Update state to 'UPGRADED'

---

## Last Updated

Phase 1 Implementation

