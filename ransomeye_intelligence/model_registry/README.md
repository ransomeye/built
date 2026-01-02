# AI/ML Model Registry

**Path:** `/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Canonical AI/ML Model Registry - Only explicitly registered models are governed

## Overview

This is the **ONLY** authoritative source for governed AI/ML models in RansomEye. Only models explicitly listed in `registry.json` are subject to:

- SHAP validation requirements
- Metadata validation requirements
- Signature verification requirements
- Version tracking
- Governance enforcement

## Registry Manifest Schema

Each model entry in `registry.json` MUST include:

```json
{
  "model_id": "unique-identifier",
  "phase": 3,
  "path": "relative/path/to/model.pkl",
  "hash": "sha256-hash",
  "trained_on": "dataset-identifier",
  "version": "semver-version",
  "requires_shap": true,
  "signature": "ed25519-signature"
}
```

## Exclusion Rules

The following are **EXPLICITLY EXCLUDED** from validation:

- `.venv/` directories
- `site-packages/` directories
- `tests/` directories
- Dependency files (e.g., `distutils-precedence.pth`, `joblib_*.pkl`)
- Test artifacts
- Any file not explicitly listed in `registry.json`

## Registry Location

**Canonical Path:** `/home/ransomeye/rebuild/ransomeye_intelligence/model_registry/`

Only files under this path (relative paths in registry.json) are considered models.

## Validation Rules

1. **Unsigned or unlisted models → IGNORED** (not validated)
2. **Registered model without SHAP → FAIL**
3. **Registered model without metadata → FAIL**
4. **Registered model without signature → FAIL**
5. **Registered model with all artifacts → PASS**

## Usage

Models must be explicitly registered in `registry.json` to be governed. Unregistered model files (even if they exist) are ignored by the validator.

## Status

✅ **Registry Established** - Ready for model registration

