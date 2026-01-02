# Performance Model

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/docs/performance_model.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Performance characteristics and resource usage model for Windows Agent

## Resource Bounds

### Memory
- **Process Tracking:** Max 10,000 processes (configurable)
- **Filesystem Tracking:** Max 50,000 paths (configurable)
- **Network Tracking:** Max 10,000 connections (configurable)
- **Buffer Size:** 100MB default (configurable)
- **Backpressure Threshold:** 80% of buffer size

### CPU
- **ETW Overhead:** < 1% CPU (kernel-level filtering)
- **Event Processing:** Bounded by rate limiter (10,000 events/sec default)
- **Signing Overhead:** Ed25519 signing ~0.1ms per event

### Disk I/O
- **Event Buffering:** Only when Core unavailable or backpressure
- **Identity Storage:** Single JSON file (~1KB)
- **Key Storage:** Single key file (32 bytes)

## Rate Limiting

- **Default:** 10,000 events per second
- **Enforcement:** Sliding window (1 second)
- **Behavior:** Events exceeding limit are dropped with logging

## Backpressure Handling

- **Activation:** When buffer reaches 80% capacity
- **Response:** Events are dropped until buffer < 80%
- **Recovery:** Automatic when buffer drains

## Bounded Memory Proof

All tracking structures enforce hard limits:
1. Process monitor evicts oldest processes when limit reached
2. Filesystem monitor evicts oldest paths when limit reached
3. Network monitor evicts oldest connections when limit reached
4. Buffer enforces maximum size with backpressure

## Deterministic Behavior

- Event processing is deterministic (no random behavior)
- Sequence numbers are monotonic
- Timestamps are consistent
- Feature extraction is idempotent

