# Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/docs/privacy_guarantees.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details of functionality of this file: Privacy guarantees and data handling

# Privacy Guarantees

## Data Collection

### Collected Data
- Process metadata (PID, PPID, executable path, command line)
- Filesystem metadata (paths, operations, permissions)
- Network metadata (socket operations, addresses, ports)
- Syscall metadata (syscall numbers, arguments)

### Not Collected
- Process memory contents
- File contents
- Network payload
- User data
- Sensitive information

## Data Retention

- **No Persistent Storage**: Events emitted immediately
- **No Local Logging**: No local event storage
- **Memory Only**: Events held in memory only until emission
- **Bounded Memory**: All data structures have size limits

## Data Lifecycle

1. **Capture**: Syscall event captured
2. **Parse**: Extract metadata only
3. **Process**: Create event envelope
4. **Sign**: Ed25519 signature
5. **Emit**: Send to Phase 4 pipeline
6. **Discard**: Event discarded from memory

## Privacy Protection

- **Minimal Data**: Only syscall metadata
- **No Enrichment**: No external data sources
- **No Inference**: No AI/ML processing
- **No Policy Logic**: No decision-making

## Compliance

- **GDPR**: Minimal data collection
- **Privacy by Design**: No payload retention
- **Data Minimization**: Only necessary metadata
- **Transparency**: Clear data collection policy

