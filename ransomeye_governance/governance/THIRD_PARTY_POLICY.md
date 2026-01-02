# Path: /home/ransomeye/rebuild/ransomeye_governance/governance/THIRD_PARTY_POLICY.md
# Author: RansomEye Core Team
# Purpose: Defines policy for third-party dependencies, including license compliance, security, and trust requirements

# RansomEye Third-Party Policy

## Overview

This policy governs the use of third-party dependencies, libraries, tools, and components in RansomEye. All third-party components must meet strict security, license, and trust requirements.

**This policy is enforced by code, not documentation.**

---

## General Principles

### Principle 1: Minimal Dependencies

- Minimize third-party dependencies
- Prefer standard library over external libraries
- Justify every dependency

### Principle 2: License Compliance

- All dependencies must comply with LICENSE_POLICY.md
- No GPL, AGPL, SSPL, or unknown licenses
- License must be explicitly identified

### Principle 3: Security Requirements

- Dependencies must be actively maintained
- Known vulnerabilities must be patched
- Security advisories must be monitored

### Principle 4: Trust Requirements

- Dependencies must be from trusted sources
- Signatures must be verified
- Provenance must be documented

---

## Allowed Sources

### Package Registries

- **Rust:** crates.io (official)
- **Python:** PyPI (official)
- **NPM:** npmjs.com (official)
- **System Packages:** OS vendor repositories only

### Forbidden Sources

- Unverified GitHub repositories
- Personal package repositories
- Unauthenticated downloads
- Binary-only distributions without source

---

## Dependency Requirements

### Requirement 1: License Identification

Every dependency must have:
- Explicit license declaration
- License file included
- License compatibility verified

### Requirement 2: Version Pinning

- Exact versions must be specified (no ranges)
- Versions must be locked in lockfiles
- Updates must be reviewed and tested

### Requirement 3: Security Scanning

- Dependencies scanned for known vulnerabilities
- CVEs checked against vulnerability databases
- Security patches applied promptly

### Requirement 4: Provenance

- Source code location documented
- Build artifacts signed
- SBOM (Software Bill of Materials) generated

---

## Prohibited Dependencies

### Prohibited Categories

1. **Copyleft Dependencies** - GPL, AGPL, SSPL
2. **Unmaintained Dependencies** - No updates in 12+ months
3. **Vulnerable Dependencies** - Known unpatched CVEs
4. **Unknown Source Dependencies** - Unverified origin
5. **Binary-Only Dependencies** - No source code available

### Prohibited Patterns

- Dependencies with unclear licensing
- Dependencies with security advisories
- Dependencies from untrusted sources
- Dependencies with transitive GPL dependencies

---

## Enforcement

### Build-Time Enforcement

1. **License Validation** - All dependencies checked against LICENSE_POLICY.md
2. **Security Scanning** - Vulnerability databases queried
3. **Provenance Verification** - Source and signatures verified

### CI Enforcement

1. **Dependency Audit** - Regular automated audits
2. **License Scanning** - CI blocks on license violations
3. **Security Scanning** - CI blocks on known vulnerabilities

### Runtime Enforcement

1. **Startup Validation** - System refuses to start if dependency policy violated
2. **Audit Logging** - All dependency checks logged

---

## SBOM Requirements

### Software Bill of Materials

Every build must generate:
- Complete dependency list
- License information
- Version information
- Source locations
- Signatures

SBOM format: SPDX 2.3 or CycloneDX

---

## Update Process

### Update Requirements

1. **Security Updates** - Applied immediately
2. **Feature Updates** - Reviewed and tested
3. **Breaking Changes** - Full regression testing required

### Update Approval

- All updates require review
- License compliance verified
- Security impact assessed
- Testing completed

---

## Compliance

This policy is enforced by:
- `ransomeye_governance/tooling/license_validator.py`
- `ransomeye_governance/tooling/provenance_generator.py`
- `ransomeye_governance/ci/license_scan.yml`
- `ransomeye_governance/ci/provenance.yml`

**Violation = Build Failure = CI Failure = Runtime Failure**

---

## References

- LICENSE_POLICY.md
- SECURITY_POLICY.md
- SUPPLY_CHAIN_SECURITY.md

