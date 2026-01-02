# Performance Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_core/docs/performance_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Performance characteristics and scale model

## Overview

The correlation engine is designed to operate at **50,000+ endpoints scale** with bounded memory and deterministic performance.

## Memory Bounds

### Per-Entity Limits

- **Max Signals:** 1000 signals per entity (configurable)
- **Max Transitions:** 50 transitions per entity (configurable)
- **Signal History:** Bounded by max_signals_per_entity
- **Transition History:** Bounded by max_transitions_per_entity

### Global Limits

- **Max Entities:** 50,000 entities (configurable)
- **Temporal Window Events:** 1000 events per window (configurable)
- **Entity Graph Nodes:** 50,000 nodes (configurable)

## Eviction Strategy

### TTL-Based Eviction

- Entities expire after `entity_ttl_seconds` (default: 3600 seconds)
- Expired entities are evicted during cleanup
- Eviction runs periodically or on capacity pressure

### LRU Eviction

- Least-recently-used entities evicted first
- Used when TTL eviction insufficient
- Maintains fair access to correlation resources

### Signal History Eviction

- Oldest signals evicted when history exceeds limit
- Maintains temporal window for correlation
- Preserves recent signals for detection

## Throughput Limits

### Event Processing

- **Per-Entity:** Bounded by signal history limits
- **Global:** Bounded by entity count limits
- **Temporal Window:** Bounded by max_events_per_window

### Backpressure

- If capacity exceeded, events may be dropped
- Dropped events logged for audit
- Phase 4 should handle backpressure

## Scheduler Fairness

### Priority-Based Scheduling

- **Critical:** Highest priority (ransomware detections)
- **High:** High priority (suspicious activity)
- **Normal:** Normal priority (baseline activity)
- **Low:** Low priority (legitimate activity)

### Round-Robin

- Entities processed in round-robin order within priority
- Prevents starvation
- Ensures fair resource allocation

## Lock-Free Design

### DashMap Usage

- Entity state stored in DashMap (lock-free hash map)
- Low-lock contention for concurrent access
- Scales to high concurrency

### Parking Lot Locks

- Used for infrequent operations (eviction, stats)
- Minimizes lock contention
- Non-blocking for common operations

## Scale Testing

### 50k Entity Test

- Creates 50,000 entities
- Processes events for each entity
- Verifies memory bounds maintained
- Validates eviction works correctly

### Memory Bounds Test

- Adds 200 signals to single entity
- Verifies signal history bounded
- Confirms memory usage reasonable

### Eviction Test

- Fills to capacity
- Adds one more entity
- Verifies eviction triggered
- Confirms capacity maintained

## Performance Characteristics

- **Memory:** Bounded per entity and globally
- **Latency:** Deterministic (no GC pauses)
- **Throughput:** Scales with entity count
- **Concurrency:** Lock-free design supports high concurrency

