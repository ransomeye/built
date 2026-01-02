# Windows Agent Architecture

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_windows_agent/docs/agent_architecture.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Architecture documentation for RansomEye Windows Agent - user-mode endpoint sensor

---

## Overview

The Windows Agent is a **stand-alone, untrusted endpoint sensor** that observes Windows endpoint activity, collects telemetry, and emits signed events to the Control Plane (Core).

## Core Principles

### 1. User-Mode Only
- **No kernel drivers**: Pure user-mode implementation
- **Windows APIs only**: Uses standard Windows APIs
- **No system modifications**: Never modifies system behavior

### 2. Untrusted Component
- Windows Agent is **untrusted** by design
- All telemetry must be signed
- Core validates all received events
- Invalid events are rejected

### 3. No Decision Making
- **Zero AI/ML**: No classification or inference
- **Zero heuristics**: No pattern matching
- **Observation only**: Metadata collection

### 4. Bounded State
- **Bounded buffers**: Limited memory usage
- **No long-term storage**: No persistent state
- **Ephemeral monitoring**: Activity state cleaned up automatically

## Component Architecture

The Windows Agent consists of:

1. **Process Activity Monitor**: Observes process creation/termination
2. **File Activity Monitor**: Observes file create/modify/delete
3. **Registry Activity Monitor**: Observes registry create/modify/delete
4. **Auth Activity Monitor**: Observes authentication events
5. **Network Activity Monitor**: Observes network connections
6. **Event Signer**: Signs all telemetry with RSA-4096-PSS-SHA256
7. **Transport Client**: mTLS client for sending signed events to Core
8. **Disk Buffer**: Persistent buffering when Core unavailable
9. **Backpressure Handler**: Manages bounded buffers and backpressure signals

## Data Flow

1. **Activity Detection**: Windows APIs detect process/file/registry/network/auth activity
2. **Event Creation**: Activity converted to structured event
3. **Event Signing**: Telemetry signed with RSA-4096-PSS-SHA256
4. **Transport**: Signed events sent to Core via mTLS
5. **Backpressure Handling**: Core signals backpressure if overloaded
6. **Disk Buffering**: Events buffered to disk when Core unavailable

## Security Model

### Identity
- Per-instance X.509 certificate
- Unique agent identity hash
- Certificate stored securely

### Signing
- All events signed with RSA-4096-PSS-SHA256
- Nonce-based replay protection
- Timestamp validation

### Transport
- mTLS to Core API
- Certificate chain validation
- Trust anchor verification

## Failure Modes

- **Core unavailable**: Events buffered to disk, retry later
- **Privilege violation**: Sensor disabled, log error
- **Identity failure**: Transmission halted
- **Resource exhaustion**: Graceful degradation

## Constraints

- **User-mode only**: No kernel drivers
- **Observation only**: No enforcement, no blocking
- **No AI/ML**: Feature extraction only
- **Bounded state**: Limited memory and disk usage
