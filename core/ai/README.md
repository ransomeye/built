# RansomEye AI Advisory

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/README.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Phase 8 - AIML Inference, Explainability & Analyst Assistance

## Overview

The RansomEye AI Advisory system provides **advisory-only assistance** to security analysts. It provides risk scoring, context enrichment, explainability (SHAP), and SOC Copilot functionality.

## Core Principles

### Advisory-Only
- AI outputs are advisory ONLY
- AI does NOT influence policy decisions
- AI does NOT trigger enforcement
- AI does NOT modify state

### Fail-Closed
- Missing baseline → AI DISABLED
- Missing SHAP → OUTPUT BLOCKED
- Unsigned model → AI DISABLED
- Runtime error → AI DISABLED

### Mandatory SHAP
- SHAP required for all numeric outputs
- Outputs without SHAP are blocked
- SHAP validated before output

## Architecture

### Components

1. **AdvisoryEngine** - Main orchestrator
2. **RiskScorer** - Advisory risk scoring
3. **SHAPExplainer** - Mandatory SHAP generation
4. **ContextEnricher** - Read-only context enrichment
5. **SOCCopilot** - Read-only analyst assistance
6. **ModelRegistry** - Signed baseline model management
7. **AIController** - AI state management

### Advisory Functions

- **Risk Scoring:** Advisory risk scores with confidence bounds
- **Context Enrichment:** Related alerts, historical context, threat intel
- **Explainability:** SHAP explanations for all outputs
- **SOC Copilot:** Read-only analyst assistance

## Safety Guarantees

### Non-Authority
- AI cannot influence policy decisions
- AI cannot trigger enforcement
- AI cannot modify state
- AI is advisory-only

### Fail-Closed
- Any failure → AI DISABLED
- Core operations continue
- No impact on policy or enforcement

### Read-Only
- All AI operations are read-only
- No write access to Control Plane
- No write access to Enforcement Plane

## Configuration

### Required Environment Variables
- `RANSOMEYE_AI_MODELS_DIR`: Models directory
- `RANSOMEYE_AI_MODEL_PUBLIC_KEY_PATH`: Public key path

### Optional Environment Variables
- `RANSOMEYE_AI_REVOCATION_LIST_PATH`: Revocation list path
- `RANSOMEYE_AI_DATA_DIR`: Data directory

## Usage

### Basic Usage
```rust
use ransomeye_ai_advisory::AdvisoryEngine;

let engine = AdvisoryEngine::new()?;
let output = engine.generate_advisory("alert-1", &features).await?;
```

### Verify SHAP
```rust
if output.has_shap() {
    // SHAP is present
}
```

### Check AI State
```rust
if engine.is_enabled()? {
    // AI is enabled
}
```

## Testing

### Test Suite
- Advisory-only tests
- SHAP required tests
- Unsigned model rejection tests
- AI disable on failure tests

### Running Tests
```bash
cargo test
```

## Documentation

- [Advisory Scope](docs/advisory_scope.md)
- [Explainability](docs/explainability.md)
- [AI Failure Modes](docs/ai_failure_modes.md)
- [Governance](docs/governance.md)

## Security

### Model Signing
- RSA-4096-PSS-SHA256 algorithm
- Public key from environment
- Signature verified on load

### Revocation Checking
- Revocation list from environment
- Revoked models rejected
- Revocation list reloaded on startup

### Integrity Checking
- Model hash verified
- Signature verified
- Revocation status checked

## License

Copyright © RansomEye.Tech | Support: Gagan@RansomEye.Tech

