# DPI Probe Architecture

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/docs/dpi_architecture.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Architecture documentation for RansomEye DPI Probe - passive network sensor design

---

## Overview

The DPI Probe is a **stand-alone, untrusted, passive network sensor** that observes network traffic, assembles flows, extracts features, and emits signed telemetry to the Control Plane (Core).

## Core Principles

### 1. Passive Only
- **Zero packet modification**: Never alters packets
- **Zero enforcement**: Never blocks or drops traffic
- **Observation only**: Pure sensor functionality

### 2. Untrusted Component
- DPI Probe is **untrusted** by design
- All telemetry must be signed
- Core validates all received events
- Invalid events are rejected

### 3. No Decision Making
- **Zero AI/ML**: No classification or inference
- **Zero heuristics**: No pattern matching
- **Feature extraction only**: Metadata extraction

### 4. Bounded State
- **Bounded buffers**: Limited memory usage
- **No long-term storage**: No persistent state
- **Ephemeral flows**: Flow state cleaned up automatically

## Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   DPI Probe Process                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐      ┌──────────────┐                │
│  │   Capture    │─────▶│    Flow      │                │
│  │   Engine     │      │  Assembler   │                │
│  └──────────────┘      └──────────────┘                │
│                                │                        │
│                                ▼                        │
│                        ┌──────────────┐                │
│                        │   Feature    │                │
│                        │  Extractor   │                │
│                        └──────────────┘                │
│                                │                        │
│                                ▼                        │
│                        ┌──────────────┐                │
│                        │    Event     │                │
│                        │    Signer    │                │
│                        └──────────────┘                │
│                                │                        │
│                                ▼                        │
│  ┌──────────────┐      ┌──────────────┐                │
│  │   Backpres.  │◀─────│  Transport   │──────▶ Core    │
│  │   Handler    │      │    Client    │                │
│  └──────────────┘      └──────────────┘                │
│         │                                               │
│         ▼                                               │
│  ┌──────────────┐                                       │
│  │ Disk Buffer  │  (when Core unavailable)             │
│  └──────────────┘                                       │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

1. **Packet Capture**: Raw packets captured from network interface
2. **Flow Assembly**: Packets aggregated into network flows
3. **Feature Extraction**: Flow metadata extracted (NO AI)
4. **Event Signing**: Telemetry signed with RSA-4096-PSS-SHA256
5. **Transport**: Signed events sent to Core via mTLS
6. **Backpressure Handling**: Core signals backpressure if overloaded
7. **Disk Buffering**: Events buffered to disk when Core unavailable

## Security Model

### Identity
- Per-instance X.509 certificate
- Unique probe identity hash
- Certificate stored securely

### Signing
- All events signed with RSA-4096-PSS-SHA256
- Nonce-based replay protection
- Timestamp validation

### Transport
- mTLS to Core API
- Certificate chain validation
- Trust anchor verification

### Failure Modes
- **Signature failure**: Event dropped, not sent
- **Identity failure**: Transmission halted
- **Core unavailable**: Buffer to disk, retry later

## Failure Handling

### Core Unavailable
1. Events buffered to disk
2. Retry with exponential backoff
3. Drop oldest events if buffer full
4. Continue capturing (fail-open locally)

### Backpressure
1. Core signals backpressure
2. Transport slows down transmission
3. Events buffered if queue full
4. Drop oldest events if buffer full

### Resource Exhaustion
1. Monitor buffer size
2. Drop oldest events first
3. Log all drops
4. Preserve process stability

## Performance Characteristics

- **Throughput**: 10Gbps+ packet processing
- **Latency**: Sub-millisecond per-packet processing
- **Memory**: Bounded by buffer limits
- **CPU**: Efficient packet parsing
- **Disk**: Minimal (buffering only)

## Interface Requirements

- **Network Interface**: Promiscuous mode capture
- **File System**: Writable buffer directory
- **Network**: mTLS connection to Core API
- **System**: Standard Linux capabilities

## Constraints

- **No packet modification**: Read-only access
- **No traffic blocking**: Passive observation
- **No AI/ML**: Feature extraction only
- **No enforcement**: No policy decisions
- **No long-term storage**: Bounded state only
