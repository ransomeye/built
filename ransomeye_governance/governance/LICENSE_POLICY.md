# Path: /home/ransomeye/rebuild/ransomeye_governance/governance/LICENSE_POLICY.md
# Author: RansomEye Core Team
# Purpose: Defines absolute license policy for RansomEye - fail-closed enforcement of allowed/banned licenses

# RansomEye License Policy

## Overview

RansomEye is a **commercial, closed-source, enterprise-grade cybersecurity platform**. This policy defines **non-negotiable license rules** enforced at build time, CI time, and runtime.

**This policy is enforced by code, not documentation.**

---

## ALLOWED LICENSES (ONLY)

The following licenses are **explicitly allowed**:

### Permissive Licenses

- **MIT License** - Full permission
- **BSD 2-Clause License** - Full permission
- **BSD 3-Clause License** - Full permission
- **Apache 2.0 License** - Full permission
- **PSF License** (Python Software Foundation) - Full permission
- **CC0** (Public Domain Dedication) - Full permission

### Conditionally Allowed

- **LGPL** - ONLY if:
  - Dynamically linked (not statically linked)
  - Isolated in separate process/container
  - Replaceable without recompiling RansomEye core
  - Documented in dependency manifest

---

## BANNED LICENSES (ABSOLUTE)

The following licenses are **ABSOLUTELY FORBIDDEN**:

### Copyleft Licenses

- **GPL v2** - BANNED
- **GPL v3** - BANNED
- **AGPL (GNU Affero General Public License)** - BANNED
- **SSPL (Server Side Public License)** - BANNED

### Proprietary Restrictions

- **Elastic License** - BANNED
- **MongoDB SSPL** - BANNED

### Unknown/Unclear

- **Unknown licenses** - BANNED
- **Custom licenses** - BANNED (unless explicitly approved)
- **Unclear or ambiguous licenses** - BANNED

### Content Restrictions

- **GPL rules, signatures, datasets, or content** - BANNED (even if engine is permissive)

---

## Enforcement

### Build-Time Enforcement

1. **License Scanner** (`tooling/license_validator.py`) must run before any build
2. **CI Pipeline** (`ci/license_scan.yml`) must block on violation
3. **Dependency Check** - All dependencies scanned on `cargo build`, `pip install`, `npm install`

### Runtime Enforcement

1. **Startup Validation** - System refuses to start if banned license detected
2. **Audit Logging** - All license checks logged with timestamps

### Failure Behavior

**FAIL-CLOSED:** Any banned license detection results in:
- Build failure
- CI pipeline failure
- Runtime startup failure
- Explicit error message identifying the violation

---

## Validation Rules

### Rule 1: Dependency Scanning

Every dependency (Rust crate, Python package, npm package) must be scanned.

### Rule 2: Transitive Dependencies

Transitive dependencies are also subject to this policy.

### Rule 3: License Files

All LICENSE, COPYING, LICENSE.txt files must be checked.

### Rule 4: Source Code Headers

License headers in source code must comply.

---

## Exceptions

**NO EXCEPTIONS** without explicit written approval from RansomEye Core Team and legal review.

---

## Compliance

This policy is enforced by:
- `ransomeye_governance/tooling/license_validator.py`
- `ransomeye_governance/ci/license_scan.yml`
- `ransomeye_governance/tests/license_violation_tests.py`

**Violation = Build Failure = CI Failure = Runtime Failure**

---

## References

- RansomEye Phase 1 Specification
- RansomEye Global Architecture Rules
- RansomEye Supply Chain Security Policy

