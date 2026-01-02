#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/core/guardrails/sign_guardrails.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Signs guardrails.yaml with Ed25519 signature

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC_FILE="$SCRIPT_DIR/guardrails.yaml"
PRIVATE_KEY_FILE="$SCRIPT_DIR/guardrails_private_key.pem"
PUBLIC_KEY_FILE="$SCRIPT_DIR/guardrails_public_key.pem"
SIGNATURE_FILE="$SCRIPT_DIR/guardrails.yaml.sig"

# Fail-closed: Check if guardrails.yaml exists
if [[ ! -f "$SPEC_FILE" ]]; then
    echo "Error: guardrails.yaml not found at: $SPEC_FILE" >&2
    exit 1
fi

# Generate key pair if it doesn't exist
if [[ ! -f "$PRIVATE_KEY_FILE" ]]; then
    echo "Generating Ed25519 key pair..."
    # Using openssl to generate Ed25519 key pair
    if ! openssl genpkey -algorithm Ed25519 -out "$PRIVATE_KEY_FILE" 2>/dev/null; then
        echo "Error: openssl with Ed25519 support required" >&2
        exit 1
    fi
    if ! openssl pkey -in "$PRIVATE_KEY_FILE" -pubout -out "$PUBLIC_KEY_FILE"; then
        echo "Error: Failed to extract public key" >&2
        exit 1
    fi
    echo "Key pair generated: $PRIVATE_KEY_FILE, $PUBLIC_KEY_FILE"
fi

# Fail-closed: Check if private key exists
if [[ ! -f "$PRIVATE_KEY_FILE" ]]; then
    echo "Error: Private key not found at: $PRIVATE_KEY_FILE" >&2
    exit 1
fi

# Compute hash of spec (excluding signature and spec_hash)
echo "Computing specification hash..."
export SPEC_FILE
SPEC_HASH=$(python3 << PYTHON_SCRIPT
import yaml
import hashlib
import json
import sys
import os

# Explicitly get arguments from environment or validate
spec_file = os.environ.get('SPEC_FILE')
if not spec_file:
    print("Error: SPEC_FILE environment variable not set", file=sys.stderr)
    sys.exit(1)

if not os.path.exists(spec_file):
    print(f"Error: Specification file not found: {spec_file}", file=sys.stderr)
    sys.exit(1)

try:
    with open(spec_file, 'r', encoding='utf-8') as f:
        spec = yaml.safe_load(f)
    
    if spec is None:
        print("Error: Failed to parse YAML file", file=sys.stderr)
        sys.exit(1)
    
    # Remove signature fields for hashing
    spec_for_hash = spec.copy()
    spec_for_hash['spec_hash'] = ''
    spec_for_hash['signature'] = ''
    spec_for_hash['public_key'] = ''
    
    # Serialize to JSON with sorted keys for deterministic hashing
    json_str = json.dumps(spec_for_hash, sort_keys=True, ensure_ascii=False)
    
    # Compute SHA-256 hash (UTF-8 encoding, binary mode)
    hasher = hashlib.sha256()
    hasher.update(json_str.encode('utf-8'))
    spec_hash = hasher.hexdigest()
    
    # Update the spec with hash
    spec['spec_hash'] = spec_hash
    
    # Write back (without signature for now)
    with open(spec_file, 'w', encoding='utf-8') as f:
        yaml.dump(spec, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
    
    print(spec_hash)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_SCRIPT
)

# Fail-closed: Check if hash computation succeeded
if [[ -z "$SPEC_HASH" ]]; then
    echo "Error: Failed to compute specification hash" >&2
    exit 1
fi

echo "  Computed hash: $SPEC_HASH"

# Sign the hash (convert hex to binary first, use temp file for openssl)
echo "Signing specification hash..."
HASH_BIN=$(mktemp)
echo -n "$SPEC_HASH" | xxd -r -p > "$HASH_BIN"
SIGNATURE_B64=$(openssl pkeyutl -sign -inkey "$PRIVATE_KEY_FILE" -rawin -in "$HASH_BIN" 2>/dev/null | base64 -w 0)
rm -f "$HASH_BIN"

# Fail-closed: Check if signing succeeded
if [[ -z "$SIGNATURE_B64" ]]; then
    echo "Error: Failed to sign specification hash" >&2
    exit 1
fi

# Extract public key in hex format for YAML
PUBLIC_KEY_HEX=$(openssl pkey -in "$PRIVATE_KEY_FILE" -pubout -outform DER 2>/dev/null | xxd -p -c 256 | tr -d '\n')

# Fail-closed: Check if public key extraction succeeded
if [[ -z "$PUBLIC_KEY_HEX" ]]; then
    echo "Error: Failed to extract public key" >&2
    exit 1
fi

# Update YAML with signature and public key
echo "Updating specification with signature..."
export SPEC_FILE SIGNATURE_B64 PUBLIC_KEY_HEX
python3 << PYTHON_SCRIPT
import yaml
import sys
import os

# Explicitly get arguments from environment
spec_file = os.environ.get('SPEC_FILE')
signature = os.environ.get('SIGNATURE_B64')
public_key_hex = os.environ.get('PUBLIC_KEY_HEX')

# Fail-closed: Validate all inputs
if not spec_file:
    print("Error: SPEC_FILE environment variable not set", file=sys.stderr)
    sys.exit(1)

if not signature:
    print("Error: SIGNATURE_B64 environment variable not set", file=sys.stderr)
    sys.exit(1)

if not public_key_hex:
    print("Error: PUBLIC_KEY_HEX environment variable not set", file=sys.stderr)
    sys.exit(1)

if not os.path.exists(spec_file):
    print(f"Error: Specification file not found: {spec_file}", file=sys.stderr)
    sys.exit(1)

try:
    with open(spec_file, 'r', encoding='utf-8') as f:
        spec = yaml.safe_load(f)
    
    if spec is None:
        print("Error: Failed to parse YAML file", file=sys.stderr)
        sys.exit(1)
    
    spec['signature'] = signature
    spec['public_key'] = public_key_hex
    
    with open(spec_file, 'w', encoding='utf-8') as f:
        yaml.dump(spec, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
    
    print("Guardrails specification signed successfully")
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_SCRIPT

# Fail-closed: Check if Python script succeeded
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to update specification with signature" >&2
    exit 1
fi

# Write signature to separate file for verification
echo "$SIGNATURE_B64" > "$SIGNATURE_FILE"

# Verify signature using the Rust verifier (if available) or Python
echo "Verifying signature..."
export SPEC_FILE PUBLIC_KEY_FILE
VERIFICATION_RESULT=$(python3 << PYTHON_SCRIPT
import yaml
import hashlib
import json
import base64
import sys
import os

spec_file = os.environ.get('SPEC_FILE')
public_key_file = os.environ.get('PUBLIC_KEY_FILE')

if not spec_file or not public_key_file:
    print("Error: Missing required environment variables", file=sys.stderr)
    sys.exit(1)

try:
    # Load spec
    with open(spec_file, 'r', encoding='utf-8') as f:
        spec = yaml.safe_load(f)
    
    if spec is None:
        print("Error: Failed to parse YAML file", file=sys.stderr)
        sys.exit(1)
    
    # Get signature and public key from spec
    signature_b64 = spec.get('signature', '')
    public_key_hex = spec.get('public_key', '')
    stored_hash = spec.get('spec_hash', '')
    
    if not signature_b64 or not public_key_hex or not stored_hash:
        print("Error: Missing signature, public key, or hash in specification", file=sys.stderr)
        sys.exit(1)
    
    # Compute hash (same as signing)
    spec_for_hash = spec.copy()
    spec_for_hash['spec_hash'] = ''
    spec_for_hash['signature'] = ''
    spec_for_hash['public_key'] = ''
    
    json_str = json.dumps(spec_for_hash, sort_keys=True, ensure_ascii=False)
    hasher = hashlib.sha256()
    hasher.update(json_str.encode('utf-8'))
    computed_hash = hasher.hexdigest()
    
    # Verify hash matches
    if computed_hash != stored_hash:
        print("Error: Hash mismatch - specification may have been tampered", file=sys.stderr)
        sys.exit(1)
    
    # Verify signature using openssl (via subprocess)
    import subprocess
    import tempfile
    
    # Decode signature
    signature_bytes = base64.b64decode(signature_b64)
    
    # Convert hash to binary
    hash_bytes = bytes.fromhex(computed_hash)
    
    # Create temp files for message and signature
    with tempfile.NamedTemporaryFile(delete=False, mode='wb') as msg_file:
        msg_file.write(hash_bytes)
        msg_path = msg_file.name
    
    with tempfile.NamedTemporaryFile(delete=False, mode='wb') as sig_file:
        sig_file.write(signature_bytes)
        sig_path = sig_file.name
    
    try:
        # Verify signature using openssl pkeyutl
        # Use the public key PEM file directly
        result = subprocess.run(
            ['openssl', 'pkeyutl', '-verify', '-pubin', '-inkey', public_key_file, '-rawin', '-in', msg_path, '-sigfile', sig_path],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print("SUCCESS")
        else:
            print(f"Error: Signature verification failed: {result.stderr}", file=sys.stderr)
            sys.exit(1)
    finally:
        os.unlink(msg_path)
        os.unlink(sig_path)
        
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_SCRIPT
)

# Fail-closed: Check verification result
if [[ "$VERIFICATION_RESULT" != "SUCCESS" ]]; then
    echo "Error: Signature verification failed" >&2
    exit 1
fi

echo "✓ Guardrails specification signed and verified"
echo "  Spec hash: $SPEC_HASH"
echo "  Public key: $PUBLIC_KEY_FILE"
echo "  Private key: $PRIVATE_KEY_FILE (KEEP SECURE)"
echo "  Signature file: $SIGNATURE_FILE"
echo ""
echo "Signature verification: SUCCESS"
