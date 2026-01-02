# Core to AI Data Contract

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_architecture/data_contracts/core_to_ai.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Data contract between Control Plane and Intelligence Plane - read-only access

---

## Overview

This contract defines the **one-way, read-only data flow** from Control Plane to Intelligence Plane. AI has zero enforcement authority.

---

## Contract Properties

### Direction: One-Way

**Rule:** Data flows ONLY from Control Plane → Intelligence Plane

**Enforcement:**
- No reverse flow allowed
- No bidirectional communication
- No data from AI to Control Plane
- Enforced at API level

### Trust Level: Trusted → Advisory

**Rule:** Control Plane is trusted, Intelligence Plane is advisory

**Enforcement:**
- Read-only access only
- No write operations
- No state modification
- No enforcement authority

### Access: Read-Only

**Rule:** Intelligence Plane has read-only access to Control Plane data

**Enforcement:**
- Read-only APIs only
- No write functions accessible
- No state modification functions accessible
- No enforcement functions accessible

---

## Data Format

### Analysis Request

```json
{
  "contract_version": "1.0.0",
  "request_id": "<unique-request-id>",
  "timestamp": "<ISO-8601-timestamp>",
  "component_identity": "<control-plane-identity-hash>",
  "data": {
    "correlated_events": [],
    "threat_graph": {},
    "policy_violations": [],
    "metadata": {}
  },
  "analysis_type": "<analysis-type>",
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
- `component_identity`: Control Plane identity hash
- `data`: Read-only data for analysis
- `analysis_type`: Type of analysis requested
- `signature`: Cryptographic signature

### Versioning Rules

- **Current Version:** 1.0.0
- **Compatibility:** Only exact version match accepted (fail-closed)
- **Version Mismatch:** Reject request, terminate AI component, audit log
- **Schema Evolution:** Requires version bump (MAJOR for breaking, MINOR for additions, PATCH for fixes)
- **No Backward Compatibility:** Old versions are rejected

---

## Validation Rules

### Rule 1: Read-Only Access

**Check:**
- No write operations allowed
- No state modification allowed
- No enforcement functions accessible
- Read-only APIs only

**Failure:** Reject request, terminate AI component, audit log

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
- Identity has required permissions

**Failure:** Reject request, audit log

---

## Response Format

### Analysis Response

```json
{
  "response_id": "<unique-response-id>",
  "request_id": "<original-request-id>",
  "timestamp": "<ISO-8601-timestamp>",
  "component_identity": "<ai-component-identity-hash>",
  "advisory": true,
  "data": {
    "recommendations": [],
    "risk_scores": {},
    "explanations": {},
    "shap_values": {}
  },
  "signature": {
    "algorithm": "RSA-4096-PSS-SHA256",
    "signature": "<base64-encoded-signature>",
    "data_hash": "<SHA-256-hash>"
  }
}
```

### Required Fields

- `response_id`: Unique response identifier
- `request_id`: Original request identifier
- `timestamp`: Response timestamp (ISO-8601)
- `component_identity`: AI component identity hash
- `advisory`: Must be true (advisory only)
- `data`: Advisory recommendations
- `signature`: Cryptographic signature

---

## Forbidden Operations

### Operation 1: Write Operations

**Rule:** AI cannot perform write operations

**Enforcement:**
- No write APIs accessible
- No database writes allowed
- No state modification allowed
- No configuration changes allowed

### Operation 2: Enforcement Operations

**Rule:** AI cannot authorize enforcement

**Enforcement:**
- No enforcement APIs accessible
- No policy modification allowed
- No action dispatch allowed
- No state changes allowed

### Operation 3: Reverse Flow

**Rule:** AI cannot send data to Control Plane

**Enforcement:**
- No API endpoints accept AI data
- No function calls from AI to Control
- No communication from AI to Control
- Enforced at interface level

---

## Error Handling

### Error 1: Write Operation Attempt

**Response:**
- Reject operation
- Terminate AI component
- Revoke AI component identity
- Audit log entry
- Human notification

### Error 2: Enforcement Attempt

**Response:**
- Reject operation
- Terminate AI component
- Revoke AI component identity
- Audit log entry
- Human notification

### Error 3: Reverse Flow Attempt

**Response:**
- Reject operation
- Terminate AI component
- Revoke AI component identity
- Audit log entry
- Human notification

---

## Last Updated

Phase 2 Implementation

