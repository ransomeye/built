# Path: /home/ransomeye/rebuild/ransomeye_governance/governance/SUPPLY_CHAIN_SECURITY.md
# Author: RansomEye Core Team
# Purpose: Defines supply chain security requirements including reproducible builds, provenance, SBOM, and signed artifacts

# RansomEye Supply Chain Security Policy

## Overview

This policy defines supply chain security requirements for RansomEye, including reproducible builds, build provenance, SBOM generation, signed release manifests, and CI identity attestation.

**This policy is enforced by code, not documentation.**

---

## Reproducible Builds

### Requirement 1: Deterministic Builds

- Builds MUST be reproducible
- Same source + same dependencies = same binary
- Build environment documented
- Build tools versioned

### Requirement 2: Build Environment

- Build environment MUST be containerized
- Build environment MUST be versioned
- Build environment MUST be documented
- Build environment MUST be reproducible

### Requirement 3: Dependency Locking

- All dependencies MUST be locked
- Lockfiles MUST be committed
- Dependency versions MUST be pinned

---

## Build Provenance

### Requirement 1: Provenance Generation

Every build MUST generate:
- **Build provenance document** - Complete build information
- **Source code hash** - Hash of source code used
- **Dependency manifest** - Complete dependency list
- **Build environment** - Build environment details
- **Build timestamp** - Build time and date
- **Builder identity** - CI system identity

### Requirement 2: Provenance Format

- Provenance in SLSA Provenance format (v0.2+)
- JSON-LD format
- Cryptographically signed
- Verifiable

### Requirement 3: Provenance Storage

- Provenance stored with artifacts
- Provenance accessible for verification
- Provenance archived for audit

---

## SBOM (Software Bill of Materials)

### Requirement 1: SBOM Generation

Every build MUST generate:
- **Complete dependency list** - All dependencies
- **License information** - All licenses
- **Version information** - All versions
- **Source locations** - All source URLs
- **Vulnerability information** - Known CVEs

### Requirement 2: SBOM Format

- SBOM in SPDX 2.3 or CycloneDX format
- JSON format
- Machine-readable
- Human-readable

### Requirement 3: SBOM Distribution

- SBOM included with release
- SBOM accessible for download
- SBOM archived for audit

---

## Signed Release Manifests

### Requirement 1: Manifest Generation

Every release MUST include:
- **Release manifest** - Complete release information
- **Artifact list** - All artifacts in release
- **Checksums** - SHA-256 checksums for all artifacts
- **Signatures** - Cryptographic signatures for all artifacts
- **Provenance** - Build provenance for all artifacts

### Requirement 2: Manifest Signing

- Manifest signed with release key
- Signature verified before distribution
- Signature chain validated

### Requirement 3: Manifest Distribution

- Manifest included with release
- Manifest accessible for verification
- Manifest archived for audit

---

## Artifact Signing

### Requirement 1: All Artifacts Signed

Every artifact MUST be signed:
- **Binaries** - All executables signed
- **Libraries** - All libraries signed
- **Models** - All ML models signed
- **Configurations** - All config files signed
- **Documentation** - All docs signed (optional)

### Requirement 2: Signing Keys

- Signing keys stored securely
- Signing keys rotated regularly
- Signing key access restricted
- Signing key compromise = key rotation

### Requirement 3: Signature Verification

- Signatures verified before use
- Signature chain validated
- Invalid signature = **ARTIFACT REJECTED**

---

## CI Identity Attestation

### Requirement 1: CI Identity

- CI system identity verified
- CI system authenticated
- CI system authorized

### Requirement 2: Build Attestation

- Builds attested by CI system
- Attestation includes:
  - CI system identity
  - Build environment
  - Build inputs
  - Build outputs

### Requirement 3: Attestation Verification

- Attestations verified before trust
- Attestation chain validated
- Invalid attestation = **BUILD REJECTED**

---

## Enforcement

### Build-Time Enforcement

1. **Provenance Generation** - `tooling/provenance_generator.py` generates provenance
2. **SBOM Generation** - SBOM generated automatically
3. **Artifact Signing** - `tooling/artifact_signer.py` signs all artifacts

### CI Enforcement

1. **Provenance Verification** - CI verifies provenance
2. **SBOM Validation** - CI validates SBOM
3. **Signature Verification** - CI verifies signatures

### Runtime Enforcement

1. **Artifact Verification** - Artifacts verified before use
2. **Signature Validation** - Signatures validated at runtime
3. **Audit Logging** - All verification events logged

---

## Compliance

This policy is enforced by:
- `ransomeye_governance/tooling/provenance_generator.py`
- `ransomeye_governance/tooling/artifact_signer.py`
- `ransomeye_governance/ci/provenance.yml`
- `ransomeye_governance/ci/artifact_signing.yml`

**Violation = Build Failure = CI Failure = Runtime Failure**

---

## References

- LICENSE_POLICY.md
- SECURITY_POLICY.md
- THIRD_PARTY_POLICY.md

