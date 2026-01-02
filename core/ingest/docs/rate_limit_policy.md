# Rate Limit Policy

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/docs/rate_limit_policy.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Rate limiting policy documentation

---

## Overview

Rate limiting is **deterministic** with fixed windows and explicit counters. No adaptive heuristics.

---

## Rate Limit Types

### Type 1: Per-Producer Limit

**Limit:** 1000 events per window (default)

**Window:** 60 seconds (default)

**Enforcement:**
- Fixed window
- Deterministic counter
- Reset on window expiration

---

### Type 2: Per-Component Quota

**Limit:** Component-specific quota

**Window:** 60 seconds (default)

**Enforcement:**
- Fixed window
- Deterministic counter
- Reset on window expiration

---

### Type 3: Global Cap

**Limit:** 10000 events per window (default)

**Window:** 60 seconds (default)

**Enforcement:**
- Fixed window
- Deterministic counter
- Reset on window expiration

---

## Limit Exceeded Response

When limit is exceeded:
- Event rejected
- Explicit error message
- Backpressure signal
- Audit log entry

---

## Configuration

Rate limits are configurable via environment variables:
- `RANSOMEYE_PRODUCER_RATE_LIMIT`
- `RANSOMEYE_GLOBAL_RATE_LIMIT`
- `RANSOMEYE_RATE_LIMIT_WINDOW_SECONDS`

---

## Last Updated

Phase 4 Implementation

