# RansomEye Security Invariants

**Path and File Name:** `/home/ransomeye/rebuild/docs/SECURITY_INVARIANTS.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Immutable security guarantees enforced by Phase 0 guardrails

---

## Overview

This document defines the **immutable security guarantees** enforced by RansomEye Phase 0 guardrails. These invariants are **fail-closed** and **non-negotiable**. Any violation results in immediate build failure.

---

## 1. ENV-Only Configuration Invariant

**Rule:** All configuration must be environment-driven. No hardcoded IPs, ports, paths, or secrets.

**Enforcement:**
- Static scanner detects hardcoded values
- AST parser validates string literals
- Build fails if any hardcoded configuration is found

**Exceptions:**
- `localhost` and `127.0.0.1` are allowed for local development

**Required ENV Variables:**
- Database: `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASS`
- Models: `MODEL_DIR`, `PDF_ENGINE_PATH`, `OUTPUT_DIR`
- API: `CORE_API_PORT`, `FRONTEND_PORT`, `BACKEND_API_PORT`
- Retention: `RETENTION_YEARS`, `DB_ENCRYPTION_KEY_PATH`
- Network: `CAPTURE_IFACE`, `COMPLIANCE_CONFIG`
- AI: `ASSISTANT_TOPK`, `ASSISTANT_MAX_TOKENS`, `ASSISTANT_DATA_DIR`
- Agents: `ENABLE_EBPF`, `ENABLE_KERNEL_DRIVER`, `UPDATE_BUNDLE_DIR`
- Threat Intel: `MISP_URL`, `MISP_KEY`, `OTX_URL`, `OTX_KEY`, `TALOS_URL`, `TALOS_KEY`, `THREATFOX_URL`, `THREATFOX_KEY`

---

## 2. Mandatory File Header Invariant

**Rule:** Every source file must contain the mandatory header.

**Header Format:**
```
# Path and File Name : <absolute_path>
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: <brief description>
```

**Enforcement:**
- Header enforcer validates all files
- Build fails if any file is missing the header

**Exceptions:**
- Configuration files: `rules.yaml`, `retention.txt`, `key_hierarchy.json`
- Documentation: `*.md` files
- Build artifacts: `__pycache__`, `.git`, `node_modules`, `venv`

---

## 3. Train-From-Scratch ML Invariant

**Rule:** All ML models must be trained from scratch. No pretrained weights allowed without training scripts.

**Enforcement:**
- ML enforcer detects model files
- Requires corresponding training script
- Build fails if model lacks training context

**Requirements:**
- Every `.pkl`, `.h5`, `.pb`, `.onnx`, `.pt`, `.pth`, `.ckpt`, `.gguf` file must have:
  - Training script in same or parent directory
  - Training indicators: `train`, `fit`, `train_from_scratch`, `incremental_train`
  - SHAP explainability file
  - Metadata JSON with `hash`, `trained_on`, `version`

---

## 4. SHAP Explainability Invariant

**Rule:** Every ML inference with numeric output must emit SHAP explainability artifacts.

**Enforcement:**
- ML enforcer scans for inference calls
- Validates SHAP usage in context
- Build fails if inference lacks SHAP

**Requirements:**
- All `predict()`, `inference()`, `forward()`, `eval()` calls must have:
  - SHAP explainer in same file or module
  - SHAP output generation
  - SHAP artifact files for models

---

## 5. Cryptographic Signing Invariant

**Rule:** Every artifact must have `manifest.json` and `manifest.sig`.

**Enforcement:**
- Crypto enforcer validates all artifacts
- Verifies signature before runtime usage
- Build fails if artifact is unsigned

**Requirements:**
- Artifacts include: models, reports (PDF/HTML/CSV), archives, databases
- Manifest must contain: `hash`, `timestamp`, `version`, `signer`
- Signature must be RSA-4096 with PSS padding
- Trust chain must verify against Root CA

**Trust Domains:**
- `artifacts`: ML models, reports
- `agents`: Agent binaries, updates
- `updates`: System updates, patches
- `config`: Configuration files
- `reports`: Forensic reports, summaries

---

## 6. Data Retention Invariant

**Rule:** Retention policies must be enforced and validated.

**Enforcement:**
- Retention enforcer validates configuration
- Runtime enforcement via retention modules
- Build fails if configuration is invalid

**Policies:**
- **Telemetry:** 6 months default (max 7 years)
- **Forensics:** 10 days default (max 10 years)
- **AI Artifacts:** Minimum 2 years (cannot be deleted by disk pressure)
- **Disk Threshold:** 80% usage triggers retention enforcement

**Requirements:**
- `config/retention.txt` must exist and be valid
- Retention enforcement modules must exist
- Disk monitor must trigger purges
- All purges must be logged and signed

---

## 7. Offline-First Invariant

**Rule:** RansomEye must operate completely offline.

**Enforcement:**
- No internet calls during runtime (except explicit IOC API fetchers with offline cache)
- No CDN dependencies
- All models and data must be local

**Exceptions:**
- Threat Intel IOC fetchers (with mandatory offline cache)
- Initial setup/installation (one-time)

---

## 8. Runtime Determinism Invariant

**Rule:** Python runtime must be explicitly specified and validated. No reliance on `python` command.

**Enforcement:**
- Runtime check validates `python3` exists before any guardrail logic
- Minimum version requirement: Python 3.10+
- Build fails if `python3` is missing or version is insufficient
- All invocations use `python3` explicitly

**Requirements:**
- `python3` executable must be available in PATH
- Version must be >= 3.10
- Runtime check runs FIRST before any imports or logic
- No fallback to `python` command
- No auto-installation of Python runtime

**Supported Platforms:**
- Ubuntu 22.04+ (python3.10+)
- RHEL 8+ (python3.10+)

**Invocation:**
- **Canonical:** `python3 -m ransomeye_guardrails`
- **Direct:** `python3 -m ransomeye_guardrails.main`
- **Individual:** `python3 -m ransomeye_guardrails.<enforcer_name>`

**Fail Conditions:**
- `python3` command not found → Exit 1
- Python version < 3.10 → Exit 1
- Attempt to use `python -m` → Not supported (use `python3`)

**Implementation:**
- `ransomeye_guardrails/runtime_check.py` validates runtime
- `ransomeye_guardrails/__main__.py` enables module invocation
- Runtime check executes at import time in `main.py`

---

## 9. Fail-Closed Invariant

**Rule:** Any guardrail violation must fail the build immediately.

**Enforcement:**
- All enforcers use `fail_closed()` mechanism
- Exit code 1 on any violation
- Detailed violation reports to stderr

**No Exceptions:**
- No warnings-only mode
- No bypass mechanisms
- No "fix later" options

---

## Validation Process

0. **Runtime Check:** Validates python3 exists and version >= 3.10 (runs FIRST)
1. **Static Scanner:** Regex + AST pattern matching
2. **Header Enforcer:** File header validation
3. **ENV Enforcer:** Configuration validation
4. **ML Enforcer:** Model training + SHAP validation
5. **Crypto Enforcer:** Manifest + signature validation
6. **Retention Enforcer:** Policy configuration validation

All checks run in CI/CD pipeline (`ci/global_guardrails.yml`) using `python3` explicitly.

---

## Compliance

These invariants are **immutable** and **non-negotiable**. They form the foundation of RansomEye's security posture. Any attempt to bypass or weaken these guarantees will result in build failure.

**Last Updated:** Phase 0 Implementation  
**Enforcement Level:** Fail-Closed  
**Review Required:** Any changes to invariants require security architecture review

