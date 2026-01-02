# RansomEye Policy Engine

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 6 - Policy Engine & Enforcement Semantics

## Overview

The RansomEye Policy Engine is the **ONLY authority** that can decide what constitutes a violation, what action is allowed, and what is forbidden.

## Core Principles

### Determinism
- Same alert + same policies → same decision (always)
- No AI/ML/LLM involvement
- All decisions are replayable

### Signed Policies
- Policies MUST be signed
- Unsigned policy → ENGINE REFUSES TO START
- Policy tampering → verification failure

### Fail-Closed
- Ambiguity → DENY
- Missing context → DENY
- Evaluation error → DENY
- No matching policy → DENY

### Decision Only
- Engine emits enforcement decisions (NOT actions)
- No execution happens here
- Decisions are cryptographically verifiable

## Architecture

### Components

1. **PolicyEngine** - Main engine coordinator
2. **PolicyEvaluator** - Policy evaluation logic
3. **PolicyLoader** - Signed policy loading
4. **PolicyMatcher** - Policy matching
5. **EvaluationContext** - Evaluation context
6. **PolicyDecision** - Decision output

## Usage

### Configuration

Environment variables:
- `RANSOMEYE_POLICY_PATH` - Path to policies directory (default: `/etc/ransomeye/policy/policies`)

### Evaluating Policies

```rust
use ransomeye_policy::{PolicyEngine, EvaluationContext};

let engine = PolicyEngine::new(
    "/etc/ransomeye/policy/policies",
    "1.0.0"
)?;

let context = EvaluationContext::new(
    "alert_id",
    "critical",
    "actions_on_objectives",
    Some("server".to_string()),
    Some("asset_1".to_string()),
    "producer_1",
    vec!["rule_1".to_string()],
    "evidence_ref",
    serde_json::json!({}),
);

let decision = engine.evaluate(context)?;
```

## Policies

Policies are defined in YAML format and must be signed. See `policies/` directory for examples.

## Decision Output

Every decision includes:
- Decision ID
- Allowed actions
- Required approvals
- Evidence reference
- Policy version
- Signature

## Testing

Run tests with:
```bash
cargo test
```

Test categories:
- Deterministic policy tests
- Unsigned policy rejection tests
- Ambiguity deny tests
- Replay consistency tests

## Documentation

See `docs/` directory for detailed documentation:
- `policy_model.md` - Policy model
- `enforcement_semantics.md` - Enforcement semantics
- `ambiguity_rules.md` - Ambiguity rules
- `failure_modes.md` - Failure modes

## Compliance

This phase fully complies with:
- Phase 1: Core Engine & Installer
- Phase 5: Correlation Engine (consumes alerts)
- Phase 6: Policy Engine (this phase)

## License

© RansomEye.Tech | Support: Gagan@RansomEye.Tech

