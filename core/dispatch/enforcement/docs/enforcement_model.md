# Enforcement Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_enforcement/docs/enforcement_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Enforcement dispatcher model documentation - safety-first execution model

## Overview

The RansomEye Enforcement Dispatcher is the **ONLY component** that converts policy decisions into actual enforcement actions. It acts as the safety fuse between policy decisions and system execution.

## Core Principles

### 1. Fail-Closed
- Unsigned decision → REJECT
- Missing approval → HOLD
- Guardrail violation → REJECT
- Adapter failure → HALT
- Partial execution → ROLLBACK
- Any ambiguity → NO ACTION

### 2. Safety First
- All decisions must be signed
- All decisions must be validated
- All approvals must be present
- All guardrails must pass
- All rate limits must be respected
- All blast radius limits must be enforced

### 3. Execution Only
- Dispatcher does NOT make decisions
- Dispatcher does NOT override policy
- Dispatcher does NOT execute blindly
- Dispatcher ONLY executes signed, validated, approved decisions

## Enforcement Flow

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

Any failure at any step → NO ACTION

## Decision Structure

The dispatcher expects a `PolicyDecision` JSON structure with:
- `decision_id`: Unique identifier
- `policy_signature`: Cryptographic signature (RSA-4096-PSS-SHA256)
- `decision_hash`: Integrity hash
- `required_approvals`: List of required approval types
- `allowed_actions`: List of permitted actions
- `asset_class`: Asset classification
- `kill_chain_stage`: MITRE ATT&CK stage
- `severity`: Alert severity

## Safety Guards

### Max Hosts Per Action
- Default: 10 hosts
- Configurable via `RANSOMEYE_ENFORCEMENT_MAX_HOSTS`
- Prevents mass enforcement

### Max Actions Per Window
- Default: 100 actions per hour
- Configurable via `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_MAX_ACTIONS`
- Prevents rate limit abuse

### Asset Class Restrictions
- Default: production, staging, development
- Configurable via `RANSOMEYE_ENFORCEMENT_ALLOWED_ASSET_CLASSES`
- Prevents enforcement on unauthorized assets

### Environment Constraints
- Default: production, staging, development
- Configurable via `RANSOMEYE_ENFORCEMENT_ALLOWED_ENVIRONMENTS`
- Prevents enforcement in wrong environment

### Destructive Action Approval
- Destructive actions (block, isolate, quarantine, deny) require explicit approval
- Configurable via `RANSOMEYE_ENFORCEMENT_DESTRUCTIVE_REQUIRES_APPROVAL`

## Approval Workflows

### Approval Types
- `operator`: Human operator approval
- `manager`: Management approval
- `security`: Security team approval
- `automated`: Automated approval (for low-risk actions)

### Approval Process
1. Decision requires approval → Execution HELD
2. Approval recorded via `record_approval()`
3. Execution proceeds after all approvals present

## Rate Limiting

### Rate Limit Windows
- Default window: 3600 seconds (1 hour)
- Configurable via `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_WINDOW_SECONDS`
- Per-decision rate limiting

### Rate Limit Keys
- Format: `enforcement:{decision_id}`
- Tracks actions per decision
- Resets after window expires

## Blast Radius Limiting

### Blast Radius Windows
- Default max hosts: 50 per window
- Configurable via `RANSOMEYE_ENFORCEMENT_BLAST_RADIUS_MAX_HOSTS`
- Prevents mass enforcement across assets

### Blast Radius Keys
- Format: `blast_radius:{decision_id}`
- Tracks affected hosts per decision
- Resets after window expires

## Rollback

### Rollback Support
- Automatic rollback command generation
- Rollback records stored per execution
- Rollback available for reversible actions

### Rollback Process
1. Execution recorded with rollback commands
2. Rollback ID returned in execution result
3. Rollback executed via `rollback(execution_id)`

## Dry-Run Mode

### Dry-Run Simulation
- Simulates execution without actual enforcement
- Validates all safety checks
- Returns execution plan
- No actual actions taken

### Dry-Run Usage
- Test enforcement before execution
- Validate safety checks
- Preview execution plan
- No risk of actual enforcement

## Platform Adapters

### Linux Agent Adapter
- Executes via Linux agent API
- Supports: block, isolate, quarantine, monitor
- Generates iptables commands
- Configurable via `RANSOMEYE_LINUX_AGENT_API_URL`

### Windows Agent Adapter
- Executes via Windows agent API
- Supports: block, isolate, quarantine, monitor
- Generates PowerShell commands
- Configurable via `RANSOMEYE_WINDOWS_AGENT_API_URL`

### Network Adapter
- Executes at network level
- Supports: block, isolate, quarantine, monitor
- Generates network commands
- Configurable via `RANSOMEYE_NETWORK_API_URL`

## Configuration

### Required Environment Variables
- `RANSOMEYE_POLICY_PUBLIC_KEY_PATH`: Path to policy public key
- `RANSOMEYE_REVOCATION_LIST_PATH`: Path to revocation list (optional)

### Optional Environment Variables
- `RANSOMEYE_ENFORCEMENT_MAX_HOSTS`: Max hosts per action (default: 10)
- `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_MAX_ACTIONS`: Max actions per window (default: 100)
- `RANSOMEYE_ENFORCEMENT_RATE_LIMIT_WINDOW_SECONDS`: Rate limit window (default: 3600)
- `RANSOMEYE_ENFORCEMENT_BLAST_RADIUS_MAX_HOSTS`: Max hosts per blast radius (default: 50)
- `RANSOMEYE_ENFORCEMENT_ALLOWED_ASSET_CLASSES`: Allowed asset classes (comma-separated)
- `RANSOMEYE_ENFORCEMENT_ALLOWED_ENVIRONMENTS`: Allowed environments (comma-separated)

## Error Handling

### Error Types
- `UnsignedDecision`: Decision missing or empty signature
- `InvalidSignature`: Signature verification failed
- `DecisionRevoked`: Decision has been revoked
- `MissingApproval`: Required approval not present
- `GuardrailViolation`: Safety guardrail violated
- `BlastRadiusExceeded`: Blast radius limit exceeded
- `RateLimitExceeded`: Rate limit exceeded
- `AdapterFailure`: Platform adapter failed
- `RollbackFailed`: Rollback execution failed

### Error Response
- All errors result in NO ACTION
- Errors logged with full context
- Errors returned in `EnforcementResult`

## Execution Evidence

### Evidence Structure
- `validator_checks`: List of validation checks performed
- `approval_status`: List of approval statuses
- `guardrail_checks`: List of guardrail checks
- `rate_limit_status`: Rate limit status
- `blast_radius_status`: Blast radius status
- `adapter_response`: Adapter execution response
- `execution_timestamp`: Execution timestamp
- `execution_duration_ms`: Execution duration

## Security

### Signature Verification
- RSA-4096-PSS-SHA256 algorithm
- Public key from `RANSOMEYE_POLICY_PUBLIC_KEY_PATH`
- Signature verified against decision hash

### Revocation Checking
- Revocation list from `RANSOMEYE_REVOCATION_LIST_PATH`
- Revoked decisions rejected immediately
- Revocation list reloaded on startup

### Decision Integrity
- Decision hash verified
- Signature verified
- Revocation status checked
- All checks must pass

