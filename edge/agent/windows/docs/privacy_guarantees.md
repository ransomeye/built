# Privacy Guarantees

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/docs/privacy_guarantees.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Privacy guarantees and data handling for Windows Agent

## Data Collection Scope

The Windows Agent collects **telemetry only** - no enforcement, no policy decisions, no remediation.

### Collected Data
- Process events (create, terminate, command line)
- Filesystem events (rename, delete, permission changes, mass writes)
- Registry events (autoruns, persistence keys)
- Network events (connect, disconnect)

### NOT Collected
- File contents
- User credentials
- Personal data
- Application data
- Network payloads

## Data Transmission

- All events are **signed** with Ed25519
- Events sent to Phase 4 ingestion pipeline via mTLS
- No local storage of telemetry (except buffering when Core unavailable)

## Data Retention

- **Local:** No retention - events are sent immediately
- **Buffered Events:** Deleted after successful transmission
- **Identity:** Stored locally (component ID, key ID only)

## Security

- Component identity enforced at startup
- Fail-closed on identity or signing failure
- Replay protection via sequence numbers
- No data enrichment or inference

## Compliance

- Agent operates in **observation mode only**
- No enforcement actions
- No policy decisions
- No remediation actions
- Telemetry emission only

