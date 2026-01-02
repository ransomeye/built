# Path and File Name : /home/ransomeye/rebuild/ransomeye_dpi_probe/docs/failure_modes.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Failure modes and error handling

# Failure Modes

## Fail-Closed Behavior

### Identity/Signing Failure
- **Identity Load Failure**: Component fails to start
- **Signing Key Failure**: Component fails to start
- **Signature Verification Failure**: Event rejected

### Configuration Failure
- **Missing Required ENV**: Component fails to start
- **Invalid Configuration**: Component fails to start
- **Validation Failure**: Component fails to start

## Graceful Degradation

### Backpressure
- **Queue Full**: Drop packets + signal (never block)
- **Rate Limit**: Drop packets (never block)
- **Memory Limit**: Evict flows (LRU)

### Health Monitoring
- **Idle Timeout**: Component marked unhealthy
- **Error Rate**: Component marked unhealthy (>10% errors)
- **Health Check Failure**: Component stops

## Error Recovery

### Automatic Recovery
- **Backpressure**: Auto-deactivates when queue <50% threshold
- **Rate Limiting**: Tokens auto-refill
- **Flow Eviction**: Automatic LRU eviction

### Manual Recovery
- **Configuration**: Restart with corrected ENV
- **Identity**: Reload identity file
- **Signing Key**: Reload signing key file

## Error Handling

### Capture Errors
- **Device Not Found**: Fatal (fail-closed)
- **Capture Activation Failure**: Fatal (fail-closed)
- **Packet Read Error**: Drop packet + log

### Parsing Errors
- **Invalid Packet**: Drop packet + log
- **Unsupported Protocol**: Mark as Unknown
- **Truncated Packet**: Partial parse (if possible)

### Flow Tracking Errors
- **Memory Limit**: Evict flows (LRU)
- **Flow Table Full**: Reject new flows

## Monitoring

- **Health Stats**: Uptime, packets processed, errors
- **Backpressure Stats**: Packets dropped, queue size
- **Capture Stats**: Packets captured, dropped, bytes
- **Flow Stats**: Flow count, eviction count
