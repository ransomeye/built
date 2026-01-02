# Policy Language Semantics

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/language/semantics.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Semantic rules for policy evaluation

## Evaluation Semantics

### Policy Matching

A policy matches an evaluation context if ALL match conditions evaluate to true.

### Operator Semantics

- `equals`: Exact value match
- `contains`: String contains substring
- `matches`: Regular expression match
- `in`: Value is member of array
- `greater_than`: Numeric comparison
- `less_than`: Numeric comparison

### Decision Semantics

- **Allow**: Action is permitted
- **Deny**: Action is forbidden (highest precedence)
- **Quarantine**: Isolate and contain
- **Isolate**: Complete isolation
- **Block**: Block access/communication
- **Monitor**: Continue monitoring
- **Escalate**: Escalate to human review
- **RequireApproval**: Require explicit approval

### Conflict Resolution

1. Priority (higher priority wins)
2. Specificity (more conditions wins)
3. Explicit deny (always wins)
4. No action (if unresolvable)

### Fail-Closed Behavior

- Missing policy → DENY
- Invalid signature → ENGINE REFUSES TO START
- Ambiguity → DENY
- Evaluation error → DENY

