# RansomEye Enforcement Dispatcher

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_enforcement/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 7 - Enforcement Dispatcher & Safety Guards

## Overview

The RansomEye Enforcement Dispatcher is the **safety fuse** between policy decisions and system execution. It converts signed policy decisions into controlled enforcement commands with comprehensive safety guards.

## Core Principles

### Fail-Closed
- Unsigned decision → REJECT
- Missing approval → HOLD
- Guardrail violation → REJECT
- Adapter failure → HALT
- Partial execution → ROLLBACK
- Any ambiguity → NO ACTION

### Safety First
- All decisions must be signed
- All decisions must be validated
- All approvals must be present
- All guardrails must pass
- All rate limits must be respected
- All blast radius limits must be enforced

### Execution Only
- Dispatcher does NOT make decisions
- Dispatcher does NOT override policy
- Dispatcher does NOT execute blindly
- Dispatcher ONLY executes signed, validated, approved decisions

## Architecture

### Components

1. **Dispatcher** - Main orchestrator
2. **Validator** - Decision integrity validation
3. **Approvals** - Approval workflow enforcement
4. **Guardrails** - Safety guard enforcement
5. **Rate Limiter** - Execution throttling
6. **Blast Radius Limiter** - Scope limiting
7. **Rollback Manager** - Reversible operations
8. **Dry-Run Executor** - Simulation mode
9. **Platform Adapters** - Linux, Windows, Network

### Enforcement Flow

```
1. Receive signed policy decision
   ↓
2. Verify signature & revocation
   ↓
3. Check approval requirements
   ↓
4. Apply safety guardrails
   ↓
5. Apply blast-radius limits
   ↓
6. Rate-limit execution
   ↓
7. Dispatch to correct adapter
   ↓
8. Record execution evidence
```

## Safety Guards

### Mandatory Guards
- Max hosts per action (default: 10)
- Max actions per window (default: 100/hour)
- Asset class restrictions
- Environment constraints
- Destructive action approval

### Blast Radius Limiting
- Max hosts per window (default: 50)
- Window duration (default: 3600 seconds)

## Approval Workflows

### Approval Types
- `operator`: Standard operator approval
- `manager`: Management-level approval
- `security`: Security team approval
- `automated`: Automated approval (low-risk)

### Approval Process
1. Decision requires approval → Execution HELD
2. Approval recorded → Execution proceeds
3. All approvals present → Execution completed

## Platform Adapters

### Linux Agent Adapter
- Executes via Linux agent API
- Supports: block, isolate, quarantine, monitor
- Generates iptables commands

### Windows Agent Adapter
- Executes via Windows agent API
- Supports: block, isolate, quarantine, monitor
- Generates PowerShell commands

### Network Adapter
- Executes at network level
- Supports: block, isolate, quarantine, monitor
- Generates network commands

## Configuration

### Required Environment Variables
- `RANSOMEYE_POLICY_PUBLIC_KEY_PATH`: Path to policy public key

### Optional Environment Variables
- `RANSOMEYE_REVOCATION_LIST_PATH`: Path to revocation list
- `RANSOMEYE_ENFORCEMENT_MAX_HOSTS`: Max hosts per action (default: 10)
- `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_MAX_ACTIONS`: Max actions per window (default: 100)
- `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_WINDOW_SECONDS`: Rate limit window (default: 3600)
- `RANSOMEYE_ENFORCEMENT_BLAST_RADIUS_MAX_HOSTS`: Max hosts per blast radius (default: 50)
- `RANSOMEYE_ENFORCEMENT_ALLOWED_ASSET_CLASSES`: Allowed asset classes (comma-separated)
- `RANSOMEYE_ENFORCEMENT_ALLOWED_ENVIRONMENTS`: Allowed environments (comma-separated)
- `RANSOMEYE_LINUX_AGENT_API_URL`: Linux agent API URL
- `RANSOMEYE_WINDOWS_AGENT_API_URL`: Windows agent API URL
- `RANSOMEYE_NETWORK_API_URL`: Network API URL

## Usage

### Basic Usage
```rust
use ransomeye_enforcement::EnforcementDispatcher;

let dispatcher = EnforcementDispatcher::new()?;
let result = dispatcher.dispatch(decision_json, targets, dry_run).await?;
```

### Dry-Run Mode
```rust
let result = dispatcher.dispatch(decision_json, targets, true).await?;
```

### Rollback
```rust
dispatcher.rollback(execution_id)?;
```

### Record Approval
```rust
dispatcher.record_approval(decision_id, "operator", "admin")?;
```

## Testing

### Test Suite
- Unsigned decision rejection tests
- Approval required tests
- Blast radius limit tests
- Dry-run tests
- Rollback tests

### Running Tests
```bash
cargo test
```

## Documentation

- [Enforcement Model](docs/enforcement_model.md)
- [Safety Guards](docs/safety_guards.md)
- [Approval Workflows](docs/approval_workflows.md)
- [Failure Modes](docs/failure_modes.md)

## Security

### Signature Verification
- RSA-4096-PSS-SHA256 algorithm
- Public key from environment
- Signature verified against decision hash

### Revocation Checking
- Revocation list from environment
- Revoked decisions rejected immediately
- Revocation list reloaded on startup

### Decision Integrity
- Decision hash verified
- Signature verified
- Revocation status checked
- All checks must pass

## License

Copyright © RansomEye.Tech | Support: Gagan@RansomEye.Tech

