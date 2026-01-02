# Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/docs/performance_model.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Performance model and design notes

# Performance Model

## Design Goals

- **Lightweight**: Small footprint, minimal resource usage
- **Bounded Memory**: All data structures have size limits
- **Low Overhead**: Minimal impact on system performance
- **Non-Blocking**: Never block syscall execution

## Memory Bounds

### Process Tracking
- Max processes: 10,000 (configurable via `AGENT_MAX_PROCESSES`)
- Eviction: LRU at 90% capacity
- Target: 80% after eviction
- Memory: O(n) where n ≤ max_processes

### Connection Tracking
- Max connections: 1,000 (configurable via `AGENT_MAX_CONNECTIONS`)
- Eviction: LRU at 90% capacity
- Memory: O(n) where n ≤ max_connections

### Queue Management
- Max queue size: 10,000 events (configurable via `AGENT_MAX_QUEUE_SIZE`)
- Backpressure: Drop at 80% threshold
- Memory: O(n) where n ≤ max_queue_size

### Feature Extraction
- Max features: 100 per event (fixed)
- Max paths: 50 per event (fixed)
- Memory: O(1) per event

## Performance Characteristics

- **Syscall Overhead**: <1% (eBPF) or <5% (auditd)
- **Memory Usage**: Bounded and predictable
- **CPU Usage**: Minimal (event-driven)
- **Disk I/O**: None (no persistent storage)

## Optimization Techniques

- **Lock-Free Statistics**: Atomic counters
- **Bounded Locks**: Minimal contention
- **Zero-Copy**: Direct memory access where possible
- **Event Batching**: Group events when possible

## Resource Constraints

- **Storage**: No persistent storage (telemetry only)
- **Network**: Minimal (only event emission)
- **CPU**: Event-driven, low overhead
- **Memory**: Bounded by configuration

