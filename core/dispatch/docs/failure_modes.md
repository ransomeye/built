# Failure Modes

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dispatcher/docs/failure_modes.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Failure modes and handling

## Failure Modes

### Invalid Directive
- **Symptom**: Directive fails validation
- **Action**: REJECT, log rejection
- **Recovery**: Fix directive, resubmit

### Expired Directive
- **Symptom**: TTL exceeded
- **Action**: REJECT
- **Recovery**: Issue new directive

### Replay Attack
- **Symptom**: Duplicate directive ID or nonce
- **Action**: REJECT
- **Recovery**: Use new directive ID/nonce

### Target Resolution Failure
- **Symptom**: No valid agents found
- **Action**: ABORT
- **Recovery**: Register agents or fix target scope

### Acknowledgment Timeout
- **Symptom**: No acknowledgment received
- **Action**: Initiate rollback
- **Recovery**: Investigate agent, rollback execution

### Rollback Failure
- **Symptom**: Rollback execution failed
- **Action**: ESCALATE + ALERT
- **Recovery**: Manual intervention required

