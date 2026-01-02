# Path and File Name: /home/ransomeye/rebuild/ransomeye_linux_agent/installer/requirements.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: OS, kernel, and privilege requirements for Linux Agent

# RansomEye Linux Agent - System Requirements

## Operating System

- **Required**: Linux (any modern distribution)
- **Recommended**: Ubuntu 20.04+, RHEL 8+, Debian 11+

## Kernel Requirements

- **Minimum**: Linux kernel 3.10+ (for inotify support)
- **No special kernel features required** (runs in user space)

## Memory Requirements

- **RAM**: Minimum 512MB, recommended 1GB+
- **Swap**: **FORBIDDEN** - Linux Agent MUST NOT use swap
  - Installation will FAIL if swap is detected
  - All swap must be disabled before installation
  - Agent must run entirely in RAM

## Disk Space

- **Installation**: 100MB for binaries and configuration
- **Runtime**: Additional space for logs and buffers
  - Buffer directory (default): `/var/lib/ransomeye/linux_agent/buffer`
  - Configurable via `BUFFER_DIR` environment variable
  - Default buffer size: 512MB (configurable via `MAX_BUFFER_SIZE_MB`)

## Privileges

- **Installation**: Root (sudo) privileges required
- **Runtime**: Service runs as unprivileged user (`ransomeye`)
  - Binary starts with SetUID for privilege downgrade
  - Service automatically drops privileges after startup
  - Runs with minimal capabilities

## Software Dependencies

- **systemd**: Required for service management
- **inotify**: Built-in kernel feature (required for file monitoring)
- **GPG**: Optional but recommended for signature verification

## Network Configuration

- **mTLS**: Requires client certificate and private key
  - Configurable via `AGENT_CERT_PATH` and `AGENT_KEY_PATH`
- **CA Certificate**: Required for Core API connection
  - Configurable via `CA_CERT_PATH`

## Security Requirements

- **Binary Signing**: Binaries must be cryptographically signed
- **EULA Acceptance**: Required during installation (no bypass)
- **No Kernel Drivers**: Runs entirely in user space
- **Minimal Privileges**: Drops to unprivileged user after startup

## Performance Considerations

- **Memory Locking**: All operations run in RAM (no swap dependency)
- **Resource Limits**: Configured via systemd for memory protection
- **CPU Affinity**: Optional - can be configured in service file

## Rollback Safety

- **Clean Uninstall**: Uninstaller preserves logs (configurable)
- **State Preservation**: Configuration and logs can be preserved
- **Service State**: Service stops cleanly without data loss

