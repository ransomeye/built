# Operations Guide

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_operations/docs/operations_guide.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete operations guide for RansomEye installation, configuration, and lifecycle management

---

## Overview

RansomEye Operations provides the **only supported lifecycle** for RansomEye:
- Install
- Configure
- Start / Stop
- Upgrade
- Uninstall

No manual startup. No partial installs.

---

## Installation

### Prerequisites

- Linux operating system with systemd
- Minimum 10 GB disk space
- Root or sudo privileges
- Time synchronization (NTP/systemd-timesyncd)

### Installation Flow

1. **Preflight Checks**: Validates OS, disk space, time sync, permissions
2. **EULA Acceptance**: Requires explicit EULA acceptance
3. **Retention Configuration**: Configures data retention policies
4. **Cryptographic Identity**: Generates Ed25519 key pair
5. **Install State**: Creates signed, immutable install state
6. **Systemd Units**: Generates systemd service units (DISABLED by default)
7. **Summary**: Prints installation summary

### Installation Command

```bash
ransomeye_operations install --accept-eula --engine-version 1.0.0
```

**Requirements:**
- `--accept-eula`: EULA acceptance is mandatory
- `--engine-version`: Engine version (default: 1.0.0)

---

## Configuration

### Retention Policy

Retention policy is configured during installation with defaults:
- **Telemetry Retention**: 6 months (maximum: 84 months / 7 years)
- **Forensic Retention**: 10 days (maximum: 3650 days / 10 years)
- **Disk Usage Threshold**: 80% (minimum: 50%, maximum: 100%)

Configuration is stored in `/home/ransomeye/rebuild/config/retention.txt`.

---

## Service Lifecycle

### Starting Services

Services **will NOT start** unless:
1. EULA was accepted
2. Install state is valid
3. Retention is configured
4. Identity is generated

**Start all services:**
```bash
ransomeye_operations start
```

**Start specific service:**
```bash
ransomeye_operations start ransomeye-core
```

### Stopping Services

**Stop all services:**
```bash
ransomeye_operations stop
```

**Stop specific service:**
```bash
ransomeye_operations stop ransomeye-core
```

### Restarting Services

**Restart all services:**
```bash
ransomeye_operations restart
```

**Restart specific service:**
```bash
ransomeye_operations restart ransomeye-core
```

### Checking Status

**Check all services:**
```bash
ransomeye_operations status
```

**Check specific service:**
```bash
ransomeye_operations status ransomeye-core
```

---

## Service Dependencies

Services start in dependency order:

1. `ransomeye-core` (no dependencies)
2. `ransomeye-ingestion` (depends on core)
3. `ransomeye-correlation` (depends on ingestion)
4. `ransomeye-policy` (depends on correlation)
5. `ransomeye-enforcement` (depends on policy)
6. `ransomeye-intelligence` (depends on correlation)
7. `ransomeye-reporting` (depends on enforcement)

Services stop in reverse dependency order.

---

## Systemd Integration

### Installing Systemd Units

After installation, systemd units must be installed:

```bash
sudo cp /home/ransomeye/rebuild/systemd/*.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### Enabling Services

Services are **DISABLED by default**. To enable:

```bash
sudo systemctl enable ransomeye-*
```

### Starting Services

```bash
sudo systemctl start ransomeye-*
```

---

## Uninstallation

### Uninstallation Flow

1. **Verification**: Verifies install state is valid
2. **Service Stop**: Stops all services
3. **Cleanup**: Removes services, configs, and optionally evidence

### Uninstallation Command

```bash
ransomeye_operations uninstall --confirm
```

**Options:**
- `--confirm`: Confirmation is mandatory
- `--remove-evidence`: Remove evidence (default: preserve)
- `--secure-delete`: Use secure deletion (3-pass overwrite)

**Example with evidence removal:**
```bash
ransomeye_operations uninstall --confirm --remove-evidence --secure-delete
```

---

## Upgrade

Upgrade procedures:
1. Validate compatibility
2. Preserve evidence and configs
3. Rotate keys if required
4. Signed upgrade state
5. Rollback on failure

See `upgrade_procedure.md` for detailed upgrade procedures.

---

## Fail-Closed Behavior

Operations **fail-closed** on:
- EULA not accepted
- Install state invalid or tampered
- Retention configuration invalid
- Identity missing or invalid
- Service operation failures

All failures are logged and auditable.

---

## Audit Trail

All operations are logged:
- Installation: Install state with signature
- Uninstallation: Cleanup log with timestamps
- Service operations: Systemd journal entries

---

## Security

- **Signed Install State**: All install states are cryptographically signed
- **Immutable State**: Install state cannot be modified without detection
- **Secure Deletion**: 3-pass overwrite for sensitive data
- **Service Hardening**: Systemd security hardening enabled

---

## Troubleshooting

### Installation Fails

- Check preflight requirements (OS, disk, time, permissions)
- Verify EULA acceptance
- Check logs: `/home/ransomeye/rebuild/logs/`

### Services Won't Start

- Verify install state is valid: `ransomeye_operations status`
- Check systemd logs: `journalctl -u ransomeye-*`
- Verify install state file exists and is valid

### Uninstallation Fails

- Verify install state is valid
- Check service status
- Review cleanup logs

---

## Support

Contact: Gagan@RansomEye.Tech  
Support: Gagan@RansomEye.Tech

