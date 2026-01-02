# Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_advisory/docs/model_governance.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Model governance policy - signature verification, integrity checks, fail-closed behavior

# Model Governance Policy

## Overview

All AI models used in RansomEye Phase 8 (AI Advisory) must be:
- Signed with RSA-4096 signatures
- Verified before loading
- Integrity-checked via SHA-256 hashes
- Fail-closed on any verification failure

## Signature Verification

### Requirements

1. **RSA-4096 Signatures**: All models must be signed with RSA-4096 keys
2. **Public Key**: Public key must be stored securely and verified at startup
3. **Manifest Signing**: Model manifests must be signed and verified
4. **Trust Chain**: Root trust key must be established before model loading

### Process

1. Load public key from secure location
2. Verify manifest signature before parsing
3. Verify model file signature (if present)
4. Compute and verify model hash matches manifest
5. Verify model size matches manifest

## Integrity Checks

### Hash Verification

- SHA-256 hash computed for all model files
- Hash stored in manifest
- Hash verified before model loading
- Any mismatch → FAIL-CLOSED

### Size Verification

- Model size verified against manifest
- Size mismatch → FAIL-CLOSED

## Fail-Closed Behavior

### On Verification Failure

1. **AI Advisory Module Disabled**: Module stops accepting requests
2. **Audit Log**: Failure logged with full details
3. **Core Continues**: Other RansomEye modules continue operating
4. **No Partial Loads**: Models are either fully verified and loaded, or not loaded at all

### Error Handling

- Signature verification failure → Module disabled
- Hash mismatch → Module disabled
- Missing manifest → Module disabled
- Corrupted model file → Module disabled

## Memory Limits

- **Core AI Storage**: ≤ 3 GB per model
- **Total Models**: Must not exceed available memory
- **Runtime Memory**: Bounded feature extraction (max 1000 features)

## Model Lifecycle

1. **Deployment**: Model signed and deployed with manifest
2. **Verification**: Signature and integrity verified at startup
3. **Loading**: Model loaded into memory (read-only)
4. **Runtime**: Model used for advisory inference only
5. **Updates**: New models must be signed and verified before replacement

## Compliance

All models must comply with:
- RSA-4096 signature requirement
- SHA-256 integrity requirement
- Memory limit requirement
- Fail-closed behavior requirement
