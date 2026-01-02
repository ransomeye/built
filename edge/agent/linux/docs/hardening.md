# Path and File Name : /home/ransomeye/rebuild/ransomeye_linux_agent/docs/hardening.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Security hardening documentation

# Security Hardening

## Component Identity

- **Unique Identity**: Each agent instance has unique component ID
- **Identity Hash**: SHA-256 hash verification
- **Fail-Closed**: Agent fails to start if identity invalid

## Event Signing

- **Ed25519 Signatures**: Every event signed
- **Replay Protection**: Sequence numbers prevent replay attacks
- **Fail-Closed**: Events rejected if signature invalid

## Attestation

- **Component Attestation**: Trust verification
- **Hash-Based Integrity**: Attestation hash verification
- **Timestamp Validation**: Attestation timestamp checked

## Security Boundaries

### NO Enforcement
- Agent does NOT perform enforcement actions
- Agent does NOT kill processes
- Agent does NOT block syscalls
- Agent does NOT modify system state

### NO Policy Logic
- Agent does NOT make policy decisions
- Agent does NOT evaluate rules
- Agent does NOT trigger responses

### NO Kill-Switch Authority
- Agent does NOT have kill-switch capability
- Agent does NOT control system shutdown
- Agent does NOT override system controls

## Hardening Measures

- **Privilege Separation**: Agent runs with minimal privileges
- **Capability Dropping**: Drop unnecessary capabilities
- **Resource Limits**: Bounded memory and CPU usage
- **Input Validation**: All inputs validated
- **Output Sanitization**: All outputs sanitized

## Threat Model

### Protected Against
- **Replay Attacks**: Sequence numbers prevent replay
- **Tampering**: Signatures prevent tampering
- **Identity Spoofing**: Identity hash prevents spoofing
- **Resource Exhaustion**: Bounded memory prevents exhaustion

### Not Protected Against
- **Kernel Compromise**: Agent cannot protect against kernel compromise
- **Hardware Attacks**: Agent cannot protect against hardware attacks
- **Physical Access**: Agent cannot protect against physical access

