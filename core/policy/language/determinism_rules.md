# Determinism Rules

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_policy/language/determinism_rules.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Rules ensuring deterministic policy evaluation

## Determinism Guarantees

### Same Input → Same Output

Given identical:
- Evaluation context
- Policy set
- Engine version

The output decision MUST be identical.

### Evaluation Order

1. Policies sorted by priority (descending)
2. Policies with same priority sorted by specificity (descending)
3. Explicit deny policies take precedence
4. Evaluation stops at first matching policy (unless conflict)

### No Non-Deterministic Operations

- No random number generation
- No current time in conditions (only in metadata)
- No external state queries
- No network calls
- No file system reads (except policy loading at startup)

### Replay Guarantee

All decisions can be replayed with:
- Original context
- Original policy set
- Original engine version

Result will be identical.

