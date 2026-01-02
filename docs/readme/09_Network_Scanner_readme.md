# Phase 9: Network Scanner

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/09_Network_Scanner_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 9 - Network Scanner

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/network_scanner/`
- **Service:** `systemd/ransomeye-network-scanner.service`
- **Binary:** `ransomeye-network-scanner` (Rust)

### Core Components

1. **Active Scanner** (`scanner/src/`)
   - CIDR discovery
   - Host liveness (ICMP/TCP SYN)
   - Port enumeration (bounded by MAX_PORTS)
   - Service fingerprinting (banner-based, no exploit)
   - Rate-limited (tokens/sec, concurrency caps)

2. **Passive Scanner** (`passive/src/`)
   - Flow metadata ingestion (from Phase 4)
   - NO packet capture
   - NO payload inspection
   - Correlates flows to discovered assets

3. **Result Model & Signing** (`scanner/src/`)
   - Ed25519 signature generation and verification
   - Content hash computation
   - Immutable scan results

4. **Persistence** (`persistence/src/persistence.rs`)
   - PostgreSQL integration
   - Tables: `scan_results`, `scan_assets`, `scan_port_services`, `scan_deltas`
   - Scan metadata storage
   - Asset tracking
   - Port/service mappings
   - Scan deltas (what changed since last scan)

5. **Correlation Integration** (`correlation/src/`)
   - Exposes results to Phase 5 correlation engine
   - Asset risk changes
   - Newly exposed services
   - Unexpected exposure detection
   - NO implicit policy actions

6. **Playbook Integration** (`playbook_integration/src/`)
   - Explicit playbook triggering
   - Declarative trigger conditions
   - NO auto-execution
   - Returns playbook IDs for Phase 6 execution

7. **SOC Copilot Visibility** (`visibility/src/`)
   - Read-only access to discovered assets
   - Exposure changes
   - Scan history
   - Risk deltas
   - Cannot initiate scans

---

## WHAT DOES NOT EXIST

1. **No packet capture** - Passive scanner uses flow metadata only
2. **No payload inspection** - No deep packet inspection
3. **No exploit execution** - Banner-based fingerprinting only
4. **No automatic enforcement** - Playbooks triggered explicitly only

---

## DATABASE SCHEMAS

**PostgreSQL Tables:**

1. **scan_results**
   - `scan_id` (VARCHAR(36), PRIMARY KEY)
   - `timestamp` (TIMESTAMP WITH TIME ZONE, NOT NULL)
   - `scanner_mode` (VARCHAR(20), NOT NULL)
   - `asset_ip` (VARCHAR(45), NOT NULL)
   - `asset_hostname`, `asset_mac`, `asset_vendor` (VARCHAR)
   - `open_ports` (JSONB, NOT NULL, DEFAULT '[]')
   - `services` (JSONB, NOT NULL, DEFAULT '[]')
   - `confidence_score` (DOUBLE PRECISION, NOT NULL)
   - `hash` (VARCHAR(64), NOT NULL)
   - `signature` (TEXT, NOT NULL)
   - `metadata` (JSONB)
   - `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())

2. **scan_assets**
   - `asset_id` (SERIAL, PRIMARY KEY)
   - `ip` (VARCHAR(45), NOT NULL, UNIQUE)
   - `hostname`, `mac`, `vendor` (VARCHAR)
   - `first_seen`, `last_seen` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())
   - `scan_count` (INTEGER, NOT NULL, DEFAULT 1)

3. **scan_port_services**
   - `mapping_id` (SERIAL, PRIMARY KEY)
   - `asset_ip` (VARCHAR(45), NOT NULL)
   - `port` (INTEGER, NOT NULL)
   - `protocol` (VARCHAR(10), NOT NULL)
   - `service_name`, `service_version` (VARCHAR)
   - `first_seen`, `last_seen` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())
   - UNIQUE(asset_ip, port, protocol)

4. **scan_deltas**
   - `delta_id` (SERIAL, PRIMARY KEY)
   - `scan_id` (VARCHAR(36), NOT NULL)
   - `asset_ip` (VARCHAR(45), NOT NULL)
   - `delta_type` (VARCHAR(20), NOT NULL)
   - `delta_data` (JSONB, NOT NULL)
   - `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, DEFAULT NOW())

**Indexes:**
- `idx_scan_results_asset_ip` ON `scan_results(asset_ip)`
- `idx_scan_results_timestamp` ON `scan_results(timestamp)`
- `idx_scan_port_services_asset_ip` ON `scan_port_services(asset_ip)`
- `idx_scan_deltas_asset_ip` ON `scan_deltas(asset_ip)`

---

## RUNTIME SERVICES

**Service:** `ransomeye-network-scanner.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-network-scanner.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-correlation.service`
- **ExecStart:** `/usr/local/bin/ransomeye-network-scanner`

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/network_scanner, /run/ransomeye/network_scanner

---

## GUARDRAILS ALIGNMENT

Phase 9 enforces guardrails:

1. **Rate Limited** - Scanner rate-limited by design
2. **Signed Results** - All scan results signed with Ed25519
3. **Bounded Ports** - Port enumeration bounded by MAX_PORTS
4. **No Exploits** - Banner-based fingerprinting only
5. **Fail-Closed** - Invalid configuration → Scanner DISABLED

---

## INSTALLER BEHAVIOR

**Installation:**
- Network scanner service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Binary built from Rust crate: `core/network_scanner/`
- Service disabled by default

---

## SYSTEMD INTEGRATION

**Service File:**
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Restart always
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 9 does not use AI/ML models.

**Network scanning is deterministic:**
- Rule-based discovery
- Banner-based fingerprinting
- No ML inference
- No model loading

---

## COPILOT REALITY

**SOC Copilot Visibility:**
- Read-only access to discovered assets
- Exposure changes
- Scan history
- Risk deltas
- Cannot initiate scans
- Cannot modify results

---

## UI REALITY

**NONE** - Phase 9 has no UI.

**Scanner results:**
- Stored in PostgreSQL
- Available via correlation integration
- Available via playbook integration
- Available via SOC Copilot visibility

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Configuration** → Scanner DISABLED
2. **Rate Limit Exceeded** → Scan REJECTED
3. **Unsigned Results** → Results REJECTED
4. **Invalid Signature** → Results REJECTED
5. **Replay Attempt** → Results REJECTED

**No Bypass:**
- No `--skip-rate-limit` flag
- No `--skip-signature` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 9 is fully implemented and production-ready:

✅ **Complete Implementation**
- Active scanner functional
- Passive scanner functional
- Persistence complete (PostgreSQL)
- Correlation integration working
- Playbook integration working
- SOC Copilot visibility working

✅ **Guardrails Alignment**
- Rate limited
- Signed results
- Bounded ports
- No exploits
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Scanner disabled on any failure

**Recommendation:** Deploy as-is. Phase 9 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
