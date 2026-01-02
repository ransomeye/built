# Analyst Assistance Prompts

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/llm/prompts/analyst_assist.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** SOC Copilot prompts for analyst assistance

## Overview

This document contains prompts used by the SOC Copilot for analyst assistance. All responses are advisory-only and read-only.

## Prompt Templates

### Risk Assessment
```
Analyze the following alert and provide an advisory risk assessment:

Alert ID: {alert_id}
Context: {context}

Provide:
- Risk score (0-1)
- Confidence bounds
- Key risk factors
- Recommended analyst actions (advisory only)
```

### Context Enrichment
```
Enrich the following alert with relevant context:

Alert ID: {alert_id}
Initial Context: {initial_context}

Provide:
- Related alerts
- Historical patterns
- Threat intelligence matches
- Kill chain stage inference
```

### Explanation Request
```
Explain the following AI output:

Output: {output}
SHAP Explanation: {shap}

Provide:
- Feature importance ranking
- Contribution interpretation
- Confidence assessment
- Analyst guidance
```

## Response Format

### Advisory Response
- **Type:** Advisory only
- **Impact:** None on policy or enforcement
- **Usage:** Analyst reference

### Read-Only
- No state modification
- No enforcement actions
- No policy changes

## Best Practices

### Prompt Design
- Clear and specific
- Include context
- Request structured output
- Emphasize advisory nature

### Response Validation
- Verify advisory-only
- Check for state modification attempts
- Validate response format
- Log all interactions

