# RansomEye Event Ingestion

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 4 - Core Event Ingestion & Deterministic Backpressure

---

## Overview

Phase 4 builds the **ONLY ingress point** into the RansomEye Control Plane. All incoming data is untrusted, potentially malicious, and must be strictly validated.

**NO correlation. NO policy. NO AI. NO inference.**

---

## Key Components

### 1. Event Listener

**Location:** `src/listener.rs`

**Function:** Receives events from producers and processes them through the ingestion pipeline.

**Pipeline:**
1. Authentication
2. Signature Verification
3. Schema Validation
4. Rate Limiting
5. Backpressure Check
6. Ordering Check
7. Buffer Add
8. Dispatch

---

### 2. Authentication

**Location:** `src/auth.rs`

**Function:** Verifies producer identity, expiration, and revocation.

**Checks:**
- Producer ID valid
- Identity certificate valid
- Identity not revoked
- Identity not expired
- Component type valid

---

### 3. Signature Verification

**Location:** `src/signature.rs`

**Function:** Verifies cryptographic signatures on all events.

**Algorithm:** RSA-4096-PSS-SHA256

**Checks:**
- Signature present
- Signature format valid
- Signature matches producer identity
- Signature matches event data

---

### 4. Schema Validation

**Location:** `src/schema.rs`

**Function:** Strictly validates event schemas.

**Rules:**
- Schema version compatible
- Required fields present
- Field types correct
- No permissive parsing

---

### 5. Rate Limiting

**Location:** `src/rate_limit.rs`

**Function:** Enforces deterministic rate limits.

**Types:**
- Per-producer limits
- Per-component quotas
- Global caps

**Method:** Fixed windows, deterministic counters

---

### 6. Backpressure

**Location:** `src/backpressure.rs`

**Function:** Manages explicit backpressure signals.

**Triggers:**
- Buffer full
- Rate limit exceeded
- System overload

**Response:** Explicit rejection + backpressure signal

---

### 7. Buffer Management

**Location:** `src/buffer.rs`

**Function:** Manages bounded event buffering.

**Properties:**
- Bounded capacity
- Explicit rejection when full
- No silent drops

---

### 8. Ordering

**Location:** `src/ordering.rs`

**Function:** Ensures per-producer event ordering.

**Checks:**
- Sequence number in order
- No replay detected
- Nonce not seen before
- Timestamp within tolerance

---

## Hard Rules Enforced

1. **No unsigned event may pass** - Signature verification required
2. **No schema violation may pass** - Strict schema validation
3. **No unbounded buffer allowed** - Bounded buffers only
4. **No silent drops** - Explicit rejection messages
5. **No implicit trust** - Authentication required
6. **No AI/ML/LLM usage** - Pure ingestion only
7. **Backpressure must be explicit** - Explicit signals
8. **Any ambiguity → REJECT** - Fail-closed

---

## Failure Modes

All failures are **fail-closed**:

1. **Producer Flooding** → Reject + Backpressure
2. **Invalid Signatures** → Reject + Audit Log
3. **Schema Mismatches** → Reject + Audit Log
4. **Buffer Exhaustion** → Reject + Backpressure
5. **Replay Attacks** → Reject + Audit Log
6. **Network Partitions** → Buffer (if capacity) or Reject
7. **Downstream Unavailable** → Buffer (if capacity) or Reject

---

## Testing

### Rust Tests

```bash
cd /home/ransomeye/rebuild/ransomeye_ingestion/tests
cargo test
```

**Test Suites:**
- `auth_failure_tests.rs` - Authentication failures
- `signature_failure_tests.rs` - Signature failures
- `schema_rejection_tests.rs` - Schema violations
- `overload_tests.rs` - System overload
- `replay_attack_tests.rs` - Replay attacks

---

## Configuration

Environment Variables:
- `RANSOMEYE_INGESTION_LISTEN_ADDR` - Listen address (default: 0.0.0.0:8080)
- `RANSOMEYE_CONTROL_PLANE_ADDR` - Control Plane address (default: 127.0.0.1:9090)
- `RANSOMEYE_BUFFER_CAPACITY` - Buffer capacity (default: 10000)
- `RANSOMEYE_PRODUCER_RATE_LIMIT` - Per-producer limit (default: 1000)
- `RANSOMEYE_GLOBAL_RATE_LIMIT` - Global limit (default: 10000)
- `RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS` - Rate limit window (default: 60)

---

## Key Guarantees

1. **No Unsigned Events** - All events must be signed
2. **No Schema Violations** - Strict schema validation
3. **No Silent Drops** - Explicit rejection messages
4. **No Implicit Trust** - Authentication required
5. **Explicit Backpressure** - Deterministic signals
6. **Fail-Closed** - All failures result in rejection

---

## Last Updated

Phase 4 Implementation - Complete

