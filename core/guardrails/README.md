# RansomEye Guardrails Enforcement Engine - Phase 0

**Path:** `/home/ransomeye/rebuild/core/guardrails/`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** Root-of-trust enforcement layer for RansomEye architecture

## Overview

The Guardrails Enforcement Engine is Phase 0 of RansomEye - a cryptographically verified, fail-closed enforcement layer that ensures the system cannot violate its own architectural rules.

## Components

### 1. Guardrails Specification (`guardrails.yaml`)

The canonical specification defining:
- Allowed phases and modules
- Forbidden modules (phantom detection)
- ENV-only configuration rules
- Cryptographic requirements
- Systemd placement rules
- Model validation requirements
- File header requirements
- Database policy
- Offline requirements

**MUST be cryptographically signed** - any modification invalidates the signature.

### 2. Loader (`loader.rs`)

Loads and parses `guardrails.yaml`. Does NOT verify signature (that's done by verifier).

### 3. Verifier (`verifier.rs`)

Cryptographically verifies the guardrails specification signature using Ed25519.

### 4. Enforcer (`enforcer.rs`)

Runtime enforcement engine that:
- Checks for phantom modules
- Detects hardcoded configurations
- Validates systemd placement
- Enforces model requirements (SHAP, metadata)
- Validates policy signatures
- Checks file headers
- **Fails-closed on any violation**

### 5. CI Validator (`ci_validator.rs`)

CI/CD validation that detects violations before merge:
- Phantom modules
- Forbidden directories
- Hardcoded configs
- Systemd misplacement
- Unsigned artifacts

### 6. Audit Logger (`audit.rs`)

Append-only audit logging with tamper evidence:
- Signed audit records
- Hash chain for integrity
- Violation details with context

## Usage

### Building

```bash
cd /home/ransomeye/rebuild/core/guardrails
cargo build --release
```

### Signing the Specification

**IMPORTANT:** The guardrails.yaml must be signed before use:

```bash
cd /home/ransomeye/rebuild/core/guardrails
./sign_guardrails.sh
```

This will:
1. Generate Ed25519 key pair (if not exists)
2. Compute hash of specification
3. Sign the hash
4. Update guardrails.yaml with signature and public key

### Verification

```bash
ransomeye-guardrails verify
```

### Enforcement

```bash
# Installer context
ransomeye-guardrails enforce --context installer

# Service start context
ransomeye-guardrails enforce --context service --data ransomeye-core

# Model load context
ransomeye-guardrails enforce --context model --data /path/to/model.pkl

# Policy load context
ransomeye-guardrails enforce --context policy --data /path/to/policy.yaml

# Runtime context
ransomeye-guardrails enforce --context runtime

# CI context
ransomeye-guardrails enforce --context ci
```

### CI Validation

```bash
ransomeye-guardrails ci
```

## Integration

### Installer Integration

The installer (`install.sh`) enforces guardrails before proceeding:

```bash
# Section 3.5 in install.sh
ransomeye-guardrails enforce --context installer
```

If guardrails fail, installation is aborted (fail-closed).

### Systemd Integration

All systemd services must include `ExecStartPre`:

```ini
[Service]
ExecStartPre=/usr/bin/ransomeye-guardrails enforce --context service --data SERVICE_NAME
ExecStart=/usr/bin/ransomeye_operations start SERVICE_NAME
```

If guardrails fail, the service will not start.

### CI Integration

Add to CI pipeline:

```yaml
- name: Guardrails Validation
  run: ransomeye-guardrails ci
```

CI must fail on any violation.

## Enforcement Rules

### Phantom Module Detection

Any directory matching `ransomeye_*` or `core_*` that is not in the allowed modules list is considered a phantom module and triggers fail-closed.

### Hardcoded Configuration

The enforcer scans all source files for:
- Hardcoded IP addresses (except localhost)
- Hardcoded URLs
- Hardcoded API keys/tokens
- Hardcoded database credentials

### Systemd Placement

All `.service` files must be in `/home/ransomeye/rebuild/systemd/` except for standalone exceptions (Linux Agent, Windows Agent, DPI Probe).

### Model Requirements

Models must:
- Have allowed format (`.pkl`, `.gguf`, `.onnx`, `.pt`, `.pth`)
- Include SHAP explainability file (`.shap.json`)
- Include metadata file (`.metadata.json`)

### Policy Signing

Policies must be signed if `required_signing_for` includes "policies".

### File Headers

All source files (`.py`, `.rs`, `.yaml`, `.sh`, `.service`, `.ts`, `.tsx`) must include mandatory header:

```
# Path and File Name : /path/to/file
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: <description>
```

## Fail-Closed Behavior

**CRITICAL:** The guardrails engine implements fail-closed behavior:

- Any violation triggers immediate exit (non-zero)
- Audit record is emitted before exit
- System/service/installer cannot proceed
- No bypass paths exist

## Audit Logging

All violations are logged to `/var/log/ransomeye/guardrails_audit.jsonl` with:
- Timestamp
- Violation type
- Phase/module/file context
- Violation details
- Spec hash
- Record hash (for tamper evidence)
- Previous hash (hash chain)

## Testing

Run violation tests:

```bash
cd /home/ransomeye/rebuild/core/guardrails
cargo test
```

Tests intentionally violate guardrails to verify fail-closed behavior.

## Security Considerations

1. **Private Key Security**: The private key (`guardrails_private_key.pem`) must be kept secure. It should NOT be committed to the repository.

2. **Public Key Distribution**: The public key is embedded in `guardrails.yaml` and used for verification.

3. **Signature Verification**: Every enforcement check verifies the specification signature first.

4. **Audit Log Integrity**: Audit logs use hash chains to detect tampering.

5. **No Bypass Paths**: The system is designed with no bypass mechanisms - violations always fail-closed.

## Troubleshooting

### "Guardrail specification signature invalid"

The `guardrails.yaml` file has been modified or corrupted. Re-sign it:

```bash
./sign_guardrails.sh
```

### "Phantom module detected"

A directory exists that matches module naming but isn't in the allowed list. Either:
1. Add it to `allowed_modules` in `guardrails.yaml` and re-sign
2. Remove the phantom module

### "Hardcoded configuration detected"

Source code contains hardcoded IPs, URLs, or credentials. Replace with environment variables.

### "Systemd service file misplaced"

A `.service` file exists outside `/home/ransomeye/rebuild/systemd/`. Move it to the unified directory (unless it's a standalone exception).

## Status

✅ **Phase 0 Complete**

- [x] Guardrails specification created
- [x] Loader implemented
- [x] Verifier implemented
- [x] Enforcer implemented
- [x] CI validator implemented
- [x] Audit logging implemented
- [x] Tests created
- [x] Installer integration
- [x] Systemd integration
- [x] Signing script created

## Next Steps

1. Sign the guardrails.yaml specification
2. Build and install the guardrails binary
3. Verify all systemd services include ExecStartPre
4. Add CI validation to pipeline
5. Test fail-closed behavior

