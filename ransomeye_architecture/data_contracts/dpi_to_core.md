# DPI Probe to Core Data Contract

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/data_contracts/dpi_to_core.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Data contract between DPI Probe (Data Plane) and Core Correlation Engine (Control Plane)

---

## Overview

This contract defines the **one-way data flow** from DPI Probe to Core Correlation Engine. All data must be signed and validated.

---

## Contract Properties

### Direction: One-Way

**Rule:** Data flows ONLY from DPI Probe → Core Correlation Engine

**Enforcement:**
- No reverse flow allowed
- No bidirectional communication
- No control messages from Core to DPI
- Enforced at API level

### Trust Level: Untrusted → Trusted

**Rule:** DPI Probe is untrusted, Core Correlation Engine is trusted

**Enforcement:**
- All data verified
- All signatures validated
- All identities checked
- Invalid data rejected

### Signing: Mandatory

**Rule:** All data must be signed with DPI Probe identity

**Enforcement:**
- Unsigned data rejected
- Invalid signature rejected
- Missing signature rejected
- Signature validation required

---

## Data Format

### Telemetry Message

```json
{
  "contract_version": "1.0.0",
  "message_id": "<unique-message-id>",
  "timestamp": "<ISO-8601-timestamp>",
  "nonce": "<unique-nonce>",
  "component_identity": "<dpi-probe-identity-hash>",
  "data": {
    "flow_id": "<flow-identifier>",
    "src_ip": "<source-ip>",
    "dst_ip": "<destination-ip>",
    "src_port": <source-port>,
    "dst_port": <destination-port>,
    "protocol": "<protocol>",
    "packet_count": <count>,
    "byte_count": <count>,
    "classification": "<ml-classification>",
    "metadata": {}
  },
  "signature": {
    "algorithm": "RSA-4096-PSS-SHA256",
    "signature": "<base64-encoded-signature>",
    "data_hash": "<SHA-256-hash>"
  }
}
```

### Required Fields

- `contract_version`: Contract version (MANDATORY - format: "MAJOR.MINOR.PATCH")
- `message_id`: Unique message identifier
- `timestamp`: Message timestamp (ISO-8601)
- `nonce`: Unique nonce for replay protection
- `component_identity`: DPI Probe identity hash
- `data`: Telemetry data
- `signature`: Cryptographic signature

### Versioning Rules

- **Current Version:** 1.0.0
- **Compatibility:** Only exact version match accepted (fail-closed)
- **Version Mismatch:** Reject message, terminate process, audit log
- **Schema Evolution:** Requires version bump (MAJOR for breaking, MINOR for additions, PATCH for fixes)
- **No Backward Compatibility:** Old versions are rejected

---

## Validation Rules

### Rule 1: Signature Validation

**Check:**
- Signature present
- Signature format valid
- Signature algorithm valid
- Signature matches data
- Signature matches component identity

**Failure:** Reject message, terminate DPI Probe, audit log

### Rule 2: Identity Validation

**Check:**
- Component identity present
- Identity valid
- Identity not revoked
- Identity has required permissions

**Failure:** Reject message, terminate DPI Probe, audit log

### Rule 3: Timestamp Validation

**Check:**
- Timestamp present
- Timestamp format valid
- Timestamp within window
- Timestamp not expired

**Failure:** Reject message, audit log

### Rule 4: Nonce Validation

**Check:**
- Nonce present
- Nonce format valid
- Nonce unique
- Nonce not replayed

**Failure:** Reject message, audit log

### Rule 5: Data Validation

**Check:**
- Data present
- Data format valid
- Data integrity valid
- Data hash matches

**Failure:** Reject message, audit log

### Rule 6: Version Validation (MANDATORY)

**Check:**
- `contract_version` field present
- Version format valid (MAJOR.MINOR.PATCH)
- Version matches expected version (1.0.0)
- No version mismatch

**Failure:** Reject message, terminate DPI Probe, audit log

**Enforcement:**
- Version check performed before any other validation
- Missing version → immediate rejection
- Version mismatch → immediate rejection
- No warnings, no degraded mode

---

## Rate Limiting

### Limit: 10,000 messages/second

**Enforcement:**
- Rate limit per DPI Probe instance
- Exceeding limit → reject excess messages
- Rate limit violation → audit log
- Rate limit violation → component warning

---

## Error Handling

### Error 1: Invalid Signature

**Response:**
- Reject message
- Terminate DPI Probe
- Revoke DPI Probe identity
- Audit log entry
- Human notification

### Error 2: Invalid Identity

**Response:**
- Reject message
- Terminate DPI Probe
- Revoke DPI Probe identity
- Audit log entry
- Human notification

### Error 3: Replay Attack

**Response:**
- Reject message
- Terminate DPI Probe
- Revoke DPI Probe identity
- Audit log entry
- Human notification

---

## Last Updated

Phase 2 Implementation

