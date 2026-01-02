# RansomEye Windows Agent (Phase 9C)

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/README.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Windows Agent - Standalone host-based telemetry sensor

## Overview

The RansomEye Windows Agent is a **stand-alone** telemetry sensor for Windows endpoints. It collects process, filesystem, registry, and network telemetry via ETW (Event Tracing for Windows) and emits signed events to the Phase 4 ingestion pipeline.

## Key Characteristics

- **NO enforcement** - Observation only
- **NO policy decisions** - Telemetry emission only
- **NO remediation actions** - Sensor only
- **ETW primary** - High-performance kernel-level events
- **WMI fallback** - When ETW unavailable
- **Ed25519 signing** - Every event cryptographically signed
- **Phase-4 compliant** - Envelopes compatible with ingestion pipeline
- **Bounded memory** - Hard limits on all tracking structures
- **Fail-closed** - On identity or signing failure

## Directory Structure

```
ransomeye_windows_agent/
├── agent/
│   └── src/
│       ├── main.rs          # Main entry point
│       ├── lib.rs           # Library exports
│       ├── errors.rs        # Error types
│       ├── process.rs       # Process monitoring
│       ├── filesystem.rs    # Filesystem monitoring
│       ├── registry.rs      # Registry monitoring
│       ├── network.rs        # Network monitoring
│       ├── etw.rs           # ETW abstraction
│       ├── features.rs      # Feature extraction
│       ├── envelope.rs      # Phase-4 envelopes
│       ├── backpressure.rs   # Backpressure handling
│       ├── rate_limit.rs    # Rate limiting
│       └── health.rs        # Health monitoring
├── security/
│   ├── mod.rs              # Security module exports
│   ├── identity.rs          # Component identity
│   ├── signing.rs          # Ed25519 signing
│   └── attestation.rs      # Component attestation
├── config/
│   └── validation.rs       # Configuration validation
├── docs/
│   ├── etw_coverage.md     # ETW provider coverage
│   ├── performance_model.md # Performance characteristics
│   ├── privacy_guarantees.md # Privacy guarantees
│   ├── failure_modes.md    # Failure modes
│   └── hardening.md        # Security hardening
└── tests/
    ├── etw_tests.rs        # ETW tests
    ├── process_tests.rs    # Process monitoring tests
    ├── fs_behavior_tests.rs # Filesystem tests
    ├── registry_tests.rs   # Registry tests
    ├── signing_tests.rs    # Signing tests
    └── determinism_tests.rs # Determinism tests
```

## Building

```bash
cd /home/ransomeye/rebuild/ransomeye_windows_agent
cargo build --release
```

## Testing

```bash
cargo test
```

## Configuration

Configuration is loaded from environment variables or default values:
- `max_buffer_size_mb`: Maximum buffer size (default: 100MB)
- `backpressure_threshold`: Backpressure threshold (default: 0.8)
- `max_events_per_second`: Rate limit (default: 10,000)

## Security

- Ed25519 signing for all events
- Component identity enforced at startup
- Replay protection via sequence numbers
- Fail-closed on identity or signing failure

## Documentation

See `docs/` directory for detailed documentation:
- ETW coverage and providers
- Performance model and resource bounds
- Privacy guarantees
- Failure modes and recovery
- Security hardening guide

## Compliance

- Stand-alone module (no unified installer dependency)
- No kernel driver required
- ETW primary, WMI fallback
- Bounded memory usage
- Deterministic behavior
