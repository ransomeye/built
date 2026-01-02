# Policy Language Grammar

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/language/grammar.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Formal grammar definition for RansomEye policy language

## Overview

The RansomEye policy language is a restricted, declarative language designed for deterministic policy evaluation.

## Grammar Rules

### Policy Structure

```
Policy ::= {
    id: String,
    version: String,
    name: String,
    description: String,
    enabled: Boolean,
    priority: Integer,
    match_conditions: [MatchCondition],
    decision: DecisionRule,
    required_approvals: [String],
    signature: String,
    signature_hash: String
}
```

### Match Conditions

```
MatchCondition ::= {
    field: String,
    operator: Operator,
    value: Value
}

Operator ::= "equals" | "contains" | "matches" | "in" | "greater_than" | "less_than"

Value ::= String | Number | Boolean | Array[Value]
```

### Decision Rules

```
DecisionRule ::= {
    action: Action,
    allowed_actions: [Action],
    reasoning: String
}

Action ::= "allow" | "deny" | "quarantine" | "isolate" | "block" | "monitor" | "escalate" | "require_approval"
```

## Restrictions

- No loops
- No dynamic execution
- No external calls
- No user-defined functions
- Deterministic evaluation order only

