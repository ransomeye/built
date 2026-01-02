# Backpressure Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/docs/backpressure_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Backpressure model documentation

---

## Overview

Backpressure is **explicit and deterministic**. No silent drops. No best-effort buffering.

---

## Backpressure Triggers

### Trigger 1: Buffer Full

**Condition:** Event buffer at capacity

**Response:**
- Reject event
- Signal backpressure to producer
- Explicit error message

---

### Trigger 2: Rate Limit Exceeded

**Condition:** Producer or global rate limit exceeded

**Response:**
- Reject event
- Signal backpressure to producer
- Explicit error message

---

### Trigger 3: System Overload

**Condition:** System resources exhausted

**Response:**
- Reject event
- Signal global backpressure
- Explicit error message

---

## Backpressure Signals

### Signal Format

```
BACKPRESSURE_ACTIVE
```

### Signal Duration

- Per-producer: Cleared after configured timeout
- Global: Cleared when system recovers

---

## Producer Response

Producers must:
- Respect backpressure signals
- Reduce send rate
- Retry with backoff
- Never ignore backpressure

---

## Last Updated

Phase 4 Implementation

