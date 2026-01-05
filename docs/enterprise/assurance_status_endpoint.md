# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/assurance_status_endpoint.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Assurance Status Endpoint Documentation (PROMPT-60-C)

# Assurance Status Endpoint (PROMPT-60-C)

## Overview

Read-only local endpoint exposing assurance status for monitoring and verification.

## Endpoint

### URL

`http://127.0.0.1:8082/status/assurance`

### Default Configuration

- **Host**: `127.0.0.1` (localhost-only)
- **Port**: `8082`
- **Environment Variables**:
  - `RANSOMEYE_STATUS_HOST`: Override host (default: `127.0.0.1`)
  - `RANSOMEYE_STATUS_PORT`: Override port (default: `8082`)

## Response Format

### JSON Response

```json
{
  "timestamp": "2026-01-05T10:00:00+00:00",
  "verifier": {
    "green": true,
    "last_check": "2026-01-05T10:00:00+00:00",
    "assurance_mode": true
  },
  "audit": {
    "total_count": 12345,
    "last_entry": "2026-01-05T10:00:00+00:00"
  },
  "models": {
    "model_count": 5,
    "shap_explanations": 1000
  },
  "threat_intel": {
    "ioc_count": 5000,
    "last_updated": "2026-01-05T09:00:00+00:00"
  },
  "drift": {
    "baseline_exists": true,
    "drift_detected": false
  }
}
```

## Status Components

### Verifier Status

- **green**: Boolean indicating verifier health
- **last_check**: Timestamp of last verifier check
- **assurance_mode**: Boolean indicating assurance mode active

### Audit Status

- **total_count**: Total audit log entries
- **last_entry**: Timestamp of last audit entry

### Models Status

- **model_count**: Number of registered models
- **shap_explanations**: Total SHAP explanations

### Threat Intel Status

- **ioc_count**: Total IOC count
- **last_updated**: Timestamp of last threat intel update

### Drift Status

- **baseline_exists**: Boolean indicating baseline exists
- **drift_detected**: Boolean indicating drift detected

## Security

### Localhost-Only by Default

- Default binding: `127.0.0.1`
- No external exposure without explicit configuration
- No authentication required (read-only)

### Environment Control

Expose externally only if explicitly configured:

```bash
export RANSOMEYE_STATUS_HOST=0.0.0.0
export RANSOMEYE_STATUS_PORT=8082
```

### No Secrets

- No credentials exposed
- No sensitive data in response
- Read-only status information only

## Usage

### Start Service

```bash
python3 /home/ransomeye/rebuild/core/status/assurance_status_endpoint.py
```

### Query Status

```bash
curl http://127.0.0.1:8082/status/assurance
```

### Systemd Service (Optional)

Create systemd service for persistent operation:

```ini
[Unit]
Description=RansomEye Assurance Status Endpoint
After=network.target

[Service]
Type=simple
User=ransomeye
ExecStart=/usr/bin/python3 /home/ransomeye/rebuild/core/status/assurance_status_endpoint.py
Restart=always

[Install]
WantedBy=multi-user.target
```

## Acceptance Criteria

- [x] Localhost-only by default
- [x] Env-controlled exposure
- [x] No secrets exposed
- [x] Read-only status information
- [x] JSON response format

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

