# RansomEye Ingestion Environment Variables Schema

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/config/env_schema.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Complete environment variable schema for RansomEye Ingestion configuration

---

## Required Environment Variables

None - all variables have defaults, but validation will enforce constraints on values.

## Optional Environment Variables

### Network Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RANSOMEYE_INGESTION_LISTEN_ADDR` | String | `0.0.0.0:8080` | Listen address for event ingestion |
| `RANSOMEYE_CONTROL_PLANE_ADDR` | String | `127.0.0.1:9090` | Control plane API endpoint address |

### Buffer Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RANSOMEYE_BUFFER_CAPACITY` | Integer | `10000` | Maximum number of events in buffer |

### Rate Limiting Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RANSOMEYE_PRODUCER_RATE_LIMIT` | Integer | `1000` | Maximum events per producer per window |
| `RANSOMEYE_GLOBAL_RATE_LIMIT` | Integer | `10000` | Maximum events globally per window |
| `RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS` | Integer | `60` | Rate limit window duration in seconds |

### Backpressure Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RANSOMEYE_BACKPRESSURE_CLEAR_SECONDS` | Integer | `10` | Seconds before backpressure auto-clears |

### Security Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RANSOMEYE_TRUST_STORE_PATH` | String | `/etc/ransomeye/trust_store` | Path to trust store directory |
| `RANSOMEYE_CRL_PATH` | String | (optional) | Path to Certificate Revocation List |

## Configuration Validation

All integer values must be:
- Positive (> 0)
- Within reasonable bounds:
  - `RANSOMEYE_BUFFER_CAPACITY`: 1 - 1,000,000
  - `RANSOMEYE_PRODUCER_RATE_LIMIT`: 1 - 1,000,000
  - `RANSOMEYE_GLOBAL_RATE_LIMIT`: 1 - 10,000,000
  - `RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS`: 1 - 3600
  - `RANSOMEYE_BACKPRESSURE_CLEAR_SECONDS`: 1 - 300

## Security Notes

- Trust store path must be readable by the ingestion process
- CRL path must be readable if provided
- All paths must be absolute (no relative paths)
- Listen address must be a valid IP:port combination
- Control plane address must be a valid IP:port combination

## Example Configuration

```bash
export RANSOMEYE_INGESTION_LISTEN_ADDR="0.0.0.0:8080"
export RANSOMEYE_CONTROL_PLANE_ADDR="127.0.0.1:9090"
export RANSOMEYE_BUFFER_CAPACITY="10000"
export RANSOMEYE_PRODUCER_RATE_LIMIT="1000"
export RANSOMEYE_GLOBAL_RATE_LIMIT="10000"
export RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS="60"
export RANSOMEYE_BACKPRESSURE_CLEAR_SECONDS="10"
export RANSOMEYE_TRUST_STORE_PATH="/etc/ransomeye/trust_store"
export RANSOMEYE_CRL_PATH="/etc/ransomeye/crl/revocation.json"
```

