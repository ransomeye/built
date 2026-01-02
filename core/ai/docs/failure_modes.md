# Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** AI advisory failure modes and fail-closed behavior

## Fail-Closed Principle

All AI failures result in AI DISABLED, core operations continue.

## Failure Scenarios

### Model Loading Failures
- Missing model → AI DISABLED
- Invalid signature → AI DISABLED
- Hash mismatch → AI DISABLED
- Size exceeded → AI DISABLED

### Runtime Failures
- Inference error → AI DISABLED
- SHAP computation error → AI DISABLED
- Memory limit exceeded → AI DISABLED

### Integrity Failures
- Index corruption → AI DISABLED
- Manifest invalid → AI DISABLED
- Trust chain broken → AI DISABLED

## Recovery

- Manual re-enable required
- Full integrity check on re-enable
- All models re-verified
- Trust chain re-established

## Impact

- Core operations unaffected
- Policy engine continues
- Enforcement continues
- Only advisory outputs disabled

