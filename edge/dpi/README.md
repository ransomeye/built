# RansomEye DPI Probe

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Stand-alone DPI Probe sensor for high-throughput network packet inspection

## Overview

The DPI Probe is a **stand-alone, untrusted sensor** that performs passive network inspection. It never enforces policy, never runs AI, and never stores long-term state.

## Responsibilities

- Passive network inspection
- High-throughput packet processing (10Gbps+)
- Feature extraction only
- No decisions, no blocking

## Hard Rules

- Zero packet modification
- Zero enforcement
- Zero AI
- Bounded buffers
- Backpressure support

## Components

- **Capture Engine**: High-throughput packet capture using libpcap (passive only)
- **Flow Assembler**: Tracks network flows and assembles packets
- **Feature Extractor**: Extracts flow features (NO AI classification, metadata only)
- **Event Signer**: Signs all telemetry with RSA-4096-PSS-SHA256
- **Backpressure Handler**: Manages bounded buffers and backpressure signals
- **Disk Buffer**: Persistent buffering when Core unavailable (fail-open)
- **Output Formatter**: Telemetry serialization and validation
- **Health Reporter**: Reports sensor health status
- **Transport Client**: mTLS client for sending signed events to Core
- **Security Modules**: Identity, signature verification, replay protection, trust chain

## Configuration

All configuration via environment variables:

- `CORE_API_URL`: Core API endpoint (default: `https://localhost:8443`)
- `CAPTURE_IFACE`: Network interface to capture (default: `eth0`)
- `BUFFER_DIR`: Buffer directory (default: `/var/lib/ransomeye/dpi_probe/buffer`)
- `PROBE_CERT_PATH`: Client certificate path
- `PROBE_KEY_PATH`: Client private key path
- `CA_CERT_PATH`: CA certificate path
- `MAX_BUFFER_SIZE_MB`: Maximum buffer size in MB (default: 1024)
- `BACKPRESSURE_THRESHOLD`: Backpressure threshold (default: 8192)
- `FLOW_TIMEOUT_SECONDS`: Flow timeout in seconds (default: 300)

## Communication

- mTLS authentication with per-instance certificates
- Event signing with RSA-4096-PSS-SHA256
- Replay protection via nonce
- Backpressure handling

## Testing

Run integration tests:

```bash
cd ransomeye_dpi_probe
cargo test
```

Tests cover:
- **Identity spoofing prevention**: Rejects spoofed probe identity
- **Event signing verification**: Validates all events are signed
- **Backpressure handling**: Correctly handles Core backpressure signals
- **Core unavailability tolerance**: Buffers to disk when Core unavailable
- **Resource exhaustion handling**: Graceful degradation under resource pressure

## Architecture

See detailed documentation in `docs/`:
- `dpi_architecture.md`: Overall architecture and design
- `performance_model.md`: Performance characteristics and scalability
- `backpressure_behavior.md`: Backpressure handling details
- `security_model.md`: Security architecture and threat model
- `failure_modes.md`: Failure handling and recovery

## Protocol

Event schema and compatibility documentation in `protocol/`:
- `dpi_event_schema.json`: JSON schema for telemetry events
- `compatibility.md`: Protocol versioning and compatibility

## Security

Security modules in `security/`:
- Identity management and verification
- Cryptographic signature verification
- Replay attack protection
- Trust chain validation

## Installation

### Prerequisites

- Linux kernel 4.18+ (for AF_XDP support)
- Root/sudo privileges
- libpcap library (`apt-get install libpcap-dev` or `yum install libpcap-devel`)
- Minimum 4 CPU cores
- Swap: Minimum 16GB or equal to RAM (whichever is larger) - automatically created during installation

### Install

1. Build the binary:
```bash
cd ransomeye_dpi_probe
cargo build --release
```

2. Run the installer:
```bash
sudo ./installer/install.sh
```

The installer will:
- Enforce EULA acceptance (mandatory, no bypass)
- Verify binary signatures
- Validate environment constraints (OS, CPU, kernel features)
- Check/create swap (16GB or RAM, whichever is larger)
- Install binary to `/opt/ransomeye/dpi_probe/`
- Install systemd service
- Create signed install receipt

3. Verify installation:
```bash
./installer/verify.sh
```

### Service Management

**Start service:**
```bash
sudo systemctl start ransomeye-dpi-probe
```

**Stop service:**
```bash
sudo systemctl stop ransomeye-dpi-probe
```

**Enable auto-start:**
```bash
sudo systemctl enable ransomeye-dpi-probe
```

**View logs:**
```bash
sudo journalctl -u ransomeye-dpi-probe -f
```

See `installer/lifecycle.md` for complete lifecycle management documentation.

### Uninstallation

```bash
sudo ./installer/uninstall.sh
```

The uninstaller will:
- Stop and disable the service
- Remove binaries and service files
- Optionally preserve logs and configuration
- Optionally remove swap file (if created during installation)

## Requirements

See `installer/requirements.md` for detailed system requirements, including:
- OS and kernel requirements
- CPU and memory requirements
- Swap requirements (MANDATORY)
- Network and privilege requirements

## Build

```bash
cargo build --release
```

## Run (Manual)

For manual execution (not recommended for production):

```bash
./target/release/ransomeye_dpi_probe
```

**Note:** Production deployments should use the systemd service installed via the installer.

