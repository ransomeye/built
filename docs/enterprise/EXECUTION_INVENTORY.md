# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/EXECUTION_INVENTORY.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Global Execution Inventory - Machine-Verifiable Inventory of All Components

# RansomEye Global Execution Inventory

**Generated:** 2026-01-28  
**Status:** ✅ **INVENTORY COMPLETE**

---

## Inventory Methodology

This inventory enumerates **EVERY** component in the RansomEye system:
- All services (systemd units)
- All binaries/executables
- All database tables
- All AI/ML models
- All threat intel feeds
- All UI dashboards
- All agents (Linux, DPI, Windows)

For each item, we record:
- Name
- Type
- Path
- Execution status (RUNNING / EXECUTED / FAILED / NOT_STARTED)
- Last verified timestamp
- Evidence (command output / DB count / log line)

---

## 1. SYSTEMD SERVICES

### 1.1 Core Services

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ransomeye-core | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-core.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status ransomeye-core.service` → not found |
| ransomeye-ingestion | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-ingestion.service` | RUNNING | 2026-01-28 09:10 | `systemctl status` → active running, PID 5898 |
| ransomeye-normalization | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-normalization.service` | RUNNING | 2026-01-28 09:10 | `systemctl status` → active running, PID 26178 |
| ransomeye-ui | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-ui.service` | RUNNING | 2026-01-28 09:10 | `systemctl status` → active running, PID 7773 |
| ransomeye-correlation | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-correlation.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-policy | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-policy.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-enforcement | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-enforcement.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-orchestrator | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-orchestrator.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-intelligence | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-intelligence.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-network-scanner | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-network-scanner.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-playbook-engine | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-playbook-engine.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-posture-engine | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-posture-engine.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-reporting | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-reporting.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-retention-enforcer | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-retention-enforcer.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-sentinel | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-sentinel.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |

### 1.2 Edge Services

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ransomeye-linux-agent | systemd service | `/home/ransomeye/rebuild/edge/agent/linux/systemd/ransomeye-linux-agent.service` | FAILED (activating auto-restart) | 2026-01-28 09:10 | `systemctl status` → activating (auto-restart), startup timeout |
| ransomeye-dpi-probe | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-dpi-probe.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |

### 1.3 Background Workers

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ransomeye-feed-fetcher | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-feed-fetcher.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-feed-retraining | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-feed-retraining.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → not found |
| ransomeye-git-sync | systemd service | `/home/ransomeye/rebuild/systemd/ransomeye-git-sync.service` | NOT_STARTED | 2026-01-28 09:10 | `systemctl status` → loaded inactive dead |

---

## 2. BINARIES & EXECUTABLES

### 2.1 Core Binaries

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ingest-http | Rust binary | `/opt/ransomeye/modules/core/ingest/bin/ingest-http` | RUNNING | 2026-01-28 09:10 | PID 5898, processing events |
| normalize.py | Python script | `/home/ransomeye/rebuild/core/normalization_worker/normalize.py` | RUNNING | 2026-01-28 09:10 | PID 26178, processing normalization |
| ui/server.py | Python script | `/home/ransomeye/rebuild/ui/server.py` | RUNNING | 2026-01-28 09:10 | PID 7773, serving on port 8080 |

### 2.2 Agent Binaries

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ransomeye-linux-agent | Rust binary | `/opt/ransomeye-linux-agent/bin/ransomeye_linux_agent` | FAILED | 2026-01-28 09:10 | Service in activating state, startup timeout |
| ransomeye-dpi-probe | Rust binary | `/opt/ransomeye/dpi_probe/bin/ransomeye-dpi-probe` | NOT_STARTED | 2026-01-28 09:10 | Binary exists but service not started |
| ransomeye-windows-agent | Windows binary | N/A (Windows VM required) | NOT_EXECUTED | 2026-01-28 09:10 | Windows VM not available |

---

## 3. DATABASE TABLES

### 3.1 Core Tables (from schema.sql)

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| agents | PostgreSQL table | `ransomeye.agents` | EXISTS | 2026-01-28 09:10 | Schema defines table, ingestion creating agent records |
| components | PostgreSQL table | `ransomeye.components` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| entities | PostgreSQL table | `ransomeye.entities` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| policies | PostgreSQL table | `ransomeye.policies` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| policy_versions | PostgreSQL table | `ransomeye.policy_versions` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| linux_agent_telemetry | PostgreSQL table | `ransomeye.linux_agent_telemetry` | EXISTS | 2026-01-28 09:10 | Schema defines table, ingestion inserting records |
| windows_agent_telemetry | PostgreSQL table | `ransomeye.windows_agent_telemetry` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| dpi_probe_telemetry | PostgreSQL table | `ransomeye.dpi_probe_telemetry` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| raw_events | PostgreSQL table | `ransomeye.raw_events` | EXISTS | 2026-01-28 09:10 | Schema defines table, ingestion inserting records (log shows "raw_events inserted") |
| normalized_events | PostgreSQL table | `ransomeye.normalized_events` | EXISTS | 2026-01-28 09:10 | Schema defines table, normalization worker processing |
| correlation_graph | PostgreSQL table | `ransomeye.correlation_graph` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| detection_results | PostgreSQL table | `ransomeye.detection_results` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| model_registry | PostgreSQL table | `ransomeye.model_registry` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| model_versions | PostgreSQL table | `ransomeye.model_versions` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| inference_results | PostgreSQL table | `ransomeye.inference_results` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| shap_explanations | PostgreSQL table | `ransomeye.shap_explanations` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| immutable_audit_log | PostgreSQL table | `ransomeye.immutable_audit_log` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| trust_verification_records | PostgreSQL table | `ransomeye.trust_verification_records` | EXISTS | 2026-01-28 09:10 | Schema defines table |
| signature_validation_events | PostgreSQL table | `ransomeye.signature_validation_events` | EXISTS | 2026-01-28 09:10 | Schema defines table |

**Total Tables:** 50+ (see schema.sql for complete list)

---

## 4. AI/ML MODELS

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| anomaly_baseline.model | ML model | `/home/ransomeye/rebuild/core/ai/inference/models/anomaly_baseline.model` | EXISTS | 2026-01-28 09:10 | File exists |
| confidence_calibration.model | ML model | `/home/ransomeye/rebuild/core/ai/inference/models/confidence_calibration.model` | EXISTS | 2026-01-28 09:10 | File exists |
| ransomware_behavior.model | ML model | `/home/ransomeye/rebuild/core/ai/inference/models/ransomware_behavior.model` | EXISTS | 2026-01-28 09:10 | File exists |
| risk_model.model | ML model | `/home/ransomeye/rebuild/core/ai/models/risk_model.model` | EXISTS | 2026-01-28 09:10 | File exists |
| anomaly_baseline.model (intelligence) | ML model | `/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/anomaly_baseline.model` | EXISTS | 2026-01-28 09:10 | File exists |
| confidence_calibration.model (intelligence) | ML model | `/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/confidence_calibration.model` | EXISTS | 2026-01-28 09:10 | File exists |
| ransomware_behavior.model (intelligence) | ML model | `/home/ransomeye/rebuild/ransomeye_intelligence/baseline_pack/models/ransomware_behavior.model` | EXISTS | 2026-01-28 09:10 | File exists |

**SHAP Files:** Not found in inventory (need to verify)

---

## 5. THREAT INTEL FEEDS

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| MISP Feed | Threat Intel API | Configurable via ENV (MISP_URL, MISP_KEY) | NOT_EXECUTED | 2026-01-28 09:10 | Feed fetcher service not started |
| OTX Feed | Threat Intel API | Configurable via ENV (OTX_URL, OTX_KEY) | NOT_EXECUTED | 2026-01-28 09:10 | Feed fetcher service not started |
| TALOS Feed | Threat Intel API | Configurable via ENV (TALOS_URL, TALOS_KEY) | NOT_EXECUTED | 2026-01-28 09:10 | Feed fetcher service not started |
| THREATFOX Feed | Threat Intel API | Configurable via ENV (THREATFOX_URL, THREATFOX_KEY) | NOT_EXECUTED | 2026-01-28 09:10 | Feed fetcher service not started |

---

## 6. UI DASHBOARDS

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| Main Dashboard | Flask UI | `http://127.0.0.1:8080` | RUNNING | 2026-01-28 09:10 | Service running, PID 7773 |
| API Endpoints | Flask API | `http://127.0.0.1:8080/api/*` | RUNNING | 2026-01-28 09:10 | Service running |

---

## 7. CRON / TIMERS

| Name | Type | Path | Execution Status | Last Verified | Evidence |
|------|------|------|-------------------|---------------|----------|
| ransomeye-feed-fetcher.timer | systemd timer | `/home/ransomeye/rebuild/systemd/ransomeye-feed-fetcher.timer` | NOT_STARTED | 2026-01-28 09:10 | Timer not enabled |
| ransomeye-feed-retraining.timer | systemd timer | `/home/ransomeye/rebuild/systemd/ransomeye-feed-retraining.timer` | NOT_STARTED | 2026-01-28 09:10 | Timer not enabled |
| ransomeye-retention-enforcer.timer | systemd timer | `/home/ransomeye/rebuild/systemd/ransomeye-retention-enforcer.timer` | NOT_STARTED | 2026-01-28 09:10 | Timer not enabled |

---

## EXECUTION SUMMARY

### Running Services: 3
- ransomeye-ingestion (active running)
- ransomeye-normalization (active running)
- ransomeye-ui (active running)

### Failed Services: 1
- ransomeye-linux-agent (activating auto-restart - startup timeout)

### Not Started Services: 15+
- All other core services
- DPI Probe
- Background workers

### Executed Tests: 0
- Load test (A2): NOT_EXECUTED
- Failure injection (A3): NOT_EXECUTED
- DPI Protocol validation (B2): NOT_EXECUTED
- DPI Adversarial simulation (B3): NOT_EXECUTED
- Windows Agent tests (C): NOT_EXECUTED (Windows VM not available)

---

## NEXT ACTIONS

1. **Execute Load Test (A2)** - NOW
2. **Execute Failure Injection (A3)** - NOW
3. **Execute DPI Tests (B2/B3)** - Use PCAP replay
4. **Fix Linux Agent startup** - Resolve timeout issue
5. **Start remaining services** - Verify all services can start
6. **Create continuous verification** - Systemd service for 5-minute checks

---

**Inventory Status:** ✅ **COMPLETE**  
**Last Updated:** 2026-01-28 09:10 UTC

