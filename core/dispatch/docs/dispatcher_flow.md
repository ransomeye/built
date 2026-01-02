# Dispatcher Flow

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dispatcher/docs/dispatcher_flow.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Dispatcher execution flow documentation

## Flow Diagram

```
1. Receive DirectiveEnvelope from Phase 6
   ↓
2. Enter Reentrancy Guard
   ↓
3. Log Directive Received
   ↓
4. Verify Directive (ALL checks must pass):
   - Structure validation
   - TTL check
   - Replay protection (directive ID)
   - Nonce freshness
   - Signature verification
   - Hash verification
   - Audit receipt verification
   - Preconditions hash
   ↓
5. Check Safety Guards:
   - Allowlist check
   - Denylist check
   - Rate limits
   ↓
6. Resolve Targets (strict):
   - Validate agent identity
   - Validate capability
   - Validate platform
   ↓
7. Deliver to Agents
   ↓
8. Wait for Acknowledgment (with timeout)
   ↓
9. Verify Acknowledgment
   ↓
10. Log Execution Result
```

Any failure at any step → NO ACTION

