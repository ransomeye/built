# DPI Probe Environment Variables Schema

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/config/env_schema.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete environment variable schema for DPI Probe configuration

---

## Required Environment Variables

None - all variables have defaults.

## Optional Environment Variables

### Core API Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CORE_API_URL` | String | `https://localhost:8443` | Core API endpoint URL |

### Network Capture Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CAPTURE_IFACE` | String | `eth0` | Network interface for packet capture |

### Buffer Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BUFFER_DIR` | String | `/var/lib/ransomeye/dpi_probe/buffer` | Directory for disk-based buffering |
| `MAX_BUFFER_SIZE_MB` | Integer | `1024` | Maximum buffer size in megabytes |
| `BACKPRESSURE_THRESHOLD` | Integer | `8192` | Backpressure threshold in bytes |

### Certificate Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PROBE_CERT_PATH` | String | `/etc/ransomeye/certs/probe.crt` | DPI Probe client certificate path |
| `PROBE_KEY_PATH` | String | `/etc/ransomeye/certs/probe.key` | DPI Probe private key path |
| `CA_CERT_PATH` | String | `/etc/ransomeye/certs/ca.crt` | CA certificate path for mTLS |

### Flow Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `FLOW_TIMEOUT_SECONDS` | Integer | `300` | Flow timeout in seconds (5 minutes) |

### Health Reporting Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `HEALTH_REPORT_INTERVAL_SECONDS` | Integer | `60` | Health report interval in seconds |

## Configuration Validation

All integer values must be:
- Positive (> 0)
- Within reasonable bounds:
  - `MAX_BUFFER_SIZE_MB`: 1 - 10000
  - `BACKPRESSURE_THRESHOLD`: 1024 - 1048576
  - `FLOW_TIMEOUT_SECONDS`: 60 - 3600
  - `HEALTH_REPORT_INTERVAL_SECONDS`: 10 - 3600

## Security Notes

- Certificate files must be readable by the DPI Probe process
- Private key files must have permissions 0600
- Buffer directory must be writable by the DPI Probe process
- All paths must be absolute (no relative paths)

## Example Configuration

```bash
export CORE_API_URL="https://core.example.com:8443"
export CAPTURE_IFACE="eth0"
export BUFFER_DIR="/var/lib/ransomeye/dpi_probe/buffer"
export MAX_BUFFER_SIZE_MB="2048"
export BACKPRESSURE_THRESHOLD="16384"
export PROBE_CERT_PATH="/etc/ransomeye/certs/probe.crt"
export PROBE_KEY_PATH="/etc/ransomeye/certs/probe.key"
export CA_CERT_PATH="/etc/ransomeye/certs/ca.crt"
export FLOW_TIMEOUT_SECONDS="600"
export HEALTH_REPORT_INTERVAL_SECONDS="120"
```
