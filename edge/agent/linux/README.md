# RansomEye Linux Agent

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_linux_agent/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Stand-alone Linux Agent sensor for host telemetry collection

## Overview

The Linux Agent is a **stand-alone, untrusted sensor** that collects host telemetry. It never enforces policy, never runs AI, and never stores long-term state.

## Responsibilities

- Process telemetry
- File activity telemetry
- Auth activity telemetry
- Controlled response hooks (NOT enforcement)

## Hard Rules

- No kernel patching
- No persistent daemons beyond agent
- No swap dependency
- Minimal privileges
- Signed telemetry only

## Components

- **Telemetry Collector**: Collects process, file, and auth events
- **Event Signer**: Signs all telemetry with RSA-4096-PSS-SHA256
- **Transport Client**: mTLS client for sending signed events to Core
- **Backpressure Handler**: Manages bounded buffers and backpressure signals
- **Health Reporter**: Reports sensor health status

## Configuration

All configuration via environment variables:

- `CORE_API_URL`: Core API endpoint (default: `https://localhost:8443`)
- `BUFFER_DIR`: Buffer directory (default: `/var/lib/ransomeye/linux_agent/buffer`)
- `AGENT_CERT_PATH`: Client certificate path
- `AGENT_KEY_PATH`: Client private key path
- `CA_CERT_PATH`: CA certificate path
- `MAX_BUFFER_SIZE_MB`: Maximum buffer size in MB (default: 512)
- `BACKPRESSURE_THRESHOLD`: Backpressure threshold (default: 4096)
- `TELEMETRY_INTERVAL_SECONDS`: Telemetry collection interval (default: 1)
- `MONITOR_PATHS`: Comma-separated paths to monitor (default: `/,/home,/var,/tmp`)

## Communication

- mTLS authentication with per-instance certificates
- Event signing with RSA-4096-PSS-SHA256
- Replay protection via nonce
- Backpressure handling

## Testing

Run integration tests:

```bash
cd ransomeye_linux_agent
cargo test
```

Tests cover:
- Identity spoofing prevention
- Event signing verification
- Backpressure handling
- Core unavailability tolerance
- Resource exhaustion handling

## Installation

### Prerequisites

- Linux (any modern distribution)
- Root/sudo privileges
- NO SWAP enabled (installation will FAIL if swap is detected)
- Minimum 512MB RAM
- systemd for service management

### Install

1. **Disable all swap:**
```bash
# Disable all swap
sudo swapoff -a

# Remove swap entries from /etc/fstab (optional)
sudo sed -i '/ swap /d' /etc/fstab
```

2. Build the binary:
```bash
cd ransomeye_linux_agent
cargo build --release
```

3. Run the installer:
```bash
sudo ./installer/install.sh
```

The installer will:
- Enforce EULA acceptance (mandatory, no bypass)
- Verify binary signatures
- **FAIL if swap is detected** (NO-SWAP requirement)
- Validate NO-SWAP requirement
- Create unprivileged user (`ransomeye`)
- Install binary with SetUID for privilege downgrade
- Install systemd service
- Create signed install receipt

4. Verify installation:
```bash
./installer/verify.sh
```

### Service Management

**Start service:**
```bash
sudo systemctl start ransomeye-linux-agent
```

**Stop service:**
```bash
sudo systemctl stop ransomeye-linux-agent
```

**Enable auto-start:**
```bash
sudo systemctl enable ransomeye-linux-agent
```

**View logs:**
```bash
sudo journalctl -u ransomeye-linux-agent -f
```

**Check service status:**
```bash
sudo systemctl status ransomeye-linux-agent
```

The service automatically drops privileges from root to `ransomeye` user after startup.

See `installer/lifecycle.md` for complete lifecycle management documentation.

### Uninstallation

```bash
sudo ./installer/uninstall.sh
```

The uninstaller will:
- Stop and disable the service
- Remove binaries and service files
- Optionally preserve logs and configuration
- Optionally remove user and group

## Requirements

See `installer/requirements.md` for detailed system requirements, including:
- OS and kernel requirements
- Memory requirements (NO SWAP allowed)
- Privilege model (privilege downgrade)
- Security requirements

## Build

```bash
cargo build --release
```

## Run (Manual)

For manual execution (not recommended for production):

```bash
./target/release/ransomeye_linux_agent
```

**Note:** Production deployments should use the systemd service installed via the installer.

