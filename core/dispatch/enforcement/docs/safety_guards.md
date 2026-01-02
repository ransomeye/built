# Safety Guards

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_enforcement/docs/safety_guards.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Safety guardrails documentation - fail-closed safety mechanisms

## Overview

Safety guards are **mandatory checks** that prevent unsafe or unauthorized enforcement actions. All guards must pass before execution proceeds.

## Guard Types

### 1. Max Hosts Per Action
**Purpose:** Prevent mass enforcement on too many hosts at once.

**Default:** 10 hosts per action

**Configuration:**
```bash
export RANSOMEYE_ENFORCEMENT_MAX_HOSTS=10
```

**Violation:** Execution rejected with `GuardrailViolation` error.

**Rationale:** Limits impact of a single enforcement action.

### 2. Max Actions Per Window
**Purpose:** Prevent rate limit abuse and mass execution.

**Default:** 100 actions per hour

**Configuration:**
```bash
export RANSOMEYE_ENFORCEMENT_RATE_LIMIT_MAX_ACTIONS=100
export RANSOMEYE_ENFORCEMENT_RATE_LIMIT_WINDOW_SECONDS=3600
```

**Violation:** Execution rejected with `RateLimitExceeded` error.

**Rationale:** Prevents accidental or malicious mass execution.

### 3. Asset Class Restrictions
**Purpose:** Restrict enforcement to authorized asset classes.

**Default:** production, staging, development

**Configuration:**
```bash
export RANSOMEYE_ENFORCEMENT_ALLOWED_ASSET_CLASSES="production,staging,development"
```

**Violation:** Execution rejected with `GuardrailViolation` error.

**Rationale:** Prevents enforcement on unauthorized assets.

### 4. Environment Constraints
**Purpose:** Restrict enforcement to authorized environments.

**Default:** production, staging, development

**Configuration:**
```bash
export RANSOMEYE_ENFORCEMENT_ALLOWED_ENVIRONMENTS="production,staging,development"
```

**Violation:** Execution rejected with `GuardrailViolation` error.

**Rationale:** Prevents enforcement in wrong environment.

### 5. Destructive Action Approval
**Purpose:** Require explicit approval for destructive actions.

**Default:** Enabled

**Destructive Actions:**
- `block`: Block network traffic
- `isolate`: Isolate host from network
- `quarantine`: Quarantine host
- `deny`: Deny access

**Configuration:**
- Controlled via `required_approvals` in policy decision
- Approval types: `operator`, `manager`, `security`, `automated`

**Violation:** Execution held with `MissingApproval` error.

**Rationale:** Prevents accidental destructive actions.

## Blast Radius Limiting

### Purpose
Prevent mass enforcement across multiple assets in a time window.

### Default
- Max hosts per window: 50
- Window duration: 3600 seconds (1 hour)

### Configuration
```bash
export RANSOMEYE_ENFORCEMENT_BLAST_RADIUS_MAX_HOSTS=50
```

### Violation
Execution rejected with `BlastRadiusExceeded` error.

### Rationale
Prevents cascading enforcement across entire infrastructure.

## Guard Execution Order

1. **Decision Validation**
   - Signature verification
   - Hash verification
   - Revocation check

2. **Approval Check**
   - Required approvals present
   - Approval types validated

3. **Guardrail Checks**
   - Max hosts per action
   - Asset class restrictions
   - Environment constraints
   - Destructive action approval

4. **Rate Limiting**
   - Actions per window
   - Per-decision rate limiting

5. **Blast Radius Limiting**
   - Hosts per window
   - Per-decision blast radius

6. **Execution**
   - All guards passed
   - Execution proceeds

## Fail-Closed Behavior

### Any Guard Failure → NO ACTION

- Unsigned decision → REJECT
- Missing approval → HOLD
- Guardrail violation → REJECT
- Rate limit exceeded → REJECT
- Blast radius exceeded → REJECT

### Error Response
All guard failures result in:
- Execution rejected
- Error logged
- Error returned in `EnforcementResult`

## Guard Override

### No Override Allowed
- Guards cannot be bypassed
- Guards cannot be disabled
- Guards are mandatory

### Exception Handling
- Guard failures are logged
- Guard failures are returned
- No silent failures

## Guard Testing

### Test Coverage
- Max hosts per action
- Rate limiting
- Blast radius limiting
- Asset class restrictions
- Environment constraints
- Destructive action approval

### Test Scenarios
- Normal execution (all guards pass)
- Guard violation (execution rejected)
- Rate limit exceeded (execution rejected)
- Blast radius exceeded (execution rejected)
- Missing approval (execution held)

## Guard Monitoring

### Metrics
- Guard violations per hour
- Rate limit hits per hour
- Blast radius limit hits per hour
- Approval holds per hour

### Alerts
- High guard violation rate
- Rate limit abuse detected
- Blast radius limit abuse detected
- Approval workflow delays

## Guard Configuration Best Practices

### Production
- Conservative limits
- Strict asset class restrictions
- Required approvals for destructive actions
- Short rate limit windows

### Staging
- Moderate limits
- Relaxed asset class restrictions
- Optional approvals for destructive actions
- Longer rate limit windows

### Development
- Permissive limits
- Open asset class restrictions
- Automated approvals for destructive actions
- Very long rate limit windows

