# README Generation Guide

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/README_GENERATION_GUIDE.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Systematic guide for generating phase-specific README files

---

## Overview

This guide provides a systematic approach for generating comprehensive README files for all 23 RansomEye phases. Each README must follow the mandatory structure defined in the master prompt.

---

## Completed READMEs

✅ **Phase 0**: `00_Guardrails_readme.md`  
✅ **Phase 1**: `01_Core_Engine_Installer_readme.md`  
✅ **Phase 2**: `02_AI_Core_Model_Registry_readme.md`

---

## Remaining Phases to Complete

### Core Modules (Phases 3-12)
- ⏳ **Phase 3**: Alert Engine & Policy Manager (`ransomeye_intelligence`, `core/policy`)
- ⏳ **Phase 4**: KillChain & Forensic Dump (`core/ingest`, `core/reporting`)
- ⏳ **Phase 5**: LLM Summarizer/Correlation (`core/engine/correlation`)
- ⏳ **Phase 6**: Incident Response & Playbooks (`core/policy`)
- ⏳ **Phase 7**: SOC Copilot (`core/dispatch`, `core/ai`)
- ⏳ **Phase 8**: Threat Correlation/AI Advisory (`core/ai`)
- ⏳ **Phase 9**: Network Scanner (NOT FOUND - needs creation)
- ⏳ **Phase 10**: DB Core/Reporting (`core/reporting`)
- ⏳ **Phase 11**: UI & Dashboards (`ui/`)
- ⏳ **Phase 12**: Orchestrator/Validation (`qa/validation`)

### Advanced Modules (Phases 13-20)
- ⏳ **Phase 13**: Forensic Engine (Advanced) (`core/forensics`)
- ⏳ **Phase 14**: LLM Behavior Summarizer (Expanded) (`core/narrative`)
- ⏳ **Phase 15**: SOC Copilot (Advanced) (part of Phase 8)
- ⏳ **Phase 16**: Deception Framework (NOT FOUND)
- ⏳ **Phase 17**: AI Assistant (Governor Mode) (`core/governor`)
- ⏳ **Phase 18**: Threat Intelligence Feed Engine (`ransomeye_intelligence`)
- ⏳ **Phase 19**: HNMP Engine (`ransomeye_posture_engine`)
- ⏳ **Phase 20**: Global Validator (`qa/validation`)

### Standalone Agents (Phases 21-23)
- ⏳ **Phase 21**: Linux Agent (`edge/agent/linux`)
- ⏳ **Phase 22**: Windows Agent (`edge/agent/windows`)
- ⏳ **Phase 23**: DPI Probe (`edge/dpi`)

---

## Mandatory README Structure

Each README must include all sections below:

### 1️⃣ Phase Overview
- Purpose of the phase
- Security objective
- Role in overall RansomEye architecture

### 2️⃣ Implementation Status
- ✅ Fully Implemented
- ⚠️ Partially Implemented
- ❌ Not Implemented

### 3️⃣ File & Folder Structure
- Absolute paths
- Purpose of each directory
- Key files explained

### 4️⃣ Modules & Services
- Module name
- Responsibility
- Runtime behavior
- systemd integration (YES/NO)
- Installer integration (YES/NO)

### 5️⃣ AI / ML / LLM DETAILS
For each model:
- Model name
- Type (ML / LLM / Rule / Hybrid)
- Training status (Fully trained / Partially trained / Not trained)
- Training scripts present (YES/NO)
- Incremental learning support (YES/NO)
- SHAP explainability present (YES/NO)
- Model signing & verification (YES/NO)
- Data sources used

### 6️⃣ SOC Copilot / AI Copilot
- Copilot name(s)
- Location of prompts
- Access method (CLI / API / UI)
- Supported use cases
- Training material used
- Knowledge sources
- Whether embeddings are static or dynamic
- Whether Copilot is production-ready
- If NOT PRESENT → explicitly state

### 7️⃣ Database Design
For each database:
- DB type (PostgreSQL / SQLite / Other)
- Connection method (env-only)
- Tables (name, purpose, columns, indexes)
- Data ingestion source
- Data consumers
- Retention / cleanup logic

### 8️⃣ Ports & Interconnectivity
- List all ports (env-driven)
- Inbound vs outbound
- Which module talks to which
- Protocols used
- Trust boundaries enforced (YES/NO)

### 9️⃣ UI / Dashboards / Frontend
- UI frameworks used
- Dashboards implemented
- Data sources powering UI
- Authentication & RBAC status
- Deployment status (prod/dev)
- Gaps or missing features
- If NO UI → explicitly state

### 🔟 Logging, Metrics & Observability
- Logs generated
- Log formats
- Metrics exposed
- Prometheus/Grafana integration
- Audit logs (YES/NO)
- Tamper-proofing (YES/NO)

### 1️⃣1️⃣ Security & Compliance
- Fail-closed enforcement
- Cryptographic controls
- Signature verification
- Zero-trust enforcement
- STIG hardening status

### 1️⃣2️⃣ CI / Validation / Testing
- Tests present (YES/NO)
- Synthetic data generation (YES/NO)
- CI workflows
- Validation coverage

### 1️⃣3️⃣ Known Gaps & Technical Debt
- Missing components
- Partial implementations
- Design risks

### 1️⃣4️⃣ Recommendations
- Refactors
- Missing enforcement
- Architectural fixes
- Training improvements
- Security hardening

---

## Key Sources of Information

### Module Mapping
- **File**: `/home/ransomeye/rebuild/MODULE_PHASE_MAP.yaml`
- **File**: `/home/ransomeye/rebuild/MODULE_PHASE_MAP.md`
- Contains canonical mapping of modules to phases

### Systemd Services
- **Directory**: `/home/ransomeye/rebuild/systemd/`
- Lists all systemd service files and their configurations

### Architecture Documentation
- **Directory**: `/home/ransomeye/rebuild/ransomeye_architecture/`
- Contains plane definitions and trust boundaries

### Existing READMEs
- Each module may have existing README.md files
- Use as starting point but verify all claims against actual code

---

## Critical Rules

1. **NO ASSUMPTIONS** - Only document what actually exists in the codebase
2. **HONEST STATUS** - Use "NOT IMPLEMENTED" or "PARTIAL" where applicable
3. **NO MARKETING** - Factual, technical language only
4. **ENV-ONLY CONFIG** - All configuration must be ENV-driven
5. **UNIFIED SYSTEMD** - All services in `/home/ransomeye/rebuild/systemd/`
6. **UNIFIED REQUIREMENTS** - Single `/home/ransomeye/rebuild/requirements.txt`

---

## Naming Convention

All README files must follow this naming pattern:
```
<PhaseNumber>_<PhaseName>_readme.md
```

Examples:
- `03_Alert_Engine_readme.md`
- `05_Correlation_Engine_readme.md`
- `08_AI_Advisory_readme.md`
- `21_Linux_Agent_readme.md`

---

## Validation Checklist

Before considering a README complete, verify:
- [ ] All 14 mandatory sections present
- [ ] All paths are absolute
- [ ] All status indicators are factual (✅/⚠️/❌)
- [ ] No assumptions or unverified claims
- [ ] Database schema documented (if applicable)
- [ ] Models documented with training status (if applicable)
- [ ] Ports listed with ENV variable names (if applicable)
- [ ] Known gaps explicitly stated
- [ ] File naming follows convention

---

## Notes on Phantom Modules

Some phases reference modules that don't exist (phantom modules). These are documented in `MODULE_PHASE_MAP.yaml`. For phantom modules:
- Explicitly state "NOT IMPLEMENTED" or "NOT FOUND"
- Note the canonical mapping if applicable
- Do not create placeholder documentation

---

**Last Updated**: 2025-01-27

