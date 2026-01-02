# RansomEye Operations

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_operations/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 11 - Unified Installer, Uninstaller & Operations module

---

## Overview

Phase 11 provides the **only supported lifecycle** for RansomEye:
- Install
- Configure
- Start / Stop
- Upgrade
- Uninstall

No manual startup. No partial installs.

---

## Features

- **Unified Installer**: Complete installation flow with preflight, EULA, retention, crypto, and state
- **Uninstaller**: Clean uninstallation with evidence preservation options
- **Lifecycle Management**: Start, stop, restart, and status operations
- **Systemd Integration**: Systemd service units for all core modules
- **Fail-Closed Behavior**: All operations fail-closed on violations
- **Signed State**: Install state is cryptographically signed and verified
- **EULA Enforcement**: EULA acceptance is mandatory

---

## Installation

### Prerequisites

- Linux with systemd
- Minimum 10 GB disk space
- Root or sudo privileges
- Time synchronization

### Install Command

```bash
ransomeye_operations install --accept-eula --engine-version 1.0.0
```

**Requirements:**
- `--accept-eula`: EULA acceptance is mandatory
- `--engine-version`: Engine version (default: 1.0.0)

---

## Service Lifecycle

### Start Services

```bash
ransomeye_operations start
ransomeye_operations start ransomeye-core
```

### Stop Services

```bash
ransomeye_operations stop
ransomeye_operations stop ransomeye-core
```

### Restart Services

```bash
ransomeye_operations restart
ransomeye_operations restart ransomeye-core
```

### Check Status

```bash
ransomeye_operations status
ransomeye_operations status ransomeye-core
```

---

## Uninstallation

```bash
ransomeye_operations uninstall --confirm
ransomeye_operations uninstall --confirm --remove-evidence --secure-delete
```

**Options:**
- `--confirm`: Confirmation is mandatory
- `--remove-evidence`: Remove evidence (default: preserve)
- `--secure-delete`: Use secure deletion (3-pass overwrite)

---

## Architecture

### Installer Components

- **PreflightChecker**: Validates OS, disk, time, permissions
- **RetentionConfigurator**: Configures retention policies
- **CryptoIdentityManager**: Generates Ed25519 key pairs
- **InstallStateManager**: Creates signed install state
- **Installer**: Orchestrates installation flow

### Uninstaller Components

- **UninstallVerifier**: Verifies install state
- **CleanupManager**: Removes services, configs, evidence
- **Uninstaller**: Orchestrates uninstallation flow

### Lifecycle Components

- **ServiceStarter**: Starts services in dependency order
- **ServiceStopper**: Stops services in reverse order
- **ServiceRestarter**: Restarts services
- **ServiceStatusChecker**: Checks service status

---

## Systemd Services

All core modules have systemd service units:
- `ransomeye-core.service`
- `ransomeye-ingestion.service`
- `ransomeye-correlation.service`
- `ransomeye-policy.service`
- `ransomeye-enforcement.service`
- `ransomeye-intelligence.service`
- `ransomeye-reporting.service`

All services are **DISABLED by default** and require explicit enablement.

---

## Fail-Closed Behavior

Operations **fail-closed** on:
- EULA not accepted
- Install state invalid or tampered
- Retention configuration invalid
- Identity missing or invalid
- Service operation failures

---

## Documentation

- `docs/operations_guide.md`: Complete operations guide
- `docs/upgrade_procedure.md`: Upgrade procedures
- `docs/uninstall_procedure.md`: Uninstall procedures
- `docs/failure_modes.md`: Failure modes and error handling

---

## Testing

Comprehensive tests validate:
- EULA enforcement
- Install state tamper detection
- Clean uninstallation
- Lifecycle control

Run tests:
```bash
cargo test
```

---

## Security

- **Signed Install State**: All install states are cryptographically signed
- **Immutable State**: Install state cannot be modified without detection
- **Secure Deletion**: 3-pass overwrite for sensitive data
- **Service Hardening**: Systemd security hardening enabled

---

## License

Copyright © RansomEye.Tech  
Support: Gagan@RansomEye.Tech

