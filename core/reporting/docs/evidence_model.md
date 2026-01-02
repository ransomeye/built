# Evidence Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_reporting/docs/evidence_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Evidence model documentation - describes the immutable evidence preservation system

---

## Overview

The RansomEye evidence model provides **immutable, hash-chained, cryptographically signed** evidence preservation. Once evidence is sealed, it cannot be modified without detection.

---

## Core Principles

1. **Immutability**: Evidence bundles are sealed and cannot be modified
2. **Hash Chaining**: Each bundle references the previous bundle's hash
3. **Cryptographic Signing**: Bundles are signed with Ed25519 signatures
4. **Source Attribution**: All evidence includes explicit source and timestamp
5. **UTC Timestamps**: All timestamps are in UTC with explicit timezone

---

## Evidence Bundle Structure

An evidence bundle contains:

- **Bundle ID**: Unique UUID identifier
- **Created At**: Timestamp when bundle was created (UTC)
- **Sealed At**: Timestamp when bundle was sealed (UTC, null if not sealed)
- **Engine Version**: Version of RansomEye engine
- **Policy Version**: Version of active policy
- **Evidence Items**: Array of collected evidence
- **Bundle Hash**: SHA-256 hash of the bundle
- **Previous Bundle Hash**: Hash of previous bundle in chain (null for first)
- **Signature**: Ed25519 signature (base64 encoded, null if not signed)
- **Is Sealed**: Boolean indicating if bundle is immutable

---

## Evidence Item Structure

Each evidence item contains:

- **Evidence ID**: Unique UUID identifier
- **Source**: Source system identifier
- **Source Type**: Type of source (e.g., dpi_probe, linux_agent)
- **Timestamp**: When evidence was collected (UTC)
- **Kill Chain Stage**: MITRE ATT&CK stage (optional)
- **Data**: Evidence data (JSON structure)
- **Metadata**: Additional key-value metadata
- **Integrity Hash**: SHA-256 hash of the evidence item

---

## Hash Chaining

Evidence bundles form a hash chain:

```
Bundle 1 (hash: abc123...) -> Bundle 2 (prev_hash: abc123..., hash: def456...) -> Bundle 3 (prev_hash: def456..., hash: ghi789...)
```

This ensures:
- **Integrity**: Any modification breaks the chain
- **Ordering**: Chain order is cryptographically verifiable
- **Tampering Detection**: Missing or modified bundles are detected

---

## Sealing Process

1. Evidence is collected and added to bundle
2. Bundle hash is computed (SHA-256)
3. Bundle is cryptographically signed (Ed25519)
4. Bundle is saved to disk
5. Bundle is marked as sealed (immutable)

Once sealed, a bundle **cannot** be modified. Any attempt to modify a sealed bundle will be detected during verification.

---

## Verification

Evidence verification checks:

1. **Hash Integrity**: Bundle hash matches computed hash
2. **Signature Validity**: Cryptographic signature is valid
3. **Hash Chain**: Previous bundle hash references are valid
4. **Seal Status**: Bundles are properly sealed

If any check fails, the evidence store is considered **corrupted** and reports are **invalidated**.

---

## Failure Modes

- **Evidence Corruption**: Hash mismatch → Report Invalidated
- **Hash Mismatch**: Expected hash != computed hash → Report Invalidated
- **Missing Evidence**: Referenced bundle not found → Report Invalidated
- **Broken Chain**: Hash chain discontinuity → Report Invalidated

All failures result in **fail-closed** behavior: reports cannot be generated from corrupted evidence.

