# Advisory Scope

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/docs/advisory_scope.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Advisory scope documentation - strict non-authority guarantees

## Overview

The RansomEye AI Advisory system provides **advisory-only assistance** to security analysts. It MUST NOT influence policy decisions, trigger enforcement, or modify system state.

## Core Principle

**AI is an advisor, not an authority.**

## Advisory-Only Guarantees

### 1. Read-Only Access
- All AI operations are read-only
- No write access to Control Plane
- No write access to Enforcement Plane
- No state modification

### 2. Non-Blocking
- AI outputs are non-blocking
- Core operations continue if AI is down
- AI failures do not impact Core functionality

### 3. Ignorable
- AI outputs can be ignored without impact
- Core does not depend on AI
- Policy decisions are independent of AI

## Advisory Functions

### Risk Scoring
- **Purpose:** Provide risk scores for alerts
- **Output:** Advisory score with confidence bounds
- **Usage:** Analyst reference only
- **Impact:** None on policy or enforcement

### Context Enrichment
- **Purpose:** Enrich alert context with related information
- **Output:** Related alerts, historical context, threat intel matches
- **Usage:** Analyst reference only
- **Impact:** None on policy or enforcement

### Explainability (SHAP)
- **Purpose:** Explain AI outputs
- **Output:** SHAP explanations for all numeric outputs
- **Usage:** Analyst understanding
- **Impact:** None on policy or enforcement

### SOC Copilot
- **Purpose:** Provide analyst assistance
- **Output:** Read-only responses to analyst queries
- **Usage:** Analyst reference only
- **Impact:** None on policy or enforcement

## Non-Authority Guarantees

### Policy Decisions
- AI does NOT influence policy decisions
- Policy engine operates independently
- AI outputs are advisory only

### Enforcement
- AI does NOT trigger enforcement
- Enforcement dispatcher operates independently
- AI outputs cannot cause enforcement actions

### State Modification
- AI does NOT modify system state
- All AI operations are read-only
- No database writes from AI

## Failure Modes

### AI Disabled
- **Cause:** Missing baseline, unsigned model, runtime error
- **Impact:** AI subsystem disabled
- **Core Impact:** None - Core continues operating

### Missing SHAP
- **Cause:** SHAP generation failure
- **Impact:** Output blocked
- **Core Impact:** None

### Unsigned Model
- **Cause:** Model not signed
- **Impact:** AI disabled
- **Core Impact:** None

## Architecture Guarantees

### Separation of Concerns
- AI Advisory is separate from Policy Engine
- AI Advisory is separate from Enforcement Dispatcher
- No direct communication between AI and enforcement

### Fail-Closed Behavior
- AI failures disable AI subsystem only
- Core operations continue
- No impact on policy or enforcement

### Read-Only Design
- All AI operations are read-only
- No state modification
- No write access to critical systems

## Compliance

### Certifications
- AI Advisory does not affect security certifications
- Advisory-only design ensures compliance
- No authority leakage possible

### Trust
- AI outputs are advisory only
- No trust required for policy decisions
- No trust required for enforcement

## Testing

### Advisory-Only Tests
- Verify AI cannot influence enforcement
- Verify AI outputs are read-only
- Verify Core operates when AI is down

### Non-Authority Tests
- Verify AI cannot trigger enforcement
- Verify AI cannot modify state
- Verify AI cannot influence policy

## Monitoring

### Metrics
- AI advisory request count
- AI output generation rate
- AI disable events
- SHAP generation success rate

### Alerts
- AI subsystem disabled
- Missing SHAP in outputs
- Unsigned model detected
- Runtime errors

