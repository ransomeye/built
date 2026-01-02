# Phase 9: Network Scanner

**Path:** `/home/ransomeye/rebuild/core/network_scanner/`

## Overview

Phase 9 implements a standalone, first-class Network Scanner that provides proactive network visibility through active and passive discovery modes. The scanner is rate-limited by design, produces signed and auditable scan results, and integrates cleanly with Phase 4 (Ingestion), Phase 5 (Correlation), Phase 6 (Playbooks), Phase 8 (SOC Copilot), and Phase 10 (DB).

## Architecture

### Components

1. **Active Scanner** (`scanner.rs`)
   - CIDR discovery
   - Host liveness (ICMP/TCP SYN)
   - Port enumeration (bounded by MAX_PORTS)
   - Service fingerprinting (banner-based, no exploit)
   - Rate-limited (tokens/sec, concurrency caps)

2. **Passive Scanner** (`passive.rs`)
   - Flow metadata ingestion (from Phase 4)
   - NO packet capture
   - NO payload inspection
   - Correlates flows to discovered assets

3. **Result Model & Signing** (`result.rs`, `security.rs`)
   - Ed25519 signature generation and verification
   - Content hash computation
   - Immutable scan results

4. **Persistence** (`persistence.rs`)
   - PostgreSQL integration
   - Scan metadata storage
   - Asset tracking
   - Port/service mappings
   - Scan deltas (what changed since last scan)

5. **Correlation Integration** (`correlation.rs`)
   - Exposes results to Phase 5 correlation engine
   - Asset risk changes
   - Newly exposed services
   - Unexpected exposure detection
   - NO implicit policy actions

6. **Playbook Integration** (`playbook_integration.rs`)
   - Explicit playbook triggering
   - Declarative trigger conditions
   - NO auto-execution
   - Returns playbook IDs for Phase 6 execution

7. **SOC Copilot Visibility** (`visibility.rs`)
   - Read-only access to discovered assets
   - Exposure changes
   - Scan history
   - Risk deltas
   - Cannot initiate scans

## Scanner Modes

### Active Mode
- Performs active network discovery
- Rate-limited by design
- Bounded port enumeration
- Banner-based service fingerprinting (no exploits)

### Passive Mode
- Processes flow metadata from Phase 4
- NO packet capture
- NO payload inspection
- Asset discovery from observed traffic

## Configuration (ENV-ONLY)

All configuration via environment variables:

- `SCAN_CIDRS`: Comma-separated CIDR ranges to scan
- `MAX_PORTS`: Maximum ports to scan per host (1-65535)
- `MAX_RATE`: Maximum scan rate (tokens per second)
- `SCAN_TIMEOUT`: Maximum scan duration in seconds
- `ACTIVE_MODE_ENABLED`: Enable active scanning (true/false)
- `PASSIVE_MODE_ENABLED`: Enable passive scanning (true/false)
- `MAX_CONCURRENT_SCANS`: Maximum concurrent scans
- `RANSOMEYE_SCANNER_PRIVATE_KEY_PATH`: Path to Ed25519 private key
- `RANSOMEYE_SCANNER_PLAYBOOK_TRIGGERS`: Path to playbook trigger config
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASS`: Database connection

Missing or unsafe values → fail-closed.

## Fail-Closed Behavior

The system fails closed on:
- Invalid configuration
- Rate limit exceeded
- Unsigned scan results
- Invalid signatures
- Replay attempts

## Rate Limiting

Active scanner is rate-limited by design:
- Token-based rate limiting (tokens per second)
- Concurrency caps (max concurrent scans)
- Cannot exceed configured limits

## Testing

Comprehensive test suite covers:
- Active scan respects rate limits
- Passive scan never inspects payloads
- Unsigned result rejection
- Duplicate scan replay detection
- Diff logic correctness
- DB persistence integrity
- Correlation ingestion correctness

## Integration Points

- **Phase 4 (Ingestion)**: Receives flow metadata for passive scanning
- **Phase 5 (Correlation)**: Exposes scan results for correlation
- **Phase 6 (Playbooks)**: Provides explicit playbook trigger IDs
- **Phase 8 (SOC Copilot)**: Provides read-only visibility interface
- **Phase 10 (DB)**: Persists scan results, assets, deltas

## Status

✅ **IMPLEMENTED** - Phase 9 is complete and production-ready.

