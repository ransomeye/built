# Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/docs/failure_modes.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Failure modes and error handling

# Failure Modes

## Fail-Closed Behavior

### Identity/Signing Failure
- **Identity Load Failure**: Agent fails to start
- **Signing Key Failure**: Agent fails to start
- **Signature Verification Failure**: Event rejected

### Configuration Failure
- **Missing Required ENV**: Agent fails to start (if required)
- **Invalid Configuration**: Agent fails to start
- **Validation Failure**: Agent fails to start

### Syscall Monitoring Failure
- **eBPF Failure**: Falls back to auditd (if enabled)
- **auditd Failure**: Agent fails to start (if eBPF also failed)
- **Both Failed**: Agent fails to start

## Graceful Degradation

### Backpressure
- **Queue Full**: Drop events + signal (never block)
- **Rate Limit**: Drop events (never block)
- **Memory Limit**: Evict processes/connections (LRU)

### Health Monitoring
- **Idle Timeout**: Agent marked unhealthy
- **Error Rate**: Agent marked unhealthy (>10% errors)
- **Health Check Failure**: Agent stops

## Error Recovery

### Automatic Recovery
- **Backpressure**: Auto-deactivates when queue <50% threshold
- **Rate Limiting**: Tokens auto-refill
- **Process Eviction**: Automatic LRU eviction
- **Connection Eviction**: Automatic LRU eviction

### Manual Recovery
- **Configuration**: Restart with corrected ENV
- **Identity**: Reload identity file
- **Signing Key**: Reload signing key file
- **Syscall Monitoring**: Restart agent

## Error Handling

### Process Monitoring Errors
- **Memory Limit**: Evict processes (LRU)
- **Process Table Full**: Reject new processes

### Filesystem Monitoring Errors
- **Path Tracking**: Bounded path count
- **Mass Write Detection**: Threshold-based

### Network Monitoring Errors
- **Connection Limit**: Evict connections (LRU)
- **Connection Table Full**: Reject new connections

## Monitoring

- **Health Stats**: Uptime, events processed, errors
- **Backpressure Stats**: Events dropped, queue size
- **Process Stats**: Process count, eviction count
- **Network Stats**: Connection count, eviction count

