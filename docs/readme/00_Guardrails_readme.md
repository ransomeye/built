# Phase 0: Global Guardrails Enforcement

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/00_Guardrails_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 0 - Global Guardrails Enforcement

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/ransomeye_guardrails/`
- **Type:** Python tool/library (no service)
- **Entry Point:** `main.py`

### Core Components

1. **Static Scanner** (`scanner.py`)
   - Scans codebase for hardcoded values
   - Validates against `rules.yaml`
   - Reports violations with fail-closed behavior

2. **Header Enforcer** (`header_enforcer.py`)
   - Validates file headers on all `.py`, `.yaml`, `.sh`, `.service`, `.tsx`, `.ts` files
   - Required format:
     ```
     # Path and File Name : /home/ransomeye/rebuild/<relative path>
     # Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
     # Details of functionality of this file: <short explanation>
     ```

3. **ENV Enforcer** (`env_enforcer.py`)
   - Detects hardcoded IPs, ports, paths, secrets
   - Enforces environment variable usage
   - Fail-closed on hardcoded credentials

4. **ML Enforcer** (`ml_enforcer.py`)
   - Validates model files exist (`.pkl`, `.gguf`)
   - Requires SHAP explainability files
   - Requires metadata files (hash, trained_on, version)
   - Rejects dummy/placeholder models

5. **Crypto Enforcer** (`crypto_enforcer.py`)
   - Validates cryptographic implementations
   - Enforces Ed25519 for signing (RSA prohibited in some contexts)
   - Validates key management

6. **Retention Enforcer** (`retention_enforcer.py`)
   - Validates retention configuration
   - Enforces <= 7 years retention
   - Validates disk cleanup policies

7. **Runtime Check** (`runtime_check.py`)
   - Validates runtime environment
   - Checks Python version, dependencies
   - Executes before all other checks

8. **Phase 6 Validator** (`phase6_validator.py`)
   - Specialized validator for Phase 6 (playbooks)
   - Validates playbook signatures
   - Validates playbook schema

### Rules Configuration
- **File:** `rules.yaml`
- **Schema:** `rules_schema.py`
- Defines allowed/disallowed patterns
- Configurable per module type

### Tests
- **Location:** `tests/`
- Unit tests for each enforcer
- Integration tests for full scan

---

## WHAT DOES NOT EXIST

1. **No systemd service** - Guardrails is a tool, not a service
2. **No database integration** - Pure static analysis
3. **No runtime enforcement** - Pre-build/pre-commit only
4. **No AI/ML models** - Rule-based only
5. **No UI** - CLI tool only

---

## DATABASE SCHEMAS

**NONE** - Phase 0 does not interact with databases.

---

## RUNTIME SERVICES

**NONE** - Phase 0 is a build-time tool, not a runtime service.

**Usage:**
```bash
python3 -m ransomeye_guardrails
```

---

## GUARDRAILS ALIGNMENT

Phase 0 **IS** the guardrails system. It enforces:

1. **File Headers** - All files must have unified headers
2. **No Hardcoding** - IPs, ports, paths, secrets must use environment variables
3. **Model Validation** - Real models only, SHAP required, metadata required
4. **Crypto Standards** - Ed25519 preferred, RSA prohibited in some contexts
5. **Retention Limits** - <= 7 years, disk cleanup policies
6. **Phantom Module Detection** - References to non-existent modules fail build

**Fail-Closed Behavior:**
- Any violation causes build failure
- No bypass mechanisms
- All checks must pass before build proceeds

---

## INSTALLER BEHAVIOR

**Not installed by installer** - Guardrails is a development tool.

**Build Integration:**
- Called by `install.sh` before installation proceeds
- Called by CI/CD pipelines
- Can be run manually: `python3 -m ransomeye_guardrails`

---

## SYSTEMD INTEGRATION

**NONE** - No systemd service for guardrails.

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 0 does not train or use AI/ML models. It validates that other phases have proper models.

**Validation Rules:**
- Models must exist on disk (`.pkl` or `.gguf`)
- SHAP explainability files required
- Metadata files required (hash, trained_on, version)
- Dummy/placeholder models rejected

---

## COPILOT REALITY

**NONE** - Phase 0 does not provide copilot functionality.

---

## UI REALITY

**NONE** - Phase 0 is CLI-only.

**Output:**
- Console output (pass/fail per check)
- Exit code (0 = pass, non-zero = fail)

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Any violation = build failure**
   - Hardcoded IP → FAIL
   - Missing header → FAIL
   - Dummy model → FAIL
   - Invalid signature → FAIL

2. **No bypass mechanisms**
   - No `--skip-checks` flag
   - No environment variable to disable
   - No configuration to ignore violations

3. **All checks must pass**
   - Static scanner: PASS
   - Header enforcer: PASS
   - ENV enforcer: PASS
   - ML enforcer: PASS
   - Crypto enforcer: PASS
   - Retention enforcer: PASS

4. **Runtime check executes first**
   - Validates Python version
   - Validates dependencies
   - Fails if runtime invalid

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 0 is fully implemented and production-ready:

✅ **Complete Implementation**
- All 6 enforcers implemented
- Rules configuration system complete
- Test coverage adequate
- Fail-closed behavior enforced

✅ **Integration**
- Integrated into `install.sh`
- Can be run standalone
- CI/CD ready

✅ **Guardrails Alignment**
- Enforces all project guardrails
- No bypass mechanisms
- Strict fail-closed behavior

**Limitations:**
- CLI-only (no UI)
- No runtime enforcement (build-time only)
- No database integration (static analysis only)

**Recommendation:** Deploy as-is. Phase 0 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
