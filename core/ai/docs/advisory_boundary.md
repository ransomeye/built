# Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/docs/advisory_boundary.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Documentation of advisory-only boundary policy - AI cannot influence enforcement

# RansomEye AI Advisory - Advisory-Only Boundary Policy

## Overview

The `ransomeye_ai_advisory` module provides **advisory-only** AI capabilities. This module is strictly prohibited from performing any enforcement, policy execution, or dispatcher actions.

## Core Principle

**AI CANNOT INFLUENCE ENFORCEMENT**

The AI advisory system provides:
- Recommendations
- Explanations (SHAP-based)
- Context retrieval (RAG)
- Risk assessments
- Threat intelligence summaries

The AI advisory system **MUST NOT**:
- Execute enforcement actions
- Call policy engines
- Invoke dispatcher modules
- Modify system state directly
- Trigger automated responses

## Architecture Boundary

```
┌─────────────────────────────────────┐
│   ransomeye_ai_advisory             │
│   (Advisory-Only Zone)              │
├─────────────────────────────────────┤
│   ✓ Inference (recommendations)     │
│   ✓ Explainability (SHAP)           │
│   ✓ RAG (context retrieval)         │
│   ✗ NO enforcement                  │
│   ✗ NO policy calls                 │
│   ✗ NO dispatcher calls             │
└─────────────────────────────────────┘
           │
           │ (recommendations only)
           ▼
┌─────────────────────────────────────┐
│   Other RansomEye Modules           │
│   (Decision & Enforcement)           │
└─────────────────────────────────────┘
```

## Compile-Time Enforcement

The module includes compile-time tests that verify:
1. No enforcement symbols are accessible
2. No policy execution functions are imported
3. No dispatcher modules are linked
4. Any attempt to access enforcement capabilities results in compile-time failure

## Configuration

All configuration must come from environment variables:
- `MODEL_DIR` (required)
- `ASSISTANT_DATA_DIR` (required)
- `ASSISTANT_MAX_TOKENS` (optional, default: 2048)
- `ASSISTANT_TOPK` (optional, default: 5)

Missing required ENV variables will cause startup failure.

## Testing

Run boundary tests:
```bash
cargo test advisory_boundary_tests
```

These tests verify that the advisory-only boundary is enforced at compile time.
