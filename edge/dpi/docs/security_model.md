# DPI Probe Security Model

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_dpi_probe/docs/security_model.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Security architecture and threat model for DPI Probe

---

## Security Posture

DPI Probe operates as an **untrusted component** in the RansomEye architecture. All security measures are designed to:
1. Prevent tampering with telemetry
2. Ensure authenticity of events
3. Protect against replay attacks
4. Maintain trust chain with Core

## Threat Model

### Assumptions
- DPI Probe runs on untrusted host
- Network path may be compromised
- DPI Probe process may be compromised
- Storage may be accessible to attackers

### Threats

#### 1. Telemetry Tampering
- **Threat**: Attacker modifies events in transit
- **Mitigation**: Cryptographic signatures, data hashes
- **Detection**: Signature verification failure

#### 2. Replay Attacks
- **Threat**: Attacker replays old events
- **Mitigation**: Nonce-based replay protection
- **Detection**: Duplicate nonce detection

#### 3. Identity Spoofing
- **Threat**: Attacker impersonates DPI Probe
- **Mitigation**: X.509 certificates, mTLS
- **Detection**: Certificate validation failure

#### 4. Man-in-the-Middle
- **Threat**: Attacker intercepts/modifies traffic
- **Mitigation**: mTLS with certificate pinning
- **Detection**: Certificate chain validation

#### 5. Resource Exhaustion
- **Threat**: Attacker floods DPI Probe
- **Mitigation**: Bounded buffers, drop policies
- **Detection**: Buffer utilization monitoring

## Security Components

### 1. Identity Management

#### Per-Instance Identity
- Unique X.509 certificate per DPI Probe instance
- Certificate stored securely (0600 permissions)
- Identity hash derived from certificate

#### Certificate Lifecycle
- **Generation**: On first run (if not exists)
- **Storage**: Encrypted at rest (filesystem permissions)
- **Rotation**: Manual (replace certificate files)
- **Revocation**: Core maintains revocation list

### 2. Cryptographic Signing

#### Signature Algorithm
- **Algorithm**: RSA-4096-PSS-SHA256
- **Key Size**: 4096 bits
- **Padding**: PSS (Probabilistic Signature Scheme)
- **Hash**: SHA-256

#### Signing Process
1. Extract data to sign
2. Compute SHA-256 hash
3. Sign hash with RSA-4096-PSS
4. Base64 encode signature
5. Include in event envelope

#### Verification
- Core verifies all signatures
- Invalid signatures → event rejected
- Signature failure → DPI Probe identity revoked

### 3. Replay Protection

#### Nonce Generation
- **Length**: 64 hex characters (32 bytes)
- **Entropy**: Cryptographically secure random
- **Uniqueness**: Per-event nonce
- **Storage**: Tracked in memory (bounded)

#### Timestamp Validation
- **Format**: ISO-8601 UTC
- **Window**: Configurable (default: 300 seconds)
- **Validation**: Reject events outside window
- **Clock Skew**: Handled via window

#### Detection
- Nonce tracking in memory
- Timestamp window validation
- Duplicate detection
- Automatic rejection

### 4. Transport Security

#### mTLS
- **Protocol**: TLS 1.2+
- **Authentication**: Mutual TLS (client + server)
- **Ciphers**: Strong ciphers only
- **Certificate Validation**: Full chain validation

#### Certificate Validation
- **CA Certificate**: Trust anchor verification
- **Chain Validation**: Full certificate chain
- **Revocation**: CRL/OCSP checking (Core side)
- **Pinning**: Certificate pinning recommended

#### Connection Security
- **Encryption**: All data encrypted in transit
- **Integrity**: TLS MAC ensures integrity
- **Authentication**: Mutual authentication required

### 5. Trust Chain

#### Certificate Hierarchy
```
Root CA
  └─ Intermediate CA
      └─ Core Certificate
      └─ DPI Probe Certificate
```

#### Validation
- DPI Probe certificate signed by CA
- CA certificate trusted by Core
- Chain validation on connection
- Revocation checking

## Security Policies

### 1. Fail-Safe Defaults
- **Unsigned events**: Never transmitted
- **Invalid signatures**: Events dropped
- **Identity failure**: Transmission halted
- **Certificate expiration**: Connection rejected

### 2. Least Privilege
- **File permissions**: 0600 for private keys
- **Process user**: Non-root (where possible)
- **Capabilities**: Minimal required capabilities
- **Network access**: Only to Core API

### 3. Defense in Depth
- **Multiple layers**: Signing, mTLS, replay protection
- **Independent validation**: Each layer validates independently
- **Fail-closed**: Invalid events rejected

### 4. Audit and Logging
- **All failures logged**: Signature, identity, replay
- **Security events**: High-priority logging
- **Metrics**: Security-related metrics tracked
- **Alerts**: Security failures trigger alerts

## Security Best Practices

### Deployment
- **Secure storage**: Protect certificate files
- **Network isolation**: Limit network access
- **Monitoring**: Monitor security events
- **Updates**: Keep dependencies updated

### Configuration
- **Strong keys**: Use 4096-bit RSA keys
- **Secure channels**: Always use mTLS
- **Certificate rotation**: Regular rotation
- **Revocation**: Maintain revocation lists

### Operations
- **Monitoring**: Track security metrics
- **Incident response**: Plan for security incidents
- **Forensics**: Log security events
- **Recovery**: Plan for compromised probes

## Security Monitoring

### Metrics
- **Signature failures**: Count of invalid signatures
- **Replay attempts**: Count of replay attacks
- **Identity failures**: Count of identity validation failures
- **Certificate errors**: Count of certificate errors

### Alerts
- **High failure rate**: > 10 failures/minute
- **Replay attack**: Any duplicate nonce
- **Identity failure**: Certificate validation failure
- **Certificate expiration**: Certificate expiring soon

## Incident Response

### Compromised Probe
1. **Detection**: Security monitoring alerts
2. **Isolation**: Revoke certificate immediately
3. **Investigation**: Logs and forensics
4. **Recovery**: Deploy new probe with new identity

### Compromised Certificate
1. **Detection**: Certificate validation failure
2. **Revocation**: Add to revocation list
3. **Replacement**: Generate new certificate
4. **Deployment**: Deploy new certificate

### Replay Attack
1. **Detection**: Duplicate nonce detected
2. **Blocking**: Event rejected
3. **Alerting**: Security alert triggered
4. **Investigation**: Source analysis
