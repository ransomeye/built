# Phase 4: Event Ingestion & Telemetry

**Path and File Name:** `/home/ransomeye/rebuild/docs/readme/04_Ingestion_Telemetry_readme.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Forensic-grade documentation for Phase 4 - Event Ingestion & Telemetry

---

## WHAT EXISTS

### Implementation Location
- **Directory:** `/home/ransomeye/rebuild/core/ingest/`
- **Service:** `systemd/ransomeye-ingestion.service`
- **Binary:** Built from Rust crate

### Core Components

1. **Ingestion Server** (`src/server.rs`)
   - Main server orchestrator
   - Handles incoming event streams
   - Coordinates all ingestion components

2. **Event Listener** (`src/listener.rs`)
   - Receives events from producers
   - Processes through ingestion pipeline
   - Pipeline: Auth → Signature → Schema → Rate Limit → Backpressure → Ordering → Buffer → Dispatch

3. **Authentication** (`src/auth.rs`)
   - Verifies producer identity
   - Checks identity expiration
   - Validates revocation status
   - Validates component type

4. **Signature Verification** (`src/signature.rs`)
   - Verifies RSA-4096-PSS-SHA256 signatures
   - Validates signature format
   - Matches signature to producer identity
   - Matches signature to event data

5. **Schema Validation** (`src/schema.rs`)
   - Strict schema validation
   - Version compatibility checks
   - Required fields validation
   - No permissive parsing

6. **Rate Limiting** (`src/rate_limit.rs`)
   - Per-producer limits
   - Per-component quotas
   - Global caps
   - Fixed windows, deterministic counters

7. **Backpressure** (`src/backpressure.rs`)
   - Deterministic backpressure signals
   - Bounded buffer management
   - Producer notification
   - Fail-closed on buffer overflow

8. **Event Ordering** (`src/ordering.rs`)
   - Sequence number validation
   - Out-of-order detection
   - Gap detection
   - Reordering logic

9. **Event Buffer** (`src/buffer.rs`)
   - Bounded in-memory buffer
   - Persistent disk buffer (when needed)
   - Buffer overflow handling

10. **Event Dispatcher** (`src/dispatcher.rs`)
    - Routes events to correlation engine
    - Maintains event ordering
    - Handles dispatch failures

11. **Event Normalization** (`src/normalization.rs`)
    - Normalizes event formats
    - Standardizes field names
    - Validates data types

12. **Protocol** (`protocol/`)
    - Event envelope definitions
    - Protocol versioning
    - Compatibility rules

---

## WHAT DOES NOT EXIST

1. **No database persistence in ingestion** - Ingestion does not write to database directly
2. **No correlation logic** - Correlation handled by Phase 5
3. **No policy evaluation** - Policy handled by Phase 6
4. **No AI/ML inference** - AI handled by Phase 8
5. **No enforcement** - Enforcement handled by Phase 7

---

## DATABASE SCHEMAS

**NONE** - Phase 4 does not create database tables.

**Event Storage:**
- Events buffered in memory
- Events dispatched to correlation engine
- Correlation engine handles persistence

---

## RUNTIME SERVICES

**Service:** `ransomeye-ingestion.service`
- **Location:** `/home/ransomeye/rebuild/systemd/ransomeye-ingestion.service`
- **User:** `ransomeye`
- **Group:** `ransomeye`
- **Restart:** `always`
- **Dependencies:** `network.target`, `ransomeye-core.service`
- **ExecStart:** `/usr/bin/ransomeye_operations start ransomeye-ingestion`

**Service Configuration:**
- Rootless runtime (User=ransomeye)
- Capabilities: CAP_NET_BIND_SERVICE, CAP_NET_RAW, CAP_SYS_NICE
- ReadWritePaths: /home/ransomeye/rebuild, /var/lib/ransomeye/ingestion, /run/ransomeye/ingestion

---

## GUARDRAILS ALIGNMENT

Phase 4 enforces guardrails:

1. **Signed Events Only** - Unsigned events rejected
2. **Validated Identity** - Invalid identity rejected
3. **Schema Validation** - Invalid schema rejected
4. **Rate Limiting** - Rate limit exceeded → Reject
5. **Backpressure** - Buffer full → Backpressure signal
6. **Fail-Closed** - Any validation failure → Reject event

---

## INSTALLER BEHAVIOR

**Installation:**
- Ingestion service installed by main installer
- Service file created in `/home/ransomeye/rebuild/systemd/`
- Service disabled by default
- Binary built from Rust crate

---

## SYSTEMD INTEGRATION

**Service File:**
- Created by installer
- Located in unified systemd directory
- Rootless configuration
- Restart always
- Disabled by default

---

## AI/ML/LLM TRAINING REALITY

**NONE** - Phase 4 does not use AI/ML models.

**Ingestion is pure validation and routing:**
- No inference
- No model loading
- No AI dependencies

---

## COPILOT REALITY

**NONE** - Phase 4 does not provide copilot functionality.

**Events are routed to:**
- Correlation engine (Phase 5)
- Policy engine (Phase 6)
- AI advisory (Phase 8)

---

## UI REALITY

**NONE** - Phase 4 has no UI.

**Ingestion metrics:**
- Available via systemd journal
- Logged events
- Rate limit status
- Backpressure status

---

## FAIL-CLOSED BEHAVIOR

**STRICT FAIL-CLOSED:**

1. **Invalid Identity** → Event REJECTED
2. **Invalid Signature** → Event REJECTED
3. **Invalid Schema** → Event REJECTED
4. **Rate Limit Exceeded** → Event REJECTED
5. **Buffer Overflow** → Backpressure signal, event may be dropped
6. **Out-of-Order** → Event REJECTED or buffered for reordering

**No Bypass:**
- No `--skip-validation` flag
- No `--skip-signature` flag
- All checks must pass

---

## FINAL VERDICT

**PRODUCTION-VIABLE**

Phase 4 is fully implemented and production-ready:

✅ **Complete Implementation**
- Ingestion server functional
- All validation components implemented
- Rate limiting working
- Backpressure handling working
- Event ordering working
- Protocol versioning complete

✅ **Guardrails Alignment**
- Signed events only
- Validated identity
- Schema validation
- Rate limiting
- Fail-closed behavior

✅ **Fail-Closed Behavior**
- All validation checks fail-closed
- No bypass mechanisms
- Events rejected on any failure

**Recommendation:** Deploy as-is. Phase 4 meets all requirements and is production-ready.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech
