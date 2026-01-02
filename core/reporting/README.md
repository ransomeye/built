# RansomEye Reporting, Forensics & Evidence Preservation

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_reporting/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 10 - Reporting, Forensics & Evidence Preservation module

---

## Overview

Phase 10 provides **immutable evidence preservation**, **forensic timelines**, and **regulatory-compliant reporting** for RansomEye. This phase is critical for audit compliance and legal defense.

---

## Features

- **Immutable Evidence Preservation**: Evidence bundles are sealed and cannot be modified
- **Hash Chaining**: Cryptographic hash chain ensures integrity
- **Cryptographic Signing**: Ed25519 signatures protect authenticity
- **Forensic Timelines**: Deterministic chronological event ordering
- **Multi-Format Exports**: PDF, HTML, and CSV reports
- **Reproducible Reports**: Reports can be regenerated from stored evidence
- **Retention Management**: Enforces retention policies with secure deletion
- **Corruption Detection**: Detects tampering and evidence corruption

---

## Architecture

### Core Components

- **EvidenceCollector**: Gathers evidence from various sources
- **EvidenceStore**: Immutable append-only evidence storage
- **EvidenceHasher**: Cryptographic hashing and hash chaining
- **ForensicTimeline**: Deterministic chronological event ordering
- **ReportBuilder**: Constructs reproducible reports
- **ReportExporter**: Exports reports in multiple formats
- **EvidenceVerifier**: Validates evidence integrity
- **RetentionManager**: Enforces retention policies

### Evidence Flow

1. **Collection**: Evidence is collected from sources
2. **Bundling**: Evidence is bundled with metadata
3. **Sealing**: Bundles are sealed (made immutable)
4. **Chaining**: Bundles are linked via hash chain
5. **Signing**: Bundles are cryptographically signed
6. **Storage**: Bundles are stored on disk
7. **Verification**: Bundles are verified for integrity
8. **Reporting**: Reports are generated from bundles
9. **Export**: Reports are exported in multiple formats

---

## Usage

### Building

```bash
cd /home/ransomeye/rebuild/ransomeye_reporting
cargo build --release
```

### Running Tests

```bash
cargo test
```

### CLI Usage

```bash
# Verify evidence store
./target/release/ransomeye_reporting verify /path/to/store

# Export report
./target/release/ransomeye_reporting export <report_id> /output/dir pdf

# Enforce retention
./target/release/ransomeye_reporting retention /path/to/store --dry-run
```

---

## Evidence Model

Evidence is preserved in **immutable bundles** that are:

- **Sealed**: Cannot be modified after sealing
- **Hash-Chained**: Each bundle references the previous bundle's hash
- **Signed**: Cryptographically signed with Ed25519
- **Versioned**: Includes engine and policy versions
- **Timestamped**: All timestamps in UTC

See `docs/evidence_model.md` for detailed documentation.

---

## Forensic Timelines

Forensic timelines provide **deterministic chronological ordering** of events with:

- **Source Attribution**: Every event includes source and source type
- **Kill-Chain Annotations**: Events tagged with MITRE ATT&CK stages
- **UTC Timestamps**: All timestamps in UTC (explicit timezone)
- **Reproducibility**: Same evidence always produces same timeline

See `docs/forensic_timeline.md` for detailed documentation.

---

## Reporting Formats

Reports are exported in **three formats**:

1. **PDF**: Formatted reports for printing and archival
2. **HTML**: Interactive web reports for viewing
3. **CSV**: Machine-readable data for analysis

All formats include:
- Report metadata (ID, versions, build hash)
- Evidence references (bundle IDs and hashes)
- Footer: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"
- Generation timestamp (UTC)

See `docs/reporting_formats.md` for detailed documentation.

---

## Compliance

RansomEye reporting features support:

- **GDPR**: Data retention, audit trail, secure deletion
- **HIPAA**: Audit controls, data integrity, secure deletion
- **SOC 2**: Security, availability, processing integrity
- **ISO 27001**: Information security management
- **NIST Cybersecurity Framework**: Identify, protect, detect, respond, recover
- **PCI DSS**: Data protection, access control, audit trail

See `docs/compliance_mapping.md` for detailed mapping.

---

## Failure Modes

All failures result in **fail-closed** behavior:

- **Evidence Corruption**: Report generation fails
- **Hash Mismatch**: Report generation fails
- **Missing Evidence**: Report generation fails
- **Broken Hash Chain**: Report generation fails
- **Signature Verification Failure**: Report generation fails

See `docs/failure_modes.md` for detailed documentation.

---

## Testing

Comprehensive tests validate:

- **Evidence Immutability**: Proves evidence cannot be modified after sealing
- **Hash Chain Integrity**: Validates hash chain integrity
- **Report Reproducibility**: Validates reports can be regenerated
- **Export Formats**: Validates PDF, HTML, and CSV exports
- **Corruption Detection**: Validates detection of tampering

Run tests with:

```bash
cargo test
```

---

## Schemas

JSON schemas are provided for:

- **Evidence Bundles**: `schemas/evidence_schema.json`
- **Forensic Reports**: `schemas/report_schema.json`
- **Forensic Timelines**: `schemas/timeline_schema.json`

---

## Documentation

Complete documentation is available in `docs/`:

- `evidence_model.md`: Evidence preservation model
- `forensic_timeline.md`: Forensic timeline construction
- `reporting_formats.md`: Report export formats
- `compliance_mapping.md`: Compliance framework mapping
- `failure_modes.md`: Failure modes and error handling

---

## Security

- **Immutable Evidence**: Once sealed, evidence cannot be modified
- **Hash Chaining**: Cryptographic chain ensures integrity
- **Cryptographic Signing**: Ed25519 signatures protect authenticity
- **Fail-Closed**: All failures result in report invalidation
- **Secure Deletion**: 3-pass overwrite for sensitive data

---

## License

Copyright © RansomEye.Tech  
Support: Gagan@RansomEye.Tech

