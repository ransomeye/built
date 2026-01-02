# RansomEye Phase 0 - Global Guardrails

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_guardrails/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 0 implementation summary and usage guide

---

## Overview

Phase 0 establishes **irreversible, fail-closed guardrails** that enforce security and compliance rules across all RansomEye phases. No future phase can violate these rules without causing build failure.

---

## Components

### 1. Guardrails Scanner (`scanner.py`)
- Static code scanner using regex and AST parsing
- Detects hardcoded IPs, URLs, secrets, paths
- Validates ML model requirements
- Enforces rules from `rules.yaml`

### 2. Header Enforcer (`header_enforcer.py`)
- Validates mandatory file headers
- Ensures all source files have required header format
- Excludes configuration and documentation files

### 3. ENV Enforcer (`env_enforcer.py`)
- Enforces ENV-only configuration
- Detects hardcoded values (IPs, ports, paths, secrets)
- Validates environment variable usage

### 4. ML Enforcer (`ml_enforcer.py`)
- Enforces train-from-scratch requirement
- Validates SHAP explainability
- Requires training scripts for all models
- Validates model metadata

### 5. Crypto Enforcer (`crypto_enforcer.py`)
- Validates manifest.json and manifest.sig for all artifacts
- Verifies cryptographic signatures
- Enforces trust domain hierarchy

### 6. Retention Enforcer (`retention_enforcer.py`)
- Validates retention policy configuration
- Ensures retention enforcement modules exist
- Validates `config/retention.txt`

### 7. Fail-Closed Mechanism (`fail_closed.py`)
- Central abort mechanism for violations
- Exits with code 1 on any violation
- Detailed violation reporting

---

## Usage

### Runtime Requirements

**MANDATORY:** `python3` must be installed and version >= 3.10

Supported platforms:
- Ubuntu 22.04+ (python3.10+)
- RHEL 8+ (python3.10+)

Runtime check runs automatically before any guardrail logic.

### Run All Guardrails Checks

**Canonical invocation:**
```bash
cd /home/ransomeye/rebuild
python3 -m ransomeye_guardrails
```

**Alternative:**
```bash
python3 -m ransomeye_guardrails.main
```

**Note:** Using `python` (without 3) is NOT supported and will fail.

### Run Individual Enforcers

```bash
# Static scanner
python3 -m ransomeye_guardrails.scanner --rules ransomeye_guardrails/rules.yaml

# Header enforcer
python3 -m ransomeye_guardrails.header_enforcer

# ENV enforcer
python3 -m ransomeye_guardrails.env_enforcer

# ML enforcer
python3 -m ransomeye_guardrails.ml_enforcer

# Crypto enforcer
python3 -m ransomeye_guardrails.crypto_enforcer

# Retention enforcer
python3 -m ransomeye_guardrails.retention_enforcer --dry-run
```

---

## Trust Infrastructure

Located in `/home/ransomeye/rebuild/ransomeye_trust/`:

- **Root CA Generator** (`root_ca_generator.py`): Generates offline Root CA and certificate hierarchy
- **Sign Tool** (`sign_tool.py`): RSA-4096 signer for artifacts
- **Verify Tool** (`verify_tool.py`): Signature verifier

### Initialize Trust Hierarchy

```bash
python3 -m ransomeye_trust.root_ca_generator --init
```

### Sign an Artifact

```bash
python3 -m ransomeye_trust.sign_tool <file> --manifest --version 1.0.0
```

### Verify an Artifact

```bash
python3 -m ransomeye_trust.verify_tool <file>
```

---

## Retention Enforcement

Located in `/home/ransomeye/rebuild/ransomeye_retention/`:

- **Retention Parser** (`retention_parser.py`): Parses `config/retention.txt`
- **Disk Monitor** (`disk_monitor.py`): Monitors disk usage and triggers purges
- **Telemetry Retention** (`telemetry_retention.py`): Purges old telemetry data
- **Forensic Retention** (`forensic_retention.py`): Purges old forensic evidence
- **AI Retention Guard** (`ai_retention_guard.py`): Protects AI artifacts from deletion

---

## CI/CD Integration

The CI pipeline (`ci/global_guardrails.yml`) runs all guardrails checks automatically on:
- Push to `main` or `develop` branches
- Pull requests
- Manual workflow dispatch

All checks must pass for the build to succeed.

---

## Configuration

### Retention Policy

Edit `/home/ransomeye/rebuild/config/retention.txt`:

```
TELEMETRY_RETENTION_MONTHS=6
FORENSIC_RETENTION_DAYS=10
DISK_MAX_USAGE_PERCENT=80
```

### Scanner Rules

Edit `/home/ransomeye/rebuild/ransomeye_guardrails/rules.yaml` to customize forbidden patterns.

---

## Documentation

- **Security Invariants:** `/home/ransomeye/rebuild/docs/SECURITY_INVARIANTS.md`
- **Data Retention Policy:** `/home/ransomeye/rebuild/docs/DATA_RETENTION_POLICY.md`

---

## Fail-Closed Enforcement

All guardrails use the **fail-closed** mechanism:
- Any violation immediately exits with code 1
- Detailed violation reports to stderr
- No warnings-only mode
- No bypass mechanisms

---

## Dependencies

Required Python packages:
- `pyyaml` - YAML parsing for rules
- `cryptography` - RSA-4096 signing/verification
- `psycopg2-binary` - PostgreSQL for telemetry retention

Install via:
```bash
python3 -m pip install pyyaml cryptography psycopg2-binary
```

---

## Acceptance Criteria

✅ Hardcoded secret causes build failure  
✅ Missing header causes build failure  
✅ Unsigned artifact causes build failure  
✅ ML model without SHAP causes build failure  
✅ Disk pressure simulation triggers retention logic  
✅ AI artifact deletion is BLOCKED  
✅ All failures are FAIL-CLOSED  
✅ `python3` runtime validated before any guardrail logic  
✅ `python` (without 3) invocation fails-closed  
✅ CI validates python3 version explicitly  

---

## Last Updated

Phase 0 Implementation - Complete

