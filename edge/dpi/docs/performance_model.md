# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/docs/performance_model.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Performance model and design notes

# Performance Model

## Design Goals

- **≥10 Gbps Sustained Capture**: Designed for high-throughput networks
- **Zero Allocations in Hot Path**: No heap allocations during packet processing
- **Lock-Free or Bounded Locks**: Minimal contention
- **Bounded Memory**: All data structures have size limits

## Hot Path Components

### Packet Capture
- Zero-copy packet access
- Atomic statistics (lock-free)
- Non-blocking I/O

### Protocol Parsing
- Zero allocation parsing
- Direct memory access
- Deterministic results

### Flow Tracking
- Bounded hash map (max 1M flows)
- LRU eviction at 90% capacity
- Lock-free reads, bounded lock for writes

### Feature Extraction
- Bounded feature count (max 100)
- No dynamic allocations
- Deterministic extraction

## Memory Bounds

- **Flow Table**: Max 1,000,000 flows (configurable)
- **Queue Size**: Max 100,000 events (configurable)
- **Feature Count**: Max 100 features per packet
- **Packet Buffer**: 64MB capture buffer

## Performance Characteristics

- **Latency**: <1ms per packet (hot path)
- **Throughput**: ≥10 Gbps sustained
- **CPU Usage**: Optimized for multi-core
- **Memory Usage**: Bounded and predictable

## Backpressure Handling

- **Drop Threshold**: 80% of queue size
- **Deactivation**: 50% of threshold
- **Never Blocks**: Always returns immediately
- **Signal**: Non-blocking alert mechanism

## Rate Limiting

- **Token Bucket**: Lock-free implementation
- **Refill Rate**: Configurable (tokens/second)
- **Max Tokens**: Configurable
- **Non-Blocking**: Immediate return
