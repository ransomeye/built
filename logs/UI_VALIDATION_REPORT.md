# RansomEye UI - End-to-End Live Data Validation Report

**Generated:** 2026-01-07 13:41:35 UTC  
**Validation Script:** `/home/ransomeye/rebuild/ui/validate_live_data.py`

---

## EXECUTIVE SUMMARY

**STATUS: ❌ BLOCKED - NOT READY FOR PRODUCTION**

The validation identified **critical blockers** preventing the UI from displaying real system health metrics. While the database contains telemetry data and APIs are functioning, the **system metrics are not being sent in the expected format** by the agents/probes.

---

## PART 1 — DATABASE DATA PRESENCE

### ✅ PASSED: Data Volume & Freshness

- **Linux Agent Telemetry:**
  - Total rows: **82,562**
  - Unique agents: **1,656**
  - Data freshness: **Active** (last seen < 1 minute ago)
  - Oldest record: 2026-01-03 11:06:49 UTC
  - Newest record: 2026-01-07 13:40:58 UTC

- **DPI Probe Telemetry:**
  - Total rows: **174,837**
  - Unique probes: **3**
  - Data freshness: **Active** (last seen < 1 minute ago)
  - Oldest record: 2026-01-06 13:17:27 UTC
  - Newest record: 2026-01-07 13:41:08 UTC

- **Database Metrics (pg_stat_*):**
  - Active connections: 3/100
  - Cache hits: 77,492,115
  - Disk reads: 50,434,949
  - Total transactions: 1,748,947

### ❌ BLOCKER: System Metrics Missing

**Linux Agent Telemetry:**
- **Rows with system metrics: 0** (out of 82,562 total)
- **Root Cause:** Linux agents are not sending system metrics (CPU, memory, disk, network) in telemetry payloads
- **Expected Structure:** Payload should contain top-level keys: `cpu`, `memory`, `disk`, `network`, `filesystem`, `system`
- **Actual Structure:** Payloads are NULL or do not contain system metrics

**DPI Probe Telemetry:**
- **Rows with system metrics: 8** (out of 174,837 total)
- **Root Cause:** DPI probes are not sending system metrics in the expected format
- **Expected Structure:** Payload should contain a top-level `system` key with nested metrics
- **Actual Structure:** Payloads contain network flow data (`dst_ip`, `src_ip`, `protocol`, etc.) but no `system` key
- **Note:** The 8 rows that matched the LIKE query only matched because the word "system" appears in string representations (likely in protocol names or metadata), not as an actual key

---

## PART 2 — BACKEND API VALIDATION

### ✅ PASSED: API Endpoints Functional

All three dashboard API endpoints are responding correctly:

1. **`/api/dashboards/core-system-health`**
   - Status: ✅ Returns HTTP 200
   - Instance filtering: ✅ Works (returns 404 for invalid instances)
   - **Issue:** Response contains only placeholders ("Metric unavailable") due to missing system metrics in database

2. **`/api/dashboards/dpi-probe-health`**
   - Status: ✅ Returns HTTP 200
   - Instance filtering: ✅ Works (returns 404 for invalid instances)
   - **Issue:** Response contains only placeholders ("Metric unavailable") due to missing system metrics in database

3. **`/api/dashboards/db-health`**
   - Status: ✅ Returns HTTP 200
   - Instance filtering: ✅ Works correctly
   - **Data:** ✅ Contains real numeric data from pg_stat_* views

4. **`/api/system/instances`**
   - Status: ✅ Returns HTTP 200
   - Discovered instances:
     - Core: 2 instances
     - DPI: 3 instances
     - DB: 1 instance

### ✅ PASSED: Fail-Soft Behavior

- Invalid instance IDs correctly return HTTP 404
- Malformed query parameters are handled safely
- APIs gracefully handle missing data (return "Metric unavailable" instead of crashing)

---

## PART 3 — UI RENDERING VALIDATION

### ⚠️ MANUAL TESTING REQUIRED

This section requires manual validation from a Windows browser. The following checklist must be verified:

1. **Network Access:** Access UI via network IP (not localhost)
2. **Core System Health Dashboard:** Panels render numeric data (not placeholders)
3. **DPI Probe Health Dashboard:** Panels render numeric data (not placeholders)
4. **DB Health Dashboard:** Panels render numeric data (not placeholders)
5. **Refresh Updates:** Dashboard values change over time when refreshed
6. **Instance Selector (Core):** Switch between instances - panel values change
7. **Instance Selector (DPI):** Switch between probes - panel values change
8. **Instance Selector (DB):** Switch between DB instances - panel values change
9. **URL Query Params:** URL updates correctly when instance changes
10. **Offline Instance:** Offline instance shows graceful error (not crash)

**Note:** Based on Part 1 findings, items 2 and 3 will likely show placeholders until system metrics are properly sent by agents/probes.

---

## PART 4 — FAILURE & FAIL-SOFT TESTS

### ✅ PASSED: Error Handling

- Invalid instance IDs: ✅ Correctly return HTTP 404
- Malformed parameters: ✅ Handled safely (no SQL injection risk)
- Missing data: ✅ APIs return "Metric unavailable" instead of crashing
- Database connectivity: ✅ Error handling is graceful

---

## ROOT CAUSE ANALYSIS

### Primary Blocker: System Metrics Not in Expected Format

**Linux Agent:**
- **Expected:** Telemetry payloads should contain system metrics with structure:
  ```json
  {
    "cpu": { "utilization": 45.2, "load_avg_1m": 1.5, ... },
    "memory": { "total": 16777216, "used": 8388608, ... },
    "disk": { "read_iops": 100, "write_iops": 50, ... },
    "network": { "bytes_in": 1024, "bytes_out": 2048, ... },
    "filesystem": { "root_usage": 75.5, ... },
    "system": { "uptime": 86400, "process_count": 150, ... }
  }
  ```
- **Actual:** Payloads are NULL or do not contain these keys
- **Impact:** Core System Health dashboard shows only "Metric unavailable"

**DPI Probe:**
- **Expected:** Telemetry payloads should contain system metrics with structure:
  ```json
  {
    "system": {
      "cpu": { "utilization": 45.2, "dpi_process_cpu": 12.5, ... },
      "memory": { "total": 16777216, "dpi_process_rss": 524288, ... },
      "disk": { "read_throughput": 1024000, ... },
      "network": { "packets_per_sec": 1000, "bytes_per_sec": 1024000, ... },
      "processing": { "packet_processing_rate": 1000, ... },
      "system_state": { "host_uptime": 86400, ... }
    }
  }
  ```
- **Actual:** Payloads contain network flow data but no `system` key
- **Impact:** DPI Probe Health dashboard shows only "Metric unavailable"

---

## FIXES REQUIRED

### 1. Linux Agent System Metrics Collection

**File:** `edge/agent/linux/src/telemetry.rs` (or equivalent)

**Required Changes:**
- Implement system metrics collection (CPU, memory, disk, network, filesystem, system state)
- Include system metrics in telemetry payload when sending to ingestion server
- Ensure payload structure matches expected format (top-level keys: `cpu`, `memory`, `disk`, `network`, `filesystem`, `system`)

**Reference:** See `ransomeye_dpi_probe/SYSTEM_METRICS_COLLECTION.md` for implementation guidance

### 2. DPI Probe System Metrics Collection

**File:** `edge/dpi/probe/src/main.rs` (or equivalent)

**Required Changes:**
- System metrics collector exists (`edge/dpi/probe/src/system_metrics.rs`) but metrics are not being included in telemetry payloads
- Modify telemetry sending logic to include system metrics in payload with structure:
  ```json
  {
    "system": { ... }
  }
  ```
- Ensure system metrics are collected periodically (every 5-10 seconds) and included in telemetry

**Reference:** `edge/dpi/probe/src/system_metrics.rs` already has the collector implementation

---

## VALIDATION CHECKLIST

### Automated Tests
- [x] Database contains telemetry data
- [x] Data is fresh (updating continuously)
- [x] API endpoints respond correctly
- [x] Instance filtering works
- [x] Fail-soft behavior is correct
- [ ] **System metrics present in Linux Agent telemetry** ❌
- [ ] **System metrics present in DPI Probe telemetry** ❌

### Manual Tests (Windows Browser)
- [ ] Network access works
- [ ] Core System Health dashboard shows real data
- [ ] DPI Probe Health dashboard shows real data
- [ ] DB Health dashboard shows real data
- [ ] Instance selector works correctly
- [ ] URL query params update correctly
- [ ] Offline instances show graceful errors

---

## FINAL VERDICT

**STATUS: ❌ BLOCKED - NOT READY FOR PRODUCTION**

### Blockers:
1. **Linux Agent telemetry missing system metrics** - No system metrics in payloads
2. **DPI Probe telemetry missing system metrics** - System metrics not in expected format

### Next Steps:
1. Fix Linux Agent to send system metrics in telemetry payloads
2. Fix DPI Probe to include system metrics in telemetry payloads (collector exists but not integrated)
3. Re-run validation script after fixes
4. Perform manual UI validation from Windows browser

### Positive Findings:
- ✅ Database is actively receiving telemetry
- ✅ APIs are functioning correctly
- ✅ Instance discovery works
- ✅ Fail-soft behavior is correct
- ✅ DB Health dashboard works (uses pg_stat_* views directly)

---

## SQL QUERIES FOR VERIFICATION

### Check Linux Agent System Metrics:
```sql
SELECT COUNT(*) 
FROM ransomeye.linux_agent_telemetry 
WHERE payload IS NOT NULL 
  AND (payload::text LIKE '%"cpu"%' 
       OR payload::text LIKE '%"memory"%'
       OR payload::text LIKE '%"disk"%'
       OR payload::text LIKE '%"network"%');
```

### Check DPI Probe System Metrics:
```sql
SELECT COUNT(*) 
FROM ransomeye.dpi_probe_telemetry 
WHERE payload IS NOT NULL 
  AND payload::text LIKE '%"system"%';
```

### Check Data Freshness:
```sql
SELECT 
    'linux_agent' as source,
    MAX(observed_at) as last_seen,
    EXTRACT(EPOCH FROM (NOW() - MAX(observed_at)))::int as seconds_ago
FROM ransomeye.linux_agent_telemetry
UNION ALL
SELECT 
    'dpi_probe' as source,
    MAX(observed_at) as last_seen,
    EXTRACT(EPOCH FROM (NOW() - MAX(observed_at)))::int as seconds_ago
FROM ransomeye.dpi_probe_telemetry;
```

---

**Report Generated By:** RansomEye UI Validation Script  
**Report Location:** `/home/ransomeye/rebuild/logs/ui_validation_20260107_134135.json`

