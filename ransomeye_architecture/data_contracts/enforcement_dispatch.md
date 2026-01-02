# Enforcement Dispatch Data Contract

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/data_contracts/enforcement_dispatch.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Data contract for enforcement dispatch - only Control Plane can authorize

---

## Overview

This contract defines the **enforcement dispatch process**. Only Control Plane can authorize enforcement. All other planes are forbidden.

---

## Contract Properties

### Direction: One-Way

**Rule:** Enforcement requests flow ONLY from Control Plane → Enforcement Dispatcher

**Enforcement:**
- No reverse flow allowed
- No bidirectional communication
- No enforcement from other planes
- Enforced at API level

### Trust Level: Authoritative → Executor

**Rule:** Control Plane is authoritative, Enforcement Dispatcher is executor

**Enforcement:**
- Only Control Plane can authorize
- All requests must be signed
- All requests must be validated
- Unauthorized requests rejected

### Authorization: Mandatory

**Rule:** All enforcement requests must be authorized by Control Plane

**Enforcement:**
- Authorization check required
- Signature validation required
- Policy validation required
- Request validation required

---

## Data Format

### Enforcement Request

```json
{
  "contract_version": "1.0.0",
  "request_id": "<unique-request-id>",
  "timestamp": "<ISO-8601-timestamp>",
  "nonce": "<unique-nonce>",
  "component_identity": "<control-plane-identity-hash>",
  "authorization": {
    "authorized_by": "<control-plane-component>",
    "authorization_timestamp": "<ISO-8601-timestamp>",
    "policy_reference": "<policy-id>",
    "authorization_signature": "<signature>"
  },
  "action": {
    "type": "<action-type>",
    "target": "<target-identifier>",
    "parameters": {}
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
- `request_id`: Unique request identifier
- `timestamp`: Request timestamp (ISO-8601)
- `nonce`: Unique nonce for replay protection
- `component_identity`: Control Plane identity hash
- `authorization`: Authorization details
- `action`: Enforcement action details
- `signature`: Cryptographic signature

### Versioning Rules

- **Current Version:** 1.0.0
- **Compatibility:** Only exact version match accepted (fail-closed)
- **Version Mismatch:** Reject request, terminate process, audit log
- **Schema Evolution:** Requires version bump (MAJOR for breaking, MINOR for additions, PATCH for fixes)
- **No Backward Compatibility:** Old versions are rejected

---

## Validation Rules

### Rule 1: Authorization Validation

**Check:**
- Authorization present
- Authorized by Control Plane
- Authorization signature valid
- Policy reference valid
- Authorization timestamp valid

**Failure:** Reject request, audit log

### Rule 2: Signature Validation

**Check:**
- Signature present
- Signature format valid
- Signature algorithm valid
- Signature matches data
- Signature matches component identity

**Failure:** Reject request, audit log

### Rule 3: Identity Validation

**Check:**
- Component identity present
- Identity valid
- Identity not revoked
- Identity has enforcement authorization

**Failure:** Reject request, audit log

### Rule 4: Policy Validation

**Check:**
- Policy reference present
- Policy exists
- Policy valid
- Policy allows action

**Failure:** Reject request, audit log

---

## Forbidden Sources

### Source 1: Data Plane

**Rule:** Data Plane cannot authorize enforcement

**Enforcement:**
- No API endpoints accept Data Plane requests
- No function calls from Data Plane
- No communication from Data Plane
- Enforced at interface level

### Source 2: Intelligence Plane

**Rule:** Intelligence Plane cannot authorize enforcement

**Enforcement:**
- No API endpoints accept Intelligence Plane requests
- No function calls from Intelligence Plane
- No communication from Intelligence Plane
- Enforced at interface level

### Source 3: Management Plane

**Rule:** Management Plane cannot authorize enforcement directly

**Enforcement:**
- Management Plane must go through Control Plane
- No direct enforcement authorization
- No bypass of Control Plane
- Enforced at interface level

---

## Error Handling

### Error 1: Unauthorized Source

**Response:**
- Reject request
- Terminate source component
- Revoke source component identity
- Audit log entry
- Human notification

### Error 2: Invalid Authorization

**Response:**
- Reject request
- Audit log entry
- Human notification

### Error 3: Invalid Policy

**Response:**
- Reject request
- Audit log entry
- Human notification

---

## Last Updated

Phase 2 Implementation

