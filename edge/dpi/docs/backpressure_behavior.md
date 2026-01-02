# DPI Probe Backpressure Behavior

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/docs/backpressure_behavior.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Backpressure handling and buffer management behavior documentation

---

## Overview

DPI Probe implements explicit backpressure handling to prevent overwhelming Core when it cannot process events fast enough. The system uses bounded buffers and explicit signals from Core to manage transmission rate.

## Backpressure Signals

### From Core
Core can signal backpressure via:
1. **HTTP 429 Too Many Requests**: Rate limit exceeded
2. **Response header**: `X-Backpressure: true`
3. **Response body**: `{"status": "BACKPRESSURE"}`
4. **Connection close**: Implicit backpressure signal

### Detection
Transport client detects backpressure by:
- Checking HTTP status codes
- Parsing response headers
- Monitoring connection failures
- Tracking response latency

## Buffer Management

### Memory Buffer
- **Size**: Configurable via `MAX_BUFFER_SIZE_MB`
- **Purpose**: Hold events in memory before transmission
- **Policy**: Drop oldest events when full

### Disk Buffer
- **Location**: Configurable via `BUFFER_DIR`
- **Purpose**: Persistent storage when Core unavailable
- **Policy**: Drop oldest events when full

### Buffer Hierarchy
1. **Memory buffer**: First-level buffer for active transmission
2. **Disk buffer**: Second-level buffer for Core unavailability
3. **Drop policy**: Oldest events dropped first

## Backpressure States

### Normal Operation
- Events transmitted immediately
- Memory buffer < threshold
- No backpressure signals

### Backpressure Active
- Core signals backpressure
- Transmission rate reduced
- Events buffered in memory
- Exponential backoff retry

### Core Unavailable
- Connection failures
- Events buffered to disk
- Retry with exponential backoff
- Continue capturing (fail-open)

### Buffer Full
- Memory buffer at capacity
- Disk buffer at capacity
- Oldest events dropped
- Continue capturing

## Behavior Under Backpressure

### Transmission Rate
- **Normal**: Maximum transmission rate
- **Backpressure**: Reduced rate (exponential backoff)
- **Retry interval**: Starts at 100ms, doubles each retry
- **Max interval**: 30 seconds

### Buffer Growth
- **Memory buffer**: Grows until threshold
- **Disk buffer**: Grows until max size
- **Drop policy**: FIFO (oldest first)

### Event Loss
- **Logged**: All drops logged with metrics
- **Monitored**: Drop rate tracked
- **Alerts**: High drop rate triggers alerts
- **Recovery**: Automatic when backpressure clears

## Configuration

### Thresholds
- `BACKPRESSURE_THRESHOLD`: Memory buffer threshold (bytes)
- `MAX_BUFFER_SIZE_MB`: Maximum buffer size (MB)

### Retry Policy
- Initial delay: 100ms
- Max delay: 30s
- Exponential backoff: 2x per retry
- Max retries: Unlimited (until success)

## Monitoring

### Metrics
- **Buffer utilization**: Current size / max size
- **Drop count**: Total events dropped
- **Drop rate**: Drops per second
- **Backpressure duration**: Time in backpressure state
- **Retry count**: Number of retries

### Alerts
- **High buffer utilization**: > 80%
- **High drop rate**: > 100 drops/second
- **Prolonged backpressure**: > 5 minutes
- **Buffer full**: 100% utilization

## Failure Scenarios

### Core Overloaded
1. Core signals backpressure
2. DPI Probe reduces transmission rate
3. Events buffered in memory
4. Retry with backoff
5. Recovery when Core ready

### Core Unavailable
1. Connection failures
2. Events buffered to disk
3. Retry with exponential backoff
4. Continue capturing
5. Recovery when Core available

### Buffer Full
1. Memory buffer at capacity
2. Disk buffer at capacity
3. Oldest events dropped
4. Drops logged and monitored
5. Continue capturing

## Best Practices

### Buffer Sizing
- **Memory buffer**: 2-4x normal transmission rate
- **Disk buffer**: 10-100x normal transmission rate
- **Consider**: Network bandwidth, Core capacity

### Threshold Tuning
- **Low threshold**: Early backpressure detection
- **High threshold**: More buffering capacity
- **Balance**: Latency vs. drop rate

### Monitoring
- Track buffer utilization
- Monitor drop rates
- Alert on sustained backpressure
- Review Core capacity

## Recovery Behavior

### Automatic Recovery
- **Backpressure cleared**: Resume normal transmission
- **Core available**: Flush disk buffer
- **Buffer space**: Accept new events

### Manual Recovery
- **Restart DPI Probe**: Clear buffers (events lost)
- **Increase buffer size**: More capacity
- **Fix Core**: Resolve underlying issue
