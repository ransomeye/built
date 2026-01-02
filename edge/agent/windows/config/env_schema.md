# Windows Agent Environment Variables Schema

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_windows_agent/config/env_schema.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete environment variable schema for Windows Agent configuration

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
| `BUFFER_DIR` | String | `C:\ProgramData\RansomEye\WindowsAgent\buffer` | Directory for disk-based buffering |
| `MAX_BUFFER_SIZE_MB` | Integer | `512` | Maximum buffer size in megabytes |
| `BACKPRESSURE_THRESHOLD` | Integer | `4096` | Backpressure threshold in bytes |

### Certificate Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `AGENT_CERT_PATH` | String | `C:\ProgramData\RansomEye\certs\agent.crt` | Windows Agent client certificate path |
| `AGENT_KEY_PATH` | String | `C:\ProgramData\RansomEye\certs\agent.key` | Windows Agent private key path |
| `CA_CERT_PATH` | String | `C:\ProgramData\RansomEye\certs\ca.crt` | CA certificate path for mTLS |

### Telemetry Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `TELEMETRY_INTERVAL_SECONDS` | Integer | `1` | Telemetry collection interval in seconds |
| `MONITOR_PATHS` | String | `C:\,C:\Users` | Comma-separated paths to monitor for file activity |
| `MONITOR_REGISTRY_KEYS` | String | `HKCU\Software,HKLM\Software` | Comma-separated registry keys to monitor |

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

- Certificate files must be readable by the Windows Agent process
- Private key files must have appropriate ACLs (restricted access)
- Buffer directory must be writable by the Windows Agent process
- All paths must be absolute (Windows paths)
- Agent should run with minimal privileges (non-Administrator when possible)

## Example Configuration

```powershell
$env:CORE_API_URL = "https://core.example.com:8443"
$env:BUFFER_DIR = "C:\ProgramData\RansomEye\WindowsAgent\buffer"
$env:MAX_BUFFER_SIZE_MB = "1024"
$env:BACKPRESSURE_THRESHOLD = "8192"
$env:AGENT_CERT_PATH = "C:\ProgramData\RansomEye\certs\agent.crt"
$env:AGENT_KEY_PATH = "C:\ProgramData\RansomEye\certs\agent.key"
$env:CA_CERT_PATH = "C:\ProgramData\RansomEye\certs\ca.crt"
$env:TELEMETRY_INTERVAL_SECONDS = "5"
$env:MONITOR_PATHS = "C:\,C:\Users"
$env:MONITOR_REGISTRY_KEYS = "HKCU\Software,HKLM\Software"
$env:HEALTH_REPORT_INTERVAL_SECONDS = "120"
```
