# Linux Agent Environment Variables Schema

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_linux_agent/config/env_schema.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete environment variable schema for Linux Agent configuration

---

## Required Environment Variables

None - all variables have defaults.

## Optional Environment Variables

### Core API Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CORE_API_URL` | String | `https://localhost:8443` | Core API endpoint URL |

### Buffer Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `BUFFER_DIR` | String | `/var/lib/ransomeye/linux_agent/buffer` | Directory for disk-based buffering |
| `MAX_BUFFER_SIZE_MB` | Integer | `512` | Maximum buffer size in megabytes |
| `BACKPRESSURE_THRESHOLD` | Integer | `4096` | Backpressure threshold in bytes |

### Certificate Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `AGENT_CERT_PATH` | String | `/etc/ransomeye/certs/agent.crt` | Linux Agent client certificate path |
| `AGENT_KEY_PATH` | String | `/etc/ransomeye/certs/agent.key` | Linux Agent private key path |
| `CA_CERT_PATH` | String | `/etc/ransomeye/certs/ca.crt` | CA certificate path for mTLS |

### Telemetry Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `TELEMETRY_INTERVAL_SECONDS` | Integer | `1` | Telemetry collection interval in seconds |
| `MONITOR_PATHS` | String | `/,/home,/var,/tmp` | Comma-separated paths to monitor for file activity |
| `AUTH_LOG_PATHS` | String | `/var/log/auth.log,/var/log/secure` | Comma-separated paths to auth log files |

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
  - `TELEMETRY_INTERVAL_SECONDS`: 1 - 3600
  - `HEALTH_REPORT_INTERVAL_SECONDS`: 10 - 3600

## Security Notes

- Certificate files must be readable by the Linux Agent process
- Private key files must have permissions 0600
- Buffer directory must be writable by the Linux Agent process
- All paths must be absolute (no relative paths)
- Agent should run with minimal privileges (non-root when possible)

## Example Configuration

```bash
export CORE_API_URL="https://core.example.com:8443"
export BUFFER_DIR="/var/lib/ransomeye/linux_agent/buffer"
export MAX_BUFFER_SIZE_MB="1024"
export BACKPRESSURE_THRESHOLD="8192"
export AGENT_CERT_PATH="/etc/ransomeye/certs/agent.crt"
export AGENT_KEY_PATH="/etc/ransomeye/certs/agent.key"
export CA_CERT_PATH="/etc/ransomeye/certs/ca.crt"
export TELEMETRY_INTERVAL_SECONDS="5"
export MONITOR_PATHS="/,/home,/var"
export AUTH_LOG_PATHS="/var/log/auth.log"
export HEALTH_REPORT_INTERVAL_SECONDS="120"
```