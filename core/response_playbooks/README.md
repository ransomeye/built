# Phase 6: Incident Response Playbooks

**Path:** `/home/ransomeye/rebuild/core/response_playbooks/`

## Overview

Phase 6 implements a first-class Incident Response Playbook Engine that:
- Defines response intent through structured playbooks
- Executes deterministic, auditable workflows
- Supports dry-run, execution, and rollback modes
- Is cryptographically verifiable (RSA-4096/Ed25519 signatures)
- Is restart-safe and replay-safe
- Integrates with Phase 3 (Policy), Phase 7 (Enforcement), Phase 8 (SOC Copilot), and Phase 10 (DB)

## Architecture

### Components

1. **Playbook Specification** (`schema/`)
   - YAML schema with signature support
   - Required fields: id, name, version, severity, trigger_conditions, steps, rollback_steps, approvals, timeouts, signature

2. **Playbook Registry** (`registry.rs`)
   - Loads playbooks from disk
   - Verifies cryptographic signatures
   - Validates schema
   - Version control (one active version per playbook)

3. **Playbook Executor** (`executor.rs`)
   - Deterministic step execution
   - Step-level state tracking
   - Timeout enforcement
   - Crash-safe resume
   - Replay protection (nonce + execution ID)

4. **Rollback Engine** (`rollback.rs`)
   - Reverse-order execution
   - Restart-safe persistence
   - Fail-closed on rollback failure (enters SAFE-HALT state)

5. **Persistence & Audit** (`persistence.rs`)
   - PostgreSQL integration
   - Immutable execution records
   - Cryptographically chained audit log
   - Nonce tracking (replay protection)

6. **Policy Binding** (`binding.rs`)
   - Explicit policy outcome → playbook mapping
   - No implicit actions
   - Missing binding → NO ACTION (fail-closed)

7. **SOC Copilot Visibility** (`visibility.rs`)
   - Read-only access to playbook intent
   - Execution status tracking
   - Rollback status tracking
   - Cannot modify playbooks

## Execution States

- `PENDING`: Execution queued
- `DRY_RUN`: Dry-run mode (no enforcement)
- `RUNNING`: Execution in progress
- `FAILED`: Execution failed
- `ROLLED_BACK`: Execution rolled back
- `COMPLETED`: Execution completed successfully

## Fail-Closed Behavior

The system fails closed on:
- Unsigned playbooks
- Invalid signatures
- Invalid schema
- Replay attempts (duplicate nonce)
- Missing policy bindings
- Rollback failures (enters SAFE-HALT state)

## Environment Variables

- `RANSOMEYE_PLAYBOOK_PUBLIC_KEY_PATH`: Path to public key for signature verification
- `RANSOMEYE_PLAYBOOK_DIR`: Directory containing playbook YAML files
- `RANSOMEYE_POLICY_BINDING_FILE`: Path to policy-playbook binding configuration
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASS`: Database connection

## Testing

Comprehensive test suite covers:
- Unsigned playbook rejection
- Invalid schema rejection
- Replay attempt detection
- Crash resume functionality
- Rollback execution
- Dry-run mode
- Policy binding validation

## Example Playbooks

See `playbooks/` directory for example signed playbooks:
- `ransomware_isolation.yaml`: Critical ransomware isolation response
- `malware_quarantine.yaml`: High-severity malware quarantine response

## Integration Points

- **Phase 3 (Policy)**: Receives policy decisions, maps to playbooks via bindings
- **Phase 7 (Enforcement)**: Executes enforcement actions via adapters
- **Phase 8 (SOC Copilot)**: Provides read-only visibility interface
- **Phase 10 (DB)**: Persists execution state, audit logs, rollback state

## Status

✅ **IMPLEMENTED** - Phase 6 is complete and production-ready.

