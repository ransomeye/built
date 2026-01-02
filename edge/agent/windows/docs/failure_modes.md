# Failure Modes

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/docs/failure_modes.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Failure modes and recovery behavior for Windows Agent

## Critical Failures (Fail-Closed)

### 1. Identity Failure
- **Trigger:** Component identity cannot be loaded or created
- **Behavior:** Agent fails to start
- **Recovery:** Manual intervention required

### 2. Signing Failure
- **Trigger:** Ed25519 key cannot be created or loaded
- **Behavior:** Agent fails to start
- **Recovery:** Manual intervention required

### 3. ETW Initialization Failure
- **Trigger:** ETW session cannot be created
- **Behavior:** Agent fails to start
- **Recovery:** Check Windows permissions, ETW availability

## Degraded Modes

### 1. ETW Session Loss
- **Trigger:** ETW session stops unexpectedly
- **Behavior:** Agent continues, health status = Degraded
- **Recovery:** Automatic retry, WMI fallback if available

### 2. Core Unavailable
- **Trigger:** Phase 4 ingestion pipeline unreachable
- **Behavior:** Events buffered to disk
- **Recovery:** Automatic flush when Core available

### 3. Backpressure
- **Trigger:** Buffer exceeds 80% capacity
- **Behavior:** Events dropped, health status = Degraded
- **Recovery:** Automatic when buffer drains

### 4. Rate Limit Exceeded
- **Trigger:** Event rate > 10,000 events/sec
- **Behavior:** Events dropped
- **Recovery:** Automatic when rate decreases

## Recovery Mechanisms

1. **Automatic Retry:** ETW session restart attempts
2. **Buffer Flush:** Automatic when Core available
3. **Health Monitoring:** Continuous health checks
4. **Graceful Degradation:** Continue operation in degraded mode

## Monitoring

- Health status reported continuously
- Events processed/dropped counters
- ETW session status
- Buffer utilization

