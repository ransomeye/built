# Path and File Name: /home/ransomeye/rebuild/ransomeye_dpi_probe/installer/requirements.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: OS, kernel, and privilege requirements for DPI Probe

# RansomEye DPI Probe - System Requirements

## Operating System

- **Required**: Linux (any modern distribution)
- **Recommended**: Ubuntu 20.04+, RHEL 8+, Debian 11+

## Kernel Requirements

- **Minimum**: Linux kernel 4.18+
- **Required Features**:
  - AF_XDP support (available in kernel 4.18+)
  - eBPF support (available in kernel 4.18+)
  - Network namespaces

## CPU Requirements

- **Minimum**: 4 CPU cores
- **Recommended**: 8+ CPU cores for high-throughput workloads
- **NUMA**: Multi-NUMA node systems supported

## Memory Requirements

- **RAM**: Minimum 8GB, recommended 16GB+
- **Swap**: **MANDATORY** - Minimum 16GB or equal to RAM (whichever is larger)
  - Swap is automatically created during installation if insufficient
  - Swap file location: `/swapfile_ransomeye_dpi`

## Disk Space

- **Installation**: 500MB for binaries and configuration
- **Runtime**: Additional space for logs and buffers
  - Buffer directory (default): `/var/lib/ransomeye/dpi_probe/buffer`
  - Configurable via `BUFFER_DIR` environment variable
  - Default buffer size: 1024MB (configurable via `MAX_BUFFER_SIZE_MB`)

## Network Requirements

- **Interface**: Network interface for packet capture
  - Configurable via `CAPTURE_IFACE` environment variable
  - Default: `eth0`
- **Bandwidth**: Designed for 10Gbps+ sustained throughput
- **Capabilities**: Requires CAP_NET_RAW capability for packet capture

## Privileges

- **Installation**: Root (sudo) privileges required
- **Runtime**: Service runs as root for packet capture capabilities
  - Service is hardened with systemd security options

## Software Dependencies

- **libpcap**: Required for packet capture
  - Debian/Ubuntu: `libpcap-dev`
  - RHEL/CentOS: `libpcap-devel`
- **systemd**: Required for service management
- **GPG**: Optional but recommended for signature verification

## Network Configuration

- **mTLS**: Requires client certificate and private key
  - Configurable via `PROBE_CERT_PATH` and `PROBE_KEY_PATH`
- **CA Certificate**: Required for Core API connection
  - Configurable via `CA_CERT_PATH`

## Performance Considerations

- **CPU Affinity**: Recommended to pin service to specific CPU cores
- **NUMA**: For multi-NUMA systems, allocate buffers on local NUMA node
- **Memory Locking**: Consider using `mlock()` for critical buffers (not currently implemented)

## Security Requirements

- **Binary Signing**: Binaries must be cryptographically signed
- **EULA Acceptance**: Required during installation (no bypass)
- **Isolated Execution**: Service runs with restricted capabilities via systemd

