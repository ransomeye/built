# Blast Radius

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dispatcher/docs/blast_radius.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Blast radius limiting documentation

## Blast Radius Limits

- Per-action rate limits
- Per-entity rate limits
- Global execution ceilings
- Max hosts per action

## Configuration

- `RANSOMEYE_DISPATCHER_MAX_ACTIONS_PER_WINDOW`: Max actions per window (default: 100)
- `RANSOMEYE_DISPATCHER_RATE_LIMIT_WINDOW_SECONDS`: Window duration (default: 3600)
- `RANSOMEYE_DISPATCHER_MAX_GLOBAL_PER_WINDOW`: Global ceiling (default: 1000)

## Enforcement

All limits are enforced before execution. Exceeding limits → REJECT.

