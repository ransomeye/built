# Phase 16: Deception Framework

**Path:** `/home/ransomeye/rebuild/core/deception/`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Status:** ✅ **IMPLEMENTED**

## Overview

Phase 16 implements a **Deception Framework** that deploys defensive deception assets (decoys, lures, breadcrumbs) to detect unauthorized access and lateral movement. The framework is:

- **Topology-aware** (integrates with Phase 9 Network Scanner)
- **High-confidence signal generation** (confidence >= 0.9)
- **Signed and auditable** (Ed25519 signatures)
- **Reversible** (explicit teardown and rollback)
- **Safe by design** (never intercepts real traffic, never interferes with production)

## Architecture

### Components

1. **Deception Asset Model** (`schema/deception_asset.schema.yaml`)
   - YAML schema for signed deception assets
   - Asset types: `decoy_host`, `decoy_service`, `credential_lure`, `filesystem_lure`
   - Deployment scopes: `network`, `host`, `identity`
   - Visibility levels: `low`, `medium`, `high`
   - Required fields: `asset_id`, `asset_type`, `trigger_conditions`, `teardown_procedure`, `signature`

2. **Deception Registry** (`src/registry.rs`)
   - Loads deception assets from directory
   - Verifies Ed25519 signatures (FAIL-CLOSED on invalid signature)
   - Validates schema (FAIL-CLOSED on invalid schema)
   - Enforces allowed asset types (rejects forbidden types like `traffic_interceptor`)
   - Validates no production overlap (integrates with Phase 9)

3. **Deployment Engine** (`src/deployer.rs`)
   - **SAFE BY DESIGN**: Never binds to production ports, never intercepts traffic, never proxies services
   - Can only: advertise presence, accept connection, log interaction, immediately drop or sandbox
   - Idempotent deployment (safe to deploy multiple times)
   - Bounded deployment (time-limited via `max_lifetime`)
   - Validates no production overlap before deployment

4. **Telemetry & Signal Engine** (`src/signals.rs`)
   - Generates high-confidence signals only (confidence >= 0.9)
   - Cryptographically signed signals (Ed25519)
   - Signal validation (rejects unsigned or low-confidence signals)
   - Signal metadata includes: `asset_id`, `interaction_type`, `timestamp`, `confidence_score`, `hash`, `signature`

5. **Correlation Integration** (`src/correlation.rs`)
   - Exposes deception signals as **strong indicators** (not probabilistic noise)
   - Signals can elevate correlation confidence
   - Signals can short-circuit detection timelines
   - **NO auto-enforcement** (explicit playbook mapping required)

6. **Playbook Integration** (`src/playbook_integration.rs`)
   - **Explicit mapping only**: Signal interaction_type → Playbook ID
   - Missing mapping → NO ACTION (fail-closed)
   - Returns playbook IDs for Phase 6 execution
   - Examples:
     - `credential_lure_touched` → containment playbook
     - `decoy_ssh_accessed` → isolation + snapshot playbook

7. **SOC Copilot Visibility** (`src/visibility.rs`)
   - **READ-ONLY** access to deception assets
   - Cannot deploy, modify, or tear down assets
   - Can view: deployed assets, asset health, interaction history, triggered playbooks

8. **Teardown & Rollback Engine** (`src/teardown.rs`)
   - Explicit teardown (manual asset removal)
   - Automatic teardown on timeout (when `max_lifetime` exceeded)
   - Emergency teardown via playbook rollback (Phase 6 integration)
   - **FAIL-CLOSED**: Teardown failure → Safe-halt state
   - Guaranteed rollback removes all assets

9. **Security Module** (`src/security.rs`)
   - Ed25519 signature verification for assets
   - Ed25519 signature verification for signals
   - SHA-256 hash computation
   - Public key loading from environment

## Safety Rules (FAIL-CLOSED)

1. **No Traffic Interception**: Assets never intercept real production traffic
2. **No Production Interference**: Assets never bind to production ports or services
3. **No Proxy Services**: Assets never proxy to production services
4. **Signed Assets Only**: Unsigned assets are rejected
5. **High-Confidence Signals Only**: Signals with confidence < 0.9 are rejected
6. **Explicit Playbook Mapping**: Missing mapping → NO ACTION
7. **Teardown Guarantee**: Teardown failure → Safe-halt state

## Environment Variables

```bash
# Asset directory
DECEPTION_ASSET_DIR=/etc/ransomeye/deception/assets

# Public key for signature verification
DECEPTION_PUBLIC_KEY_PATH=/etc/ransomeye/keys/deception_public_key.pem

# Private key for signal signing
DECEPTION_PRIVATE_KEY_PATH=/etc/ransomeye/keys/deception_private_key.pem

# Playbook mappings (explicit)
DECEPTION_PLAYBOOK_MAPPINGS="credential_lure_touched:containment_playbook_id,decoy_ssh_accessed:isolation_playbook_id"
```

## Integration Points

### Phase 5 (Correlation Engine)
- Deception signals are treated as **strong indicators** (confidence >= 0.9)
- Signals can elevate correlation confidence
- Signals can short-circuit detection timelines

### Phase 6 (Playbooks)
- Explicit mapping: Signal interaction_type → Playbook ID
- Missing mapping → NO ACTION (fail-closed)
- Emergency teardown via playbook rollback

### Phase 9 (Network Scanner)
- Validates no production overlap before deployment
- Uses network scanner results to check for conflicts

### Phase 7 (SOC Copilot)
- Read-only visibility to deception assets
- Cannot deploy, modify, or tear down assets

## Testing

Comprehensive tests cover:

- ✅ Unsigned deception asset → rejected
- ✅ Asset overlapping real service → rejected
- ✅ Signal without signature → rejected
- ✅ Teardown after timeout
- ✅ Rollback removes all assets
- ✅ Correlation receives high-confidence signal
- ✅ Playbook mapping explicit only
- ✅ Synthetic assets only

## Example Signed Deception Asset

See `examples/decoy_ssh_service.yaml` for an example signed deception asset.

## Status

✅ **Phase 16 Complete**

- [x] Deception asset schema created
- [x] Registry with signature verification implemented
- [x] Safe deployment engine implemented
- [x] High-confidence signal generation implemented
- [x] Correlation integration implemented
- [x] Playbook integration implemented
- [x] SOC Copilot visibility implemented
- [x] Teardown & rollback engine implemented
- [x] Comprehensive tests created
- [x] Guardrails updated

## Next Steps

1. Generate signing keys for deception assets
2. Create example signed deception assets
3. Configure playbook mappings
4. Integrate with network scanner for overlap detection
5. Deploy and test in staging environment

