# Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_core/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Failure mode definitions and handling

## Overview

The correlation engine defines explicit behavior for all failure modes, ensuring correctness, avoiding false detections, and maintaining observability.

## Failure Modes

### 1. Event Floods

**Scenario:** Sudden spike in event volume exceeding capacity.

**Behavior:**
- Events beyond capacity are dropped
- Dropped events logged for audit
- Backpressure signaled to Phase 4
- Engine continues processing within capacity

**Preserves Correctness:** Yes - bounded memory prevents corruption

**Avoids False Detection:** Yes - dropped events don't trigger detections

**Observable:** Yes - dropped event count logged

### 2. Out-of-Order Events

**Scenario:** Events arrive with timestamps earlier than previously processed events.

**Behavior:**
- Clock skew allowance: 5 minutes
- Events within skew: Processed normally
- Events outside skew: Dropped with audit log

**Preserves Correctness:** Yes - temporal correlation uses event time

**Avoids False Detection:** Yes - out-of-order events don't corrupt state

**Observable:** Yes - ordering violations logged

### 3. Entity Explosion

**Scenario:** Number of entities exceeds max_entities limit.

**Behavior:**
- LRU eviction triggered
- Oldest entities evicted first
- New entities accepted after eviction
- Eviction logged for audit

**Preserves Correctness:** Yes - bounded entity count

**Avoids False Detection:** Yes - evicted entities don't trigger false detections

**Observable:** Yes - eviction events logged

### 4. Memory Pressure

**Scenario:** System memory pressure from entity state growth.

**Behavior:**
- TTL-based eviction accelerated
- Signal history trimmed aggressively
- Entity eviction prioritized
- Memory usage monitored

**Preserves Correctness:** Yes - eviction maintains correctness

**Avoids False Detection:** Yes - eviction doesn't create false detections

**Observable:** Yes - memory usage tracked and logged

### 5. Internal State Corruption

**Scenario:** Internal state becomes inconsistent (should never happen).

**Behavior:**
- State corruption detected
- Engine halts processing
- Full state dump for investigation
- Alert to operations

**Preserves Correctness:** Yes - halt prevents incorrect detections

**Avoids False Detection:** Yes - halt prevents false positives

**Observable:** Yes - corruption event logged with full context

### 6. Downstream Unavailability

**Scenario:** Policy Engine (Phase 6) unavailable.

**Behavior:**
- Detection results buffered
- Buffer size limited (bounded)
- Buffer overflow: Oldest detections dropped
- Retry with exponential backoff

**Preserves Correctness:** Yes - detections preserved in buffer

**Avoids False Detection:** Yes - no detections lost

**Observable:** Yes - buffer status logged

## Fail-Closed Principles

All failure modes follow fail-closed principles:

1. **No False Detections:** Failures never result in false positives
2. **State Preservation:** Entity state preserved when possible
3. **Audit Logging:** All failures logged with full context
4. **Graceful Degradation:** Engine continues operating within capacity

## Recovery

### Automatic Recovery

- TTL eviction: Automatic cleanup
- Memory pressure: Automatic eviction
- Buffer overflow: Automatic drop of oldest

### Manual Recovery

- State corruption: Requires investigation and restart
- Configuration errors: Requires configuration fix

## Testing

Failure modes are tested in:
- `scale_tests.rs`: Entity explosion, memory pressure
- `ordering_tests.rs`: Out-of-order events
- `invariant_violation_tests.rs`: State corruption scenarios

