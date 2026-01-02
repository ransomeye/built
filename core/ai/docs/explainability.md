# Explainability (SHAP)

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/docs/explainability.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** SHAP explainability documentation - mandatory SHAP for all outputs

## Overview

SHAP (SHapley Additive exPlanations) explanations are **MANDATORY** for all numeric AI outputs. Outputs without SHAP are blocked.

## SHAP Requirements

### Mandatory SHAP
- All numeric outputs MUST include SHAP explanations
- SHAP must be generated for every inference
- SHAP must be validated before output

### SHAP Structure
```rust
pub struct SHAPExplanation {
    pub feature_contributions: Vec<FeatureContribution>,
    pub baseline_value: f64,
    pub output_value: f64,
    pub shap_version: String,
    pub explanation_hash: String,
}
```

### Feature Contributions
- Each feature has a contribution value
- Contributions sum to (output - baseline)
- Importance is absolute value of contribution

## SHAP Generation

### Process
1. Compute baseline value
2. Calculate feature contributions
3. Normalize contributions
4. Compute explanation hash
5. Validate SHAP

### Validation
- SHAP must not be empty
- Hash must match computed hash
- Contributions must be valid

## SHAP Usage

### Analyst Understanding
- Analysts can see which features drive outputs
- Feature importance ranking
- Contribution values

### Compliance
- SHAP required for certifications
- Explainability for regulatory compliance
- Transparency for trust

## Failure Modes

### Missing SHAP
- **Cause:** SHAP generation failure
- **Impact:** Output blocked
- **Recovery:** Fix SHAP generation

### Invalid SHAP
- **Cause:** SHAP validation failure
- **Impact:** Output blocked
- **Recovery:** Regenerate SHAP

### Empty SHAP
- **Cause:** No feature contributions
- **Impact:** Output blocked
- **Recovery:** Fix feature extraction

## SHAP Schema

### JSON Schema
```json
{
  "feature_contributions": [
    {
      "feature_name": "string",
      "contribution": 0.0,
      "importance": 0.0
    }
  ],
  "baseline_value": 0.0,
  "output_value": 0.0,
  "shap_version": "1.0.0",
  "explanation_hash": "hex_string"
}
```

## Implementation

### SHAP Generator
- Generates SHAP explanations
- Validates SHAP structure
- Computes explanation hash

### SHAP Validator
- Validates SHAP explanations
- Checks hash integrity
- Verifies feature contributions

## Best Practices

### Feature Naming
- Use descriptive feature names
- Include feature metadata
- Document feature meanings

### Contribution Interpretation
- Positive contributions increase output
- Negative contributions decrease output
- Importance indicates feature impact

### Hash Verification
- Always verify explanation hash
- Detect tampering
- Ensure integrity

