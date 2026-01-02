# Hardening Guide

**Path and File Name:** /home/ransomeye/rebuild/ransomeye_windows_agent/docs/hardening.md  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Security hardening recommendations for Windows Agent deployment

## Identity Security

### Key Management
- Ed25519 signing keys stored in `%PROGRAMDATA%\RansomEye\agent\`
- Keys protected by Windows file permissions
- Keys should be backed up securely
- Key rotation recommended periodically

### Component Identity
- Component ID is unique per agent instance
- Identity file protected by Windows permissions
- Fail-closed on identity validation failure

## Network Security

### mTLS Configuration
- Agent uses mTLS for communication with Core
- Certificate validation enforced
- No unencrypted communication

### Network Isolation
- Agent can operate in air-gapped environments
- Buffering supports offline operation
- No external API dependencies

## Process Security

### Privileges
- Agent runs with minimal required privileges
- No kernel driver required
- ETW access only (standard Windows permissions)

### Process Isolation
- Agent runs as separate process
- No shared memory with other processes
- Bounded memory usage

## Configuration Security

### Configuration Validation
- All configuration validated at startup
- Invalid configuration causes startup failure
- No runtime configuration changes

### Environment Variables
- No hardcoded credentials
- Configuration via environment or file
- Sensitive data not logged

## Operational Security

### Logging
- No sensitive data in logs
- Log rotation and retention
- Log integrity via signing

### Monitoring
- Health status monitoring
- Event processing metrics
- Failure alerting

## Compliance

### Data Handling
- No data enrichment
- No inference or AI decisions
- Telemetry emission only

### Audit Trail
- All events signed and sequenced
- Replay protection enabled
- Immutable event history

