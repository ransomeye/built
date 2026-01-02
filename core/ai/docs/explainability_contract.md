# Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/docs/explainability_contract.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Explainability contract - SHAP requirements and rationale alignment

# Explainability Contract

## Overview

All AI advisory outputs must include:
- SHAP-based explanations
- Feature contributions with signal/timestamp references
- Human-readable rationale
- Alignment with deterministic Phase 5 outputs

## SHAP Requirements

### Feature Contributions

Every SHAP explanation must include:
1. **Feature Name**: Identifiable feature name
2. **Contribution Value**: Numerical contribution to output
3. **Signal Reference**: Reference to source signal (if applicable)
4. **Timestamp Reference**: Timestamp of feature observation

### SHAP Validation

SHAP values must satisfy:
- Sum of SHAP values = output - baseline
- All features have contributions
- Contributions match feature count

## Rationale Generation

### Human-Readable Output

Rationale must:
- Use plain language
- Reference top contributing features
- Include confidence level context
- Align with deterministic outputs

### Alignment Requirements

Rationale must:
- **Never contradict** deterministic Phase 5 outputs
- **Reference** same signals and timestamps
- **Support** (not override) deterministic conclusions
- **Explain** AI reasoning without enforcement

## Baseline Requirements

- SHAP baseline must be loaded from file
- Baseline values used for comparison
- Baseline must match feature count
- Default baseline (0.5) if file missing

## Output Format

### SHAP Explanation Structure

```json
{
  "feature_contributions": [
    {
      "feature_name": "feature_0",
      "contribution": 0.15,
      "signal_reference": "signal_0",
      "timestamp_reference": "2024-01-01T00:00:00Z"
    }
  ],
  "baseline_value": 0.5,
  "output_value": 0.75,
  "shap_values": [0.15, 0.05, 0.03, 0.02, 0.0]
}
```

### Rationale Format

```
High confidence anomaly detected based on feature analysis. 
Top contributing factors: feature_0 (contribution: 0.15), 
feature_1 (contribution: 0.05), feature_2 (contribution: 0.03). 
[Aligned with deterministic Phase 5 detection outputs]
```

## Compliance

All explainability outputs must:
- Include SHAP explanations
- Reference features, signals, timestamps
- Generate human-readable rationale
- Align with deterministic outputs
- Never contradict core detection
