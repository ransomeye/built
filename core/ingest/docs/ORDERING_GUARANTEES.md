# Ordering Guarantees

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ingestion/docs/ORDERING_GUARANTEES.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Explicit documentation of event ordering guarantees, gap handling, and reordering behavior

---

## Overview

RansomEye Ingestion provides **per-producer ordering guarantees** with explicit replay protection and gap detection. This document defines what is ordered, what is not ordered, how gaps are handled, and how reordering is handled.

---

## What IS Ordered

### Per-Producer Sequence Ordering

**Guarantee:** Events from the same producer are processed in strict sequence number order.

- **Scope:** Events are ordered **per producer ID**
- **Mechanism:** Sequence numbers must be monotonically increasing per producer
- **Enforcement:** Events with sequence numbers less than the expected sequence are rejected
- **Implementation:** `OrderingManager` tracks the expected sequence number per producer

### Example

For producer `producer_001`:
- Sequence 1 → ✅ Accepted (expected: 1)
- Sequence 2 → ✅ Accepted (expected: 2)
- Sequence 3 → ✅ Accepted (expected: 3)
- Sequence 2 → ❌ Rejected (expected: 4, got: 2 - out of order)

---

## What is NOT Ordered

### Cross-Producer Ordering

**No Guarantee:** Events from different producers are **NOT** ordered relative to each other.

- Events from `producer_001` and `producer_002` may be processed in any order
- No global ordering across producers
- No causal ordering guarantees between producers

### Example

```
Producer A, Sequence 1 (timestamp: 10:00:01)
Producer B, Sequence 1 (timestamp: 10:00:00)
```

These events may be processed in **any order** - Producer B's event may be processed before Producer A's event, even though it has an earlier timestamp.

---

## Gap Handling

### Small Gaps (Within Tolerance)

**Behavior:** Small sequence number gaps are **allowed** with a warning.

- **Maximum Gap:** 1000 sequence numbers (`max_sequence_gap`)
- **Action:** Event is accepted, warning is logged
- **Rationale:** Legitimate data loss or network issues may cause gaps

### Large Gaps (Beyond Tolerance)

**Behavior:** Large sequence number gaps are **allowed** with a **warning**.

- **Threshold:** Gap > 1000 sequence numbers
- **Action:** Event is accepted, warning is logged: "Large sequence number gap"
- **Rationale:** May indicate legitimate data loss, but should be investigated

### Example

```
Producer A, Sequence 1 → ✅ Accepted (expected: 1)
Producer A, Sequence 1005 → ✅ Accepted with WARNING (expected: 2, gap: 1003)
Producer A, Sequence 1006 → ✅ Accepted (expected: 1006)
```

**Note:** The system does **NOT** reject events due to gaps. Gaps are logged for monitoring but do not block processing.

---

## Reordering Behavior

### Sequence Number Regression (Replay Detection)

**Behavior:** Sequence numbers that are **less than** the last seen sequence are **rejected**.

- **Detection:** Sequence number < last seen sequence
- **Action:** Event is **rejected** with error: "Sequence number regression"
- **Rationale:** This indicates a replay attack or clock/time synchronization issues

### Example

```
Producer A, Sequence 10 → ✅ Accepted (last seen: 0)
Producer A, Sequence 5 → ❌ REJECTED (last seen: 10, regression detected)
```

---

## Timestamp Ordering

### Timestamp Tolerance

**Behavior:** Events with timestamps outside the tolerance window are **rejected**.

- **Tolerance:** ±5 minutes from current time
- **Detection:** `|event_timestamp - current_time| > 5 minutes`
- **Action:** Event is **rejected** with error: "Timestamp out of tolerance"
- **Rationale:** Prevents replay attacks and ensures events are recent

### Timestamp Regression (Per Producer)

**Behavior:** Events with timestamps **before** the last seen timestamp for the same producer are **rejected**.

- **Detection:** `event_timestamp < last_seen_timestamp` (same producer)
- **Action:** Event is **rejected** with error: "Timestamp regression"
- **Rationale:** Detects replay attacks and clock issues

### Example

```
Producer A, Sequence 1, Timestamp: 10:00:00 → ✅ Accepted
Producer A, Sequence 2, Timestamp: 09:59:00 → ❌ REJECTED (timestamp regression)
```

---

## Nonce-Based Replay Protection

### Duplicate Nonce Detection

**Behavior:** Events with duplicate nonces are **rejected**.

- **Scope:** Per-producer nonce cache
- **TTL:** 24 hours
- **Action:** Event is **rejected** with error: "Duplicate nonce"
- **Rationale:** Prevents exact event replay attacks

### Example

```
Producer A, Sequence 1, Nonce: "abc123" → ✅ Accepted
Producer A, Sequence 2, Nonce: "abc123" → ❌ REJECTED (duplicate nonce)
Producer A, Sequence 3, Nonce: "def456" → ✅ Accepted
```

---

## Summary Table

| Scenario | Action | Reason |
|----------|--------|--------|
| Sequence in order (same producer) | ✅ Accepted | Normal processing |
| Sequence out of order (same producer) | ❌ Rejected | Replay/regression detection |
| Sequence gap < 1000 | ✅ Accepted + Warning | Legitimate data loss |
| Sequence gap > 1000 | ✅ Accepted + Warning | Large gap (investigate) |
| Sequence regression | ❌ Rejected | Replay attack |
| Timestamp out of tolerance | ❌ Rejected | Replay attack |
| Timestamp regression | ❌ Rejected | Replay attack |
| Duplicate nonce | ❌ Rejected | Replay attack |
| Cross-producer ordering | No guarantee | Different producers |

---

## Implementation Details

### OrderingManager

The `OrderingManager` enforces per-producer ordering by:

1. Tracking expected sequence number per producer
2. Rejecting events with sequence < expected
3. Updating expected sequence after acceptance
4. Delegating to `ReplayProtector` for nonce and timestamp validation

### ReplayProtector

The `ReplayProtector` detects replay attacks by:

1. **Nonce Cache:** Tracks used nonces per producer (24-hour TTL)
2. **Timestamp Tolerance:** Rejects events > 5 minutes from current time
3. **Sequence Regression:** Rejects sequence numbers < last seen
4. **Timestamp Regression:** Rejects timestamps < last seen (per producer)

---

## Fail-Closed Behavior

**All ordering violations result in event rejection** - the system fails closed:

- Invalid sequence order → Reject
- Sequence regression → Reject
- Timestamp regression → Reject
- Duplicate nonce → Reject
- Timestamp out of tolerance → Reject

**No silent drops** - all rejections are logged and the producer receives an explicit error response.

---

**Last Updated:** 2025-12-22  
**Version:** 1.0

