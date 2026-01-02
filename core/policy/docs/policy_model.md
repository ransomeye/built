# Policy Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/docs/policy_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Policy model documentation - deterministic policy evaluation

## Overview

The RansomEye Policy Engine is the **ONLY authority** that can decide what constitutes a violation, what action is allowed, and what is forbidden.

## Core Principles

### 1. Determinism
- Same alert + same policies → same decision (always)
- No AI/ML/LLM involvement
- No implicit defaults
- All decisions are replayable

### 2. Signed Policies
- Policies MUST be signed
- Unsigned policy → ENGINE REFUSES TO START
- Policy tampering → verification failure

### 3. Fail-Closed
- Ambiguity → DENY
- Missing context → DENY
- Evaluation error → DENY
- No matching policy → DENY

### 4. Decision Only
- Engine emits enforcement decisions (NOT actions)
- No execution happens here
- Decisions are cryptographically verifiable

## Policy Structure

### Match Conditions
Policies match on:
- Alert metadata (severity, kill-chain stage)
- Asset class
- Producer ID
- Rule IDs

### Decision Rules
Policies specify:
- Primary action (allow, deny, quarantine, isolate, block, monitor, escalate, require_approval)
- Allowed actions (list of permitted actions)
- Required approvals (list of approval types)
- Reasoning (deterministic explanation)

### Priority
- Higher priority policies evaluated first
- Ambiguous policies (same priority) → DENY
- First matching policy wins

## Evaluation Process

1. **Context Validation**
   - Validate evaluation context completeness
   - Missing context → DENY

2. **Policy Matching**
   - Match policies against context
   - Policies evaluated by priority
   - First match wins

3. **Ambiguity Detection**
   - Multiple policies match with same priority → DENY
   - Conflicting decisions → DENY

4. **Decision Creation**
   - Create policy decision
   - Include evidence reference
   - Cryptographic signature

5. **Decision Output**
   - Serialize decision
   - Validate decision integrity
   - Output decision (no execution)

## Failure Modes

- **Unsigned Policy**: Engine refuses to start
- **Policy Ambiguity**: DENY
- **Missing Context**: DENY
- **Evaluation Error**: DENY
- **No Matching Policy**: DENY (default)

## Determinism Guarantees

- Same alert → same decision
- Same policies → same evaluation
- Decisions are reproducible
- Evidence is verifiable

