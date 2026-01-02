# RansomEye Core Systemd Services

**Path and File Name:** `/home/ransomeye/rebuild/systemd/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Core systemd services directory - ONLY Core services allowed

---

## ⚠️ CRITICAL GUARDRAIL: AGENTS ARE STANDALONE

**Agents are standalone modules and MUST NOT be placed here.**

### Standalone Modules (NOT in this directory)

- **Linux Agent:** `/home/ransomeye/rebuild/edge/agent/linux/systemd/`
- **Windows Agent:** `/home/ransomeye/rebuild/edge/agent/windows/systemd/`
- **DPI Probe:** `/home/ransomeye/rebuild/edge/dpi/systemd/`

### Core Services Only

This directory contains ONLY Core RansomEye services:
- `ransomeye-orchestrator.service`
- `ransomeye-core.service`
- `ransomeye-policy.service`
- `ransomeye-intelligence.service`
- `ransomeye-correlation.service`
- `ransomeye-reporting.service`
- `ransomeye-enforcement.service`
- `ransomeye-retention-enforcer.service`
- `ransomeye-retention-enforcer.timer`
- `ransomeye-ingestion.service`
- `ransomeye-network-scanner.service`
- `ransomeye-playbook-engine.service`
- `ransomeye-posture-engine.service`
- `ransomeye-sentinel.service`
- `ransomeye-feed-fetcher.service`
- `ransomeye-feed-retraining.service`
- `ransomeye-github-sync.service`

### Dependency Notes

Core services (e.g., `ransomeye-sentinel.service`) may reference standalone agent services via `Wants`/`After` directives. This is acceptable as it represents a runtime dependency, not ownership.

---

*Generated: 2025-12-30*  
*Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU*

