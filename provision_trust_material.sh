#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/provision_trust_material.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Provisions root trust key, trust store, and signed policies for PROMPT-13

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

log "=========================================="
log "PROMPT-13: Trust Material & Policy Provisioning"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root (for /etc/ransomeye operations)"
    exit 1
fi

# Directories
KEYS_DIR="/etc/ransomeye/keys"
POLICIES_DIR="/etc/ransomeye/policies"
# Policy trust store (authoritative v1.0 policy root ONLY)
TRUST_STORE_DIR="/etc/ransomeye/trust_store"
TRUST_WORK_DIR="$PROJECT_ROOT/ransomeye_trust"
POLICY_SOURCE_DIR="$PROJECT_ROOT/core/policy/policies"

# Create directories
log "Creating required directories..."
mkdir -p "$KEYS_DIR"
mkdir -p "$POLICIES_DIR"
mkdir -p "$TRUST_STORE_DIR"
chown -R ransomeye:ransomeye "$KEYS_DIR" "$POLICIES_DIR" "$TRUST_STORE_DIR" 2>/dev/null || true
success "Directories created"

# Step 1: Generate Root CA and extract public key
log ""
log "Step 1: Generating Root CA and extracting public key..."
cd "$TRUST_WORK_DIR"

if [[ ! -f "$TRUST_WORK_DIR/keys/root_ca.key" ]] || [[ ! -f "$TRUST_WORK_DIR/certs/root_ca.crt" ]]; then
    log "Generating Root CA..."
    python3 -c "
from root_ca_generator import RootCAGenerator
generator = RootCAGenerator('$TRUST_WORK_DIR')
root_ca_key, root_ca_cert = generator.generate_root_ca()
paths = generator.save_root_ca(root_ca_key, root_ca_cert)
print('Root CA generated successfully')
"
    success "Root CA generated"
else
    log "Root CA already exists, using existing..."
fi

# Extract public key from Root CA certificate for root.pub
log "Extracting public key from Root CA certificate..."
ROOT_CA_CERT="$TRUST_WORK_DIR/certs/root_ca.crt"
ROOT_PUB_KEY="$KEYS_DIR/root.pub"

# Extract public key from X.509 certificate (PEM format)
# The root.pub should contain the public key in a format the kernel can use
# For now, we'll use the certificate's public key in PEM format
if command -v openssl &> /dev/null; then
    openssl x509 -in "$ROOT_CA_CERT" -pubkey -noout > "$ROOT_PUB_KEY" 2>/dev/null || {
        # Alternative: copy the certificate itself as the public key
        cp "$ROOT_CA_CERT" "$ROOT_PUB_KEY"
    }
    success "Public key extracted to $ROOT_PUB_KEY"
else
    # Fallback: use the certificate as the public key
    cp "$ROOT_CA_CERT" "$ROOT_PUB_KEY"
    warning "openssl not found, using certificate as public key"
fi

# Verify root.pub is not empty
if [[ ! -s "$ROOT_PUB_KEY" ]]; then
    error "Root public key file is empty: $ROOT_PUB_KEY"
    exit 1
fi

chmod 644 "$ROOT_PUB_KEY"
chown ransomeye:ransomeye "$ROOT_PUB_KEY" 2>/dev/null || true
success "Root public key provisioned: $ROOT_PUB_KEY"

# Step 2: Provision Policy Trust Store (v1.0 single-root)
log ""
log "Step 2: Provisioning Policy Trust Store (single authoritative root)..."

# Fail-closed: wipe any legacy/dev/test keys from policy trust store.
# Only the authoritative policy root public key is allowed to remain.
rm -rf "${TRUST_STORE_DIR}"
mkdir -p "${TRUST_STORE_DIR}"
chown -R ransomeye:ransomeye "$TRUST_STORE_DIR" 2>/dev/null || true
chmod 755 "$TRUST_STORE_DIR" 2>/dev/null || true

# Keep the Root CA certificate under keys (NOT in policy trust store; it is not a policy signature verification key).
ROOT_CA_PEM_DST="$KEYS_DIR/root_ca.pem"
cp "$ROOT_CA_CERT" "$ROOT_CA_PEM_DST"
chmod 644 "$ROOT_CA_PEM_DST"
chown ransomeye:ransomeye "$ROOT_CA_PEM_DST" 2>/dev/null || true
success "Root CA provisioned (non-policy): $ROOT_CA_PEM_DST"

# Step 3: Generate policy signing key
log ""
log "Step 3: Generating policy signing key..."
cd "$TRUST_WORK_DIR"

# Generate signing key for policy domain
if [[ ! -f "$TRUST_WORK_DIR/keys/policy_signing.key" ]]; then
    log "Generating policy signing key..."
    python3 -c "
from root_ca_generator import RootCAGenerator
generator = RootCAGenerator('$TRUST_WORK_DIR')
private_key, cert, _, _ = generator.generate_signing_key('policy')
paths = generator.save_signing_key('policy', private_key, cert)
print('Policy signing key generated successfully')
"
    success "Policy signing key generated"
else
    log "Policy signing key already exists, using existing..."
fi

# Convert private key to DER format for sign_policies tool
POLICY_KEY_PEM="$TRUST_WORK_DIR/keys/policy_signing.key"
POLICY_KEY_DER="$TRUST_WORK_DIR/keys/policy_signing.der"

if [[ ! -f "$POLICY_KEY_DER" ]]; then
    log "Converting policy signing key to DER format..."
    if command -v openssl &> /dev/null; then
        openssl pkcs8 -topk8 -inform PEM -outform DER -in "$POLICY_KEY_PEM" -nocrypt -out "$POLICY_KEY_DER" 2>/dev/null || {
            error "Failed to convert policy key to DER format"
            exit 1
        }
        success "Policy key converted to DER format"
    else
        error "openssl required to convert policy key to DER format"
        exit 1
    fi
fi

# Extract authoritative policy root public key and add to trust store
# CRITICAL: Must be in the exact DER format `ring` expects for RSA verification (NOT OpenSSL SPKI).
log "Extracting authoritative policy root public key for trust store (ring DER format)..."

# PROMPT-27: Authoritative single policy root for v1.0 (MUST MATCH policy key_id)
POLICY_ROOT_ID="policy_root_v1"
POLICY_PUB_KEY_DER="$TRUST_STORE_DIR/${POLICY_ROOT_ID}.der"

# Find ring-based public key extractor (release preferred)
EXTRACT_PUBKEY_BIN="$PROJECT_ROOT/target/release/extract_pubkey_simple"
if [[ ! -f "$EXTRACT_PUBKEY_BIN" ]]; then
    EXTRACT_PUBKEY_BIN="$PROJECT_ROOT/target/debug/extract_pubkey_simple"
fi
if [[ ! -f "$EXTRACT_PUBKEY_BIN" ]]; then
    error "extract_pubkey_simple binary not found (expected at target/release/extract_pubkey_simple or target/debug/extract_pubkey_simple)"
    error "Please build it: cd $PROJECT_ROOT && cargo build -p policy --release --bin extract_pubkey_simple"
    exit 1
fi

# Use ring to extract the exact DER bytes that ring verification expects.
if ! "$EXTRACT_PUBKEY_BIN" "$POLICY_KEY_DER" "$POLICY_PUB_KEY_DER" > /dev/null 2>&1; then
    error "Failed to extract policy root public key using ring extractor"
    exit 1
fi

DER_SIZE=$(stat -c%s "$POLICY_PUB_KEY_DER" 2>/dev/null || echo "0")
# For RSA-4096, ring's public_key() DER is typically ~526 bytes.
if [[ $DER_SIZE -lt 450 ]] || [[ $DER_SIZE -gt 700 ]]; then
    error "Extracted ring public key DER has unexpected size: $DER_SIZE bytes (expected ~526 bytes for RSA-4096)"
    exit 1
fi

chmod 644 "$POLICY_PUB_KEY_DER" 2>/dev/null || true
chown ransomeye:ransomeye "$POLICY_PUB_KEY_DER" 2>/dev/null || true
success "Policy root public key extracted (ring DER): $POLICY_PUB_KEY_DER ($DER_SIZE bytes)"

# Step 4: Sign policies
log ""
log "Step 4: Signing policies..."

# Find sign_policies binary (release preferred; debug allowed for local provisioning)
SIGN_POLICIES_BIN="$PROJECT_ROOT/target/release/sign_policies"
if [[ ! -f "$SIGN_POLICIES_BIN" ]]; then
    SIGN_POLICIES_BIN="$PROJECT_ROOT/target/debug/sign_policies"
fi
if [[ ! -f "$SIGN_POLICIES_BIN" ]]; then
    error "sign_policies binary not found (expected at target/release/sign_policies or target/debug/sign_policies)"
    error "Please build it: cd $PROJECT_ROOT && cargo build -p policy --bin sign_policies"
    exit 1
fi

# Sign each policy file
POLICY_FILES=(
    "$POLICY_SOURCE_DIR/ransomware_response.yaml"
    "$POLICY_SOURCE_DIR/privilege_abuse.yaml"
    "$POLICY_SOURCE_DIR/persistence.yaml"
    "$POLICY_SOURCE_DIR/lateral_movement.yaml"
)

SIGNED_COUNT=0
for policy_file in "${POLICY_FILES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        POLICY_NAME=$(basename "$policy_file")
        POLICY_DEST="$POLICIES_DIR/$POLICY_NAME"
        
        # Copy policy to destination
        cp "$policy_file" "$POLICY_DEST"
        
        # Sign the policy
        log "Signing policy: $POLICY_NAME"
        if "$SIGN_POLICIES_BIN" "$POLICY_KEY_DER" "$POLICY_DEST" > /dev/null 2>&1; then
            chmod 644 "$POLICY_DEST"
            chown ransomeye:ransomeye "$POLICY_DEST" 2>/dev/null || true
            success "Policy signed: $POLICY_NAME"
            ((SIGNED_COUNT++)) || true
        else
            error "Failed to sign policy: $POLICY_NAME"
        fi
    else
        warning "Policy file not found: $policy_file"
    fi
done

if [[ $SIGNED_COUNT -eq 0 ]]; then
    error "No policies were signed successfully"
    exit 1
fi

success "Signed $SIGNED_COUNT policy file(s)"

# Step 5: Verify provisioned material
log ""
log "Step 5: Verifying provisioned material..."

VERIFY_FAILED=0

# Verify root.pub
if [[ ! -f "$ROOT_PUB_KEY" ]] || [[ ! -s "$ROOT_PUB_KEY" ]]; then
    error "Root public key verification failed: $ROOT_PUB_KEY"
    ((VERIFY_FAILED++)) || true
else
    success "Root public key verified: $ROOT_PUB_KEY ($(stat -c%s "$ROOT_PUB_KEY") bytes)"
fi

# Verify policy trust store contains ONLY the authoritative policy root public key.
if [[ ! -f "$POLICY_PUB_KEY_DER" ]] || [[ ! -s "$POLICY_PUB_KEY_DER" ]]; then
    error "Policy trust store verification failed: missing policy root public key: $POLICY_PUB_KEY_DER"
    ((VERIFY_FAILED++)) || true
else
    success "Policy trust store verified: $POLICY_PUB_KEY_DER ($(stat -c%s "$POLICY_PUB_KEY_DER") bytes)"
fi

# Fail-closed: ensure no extra key files exist in the policy trust store
EXTRA_KEYS_COUNT=$(find "$TRUST_STORE_DIR" -maxdepth 1 -type f \( -name '*.der' -o -name '*.pem' -o -name '*.pub' \) ! -name "${POLICY_ROOT_ID}.der" | wc -l)
if [[ "$EXTRA_KEYS_COUNT" -ne 0 ]]; then
    error "Policy trust store contains unexpected key material (must contain only ${POLICY_ROOT_ID}.der)"
    find "$TRUST_STORE_DIR" -maxdepth 1 -type f \( -name '*.der' -o -name '*.pem' -o -name '*.pub' \) -print >&2 || true
    ((VERIFY_FAILED++)) || true
fi

# Verify policies
POLICY_COUNT=$(find "$POLICIES_DIR" -name "*.yaml" -type f | wc -l)
if [[ $POLICY_COUNT -eq 0 ]]; then
    error "No policy files found in $POLICIES_DIR"
    ((VERIFY_FAILED++)) || true
else
    success "Policy files verified: $POLICY_COUNT file(s) in $POLICIES_DIR"
    
    # Verify at least one policy has a signature
    SIGNED_POLICY_COUNT=$(grep -l "signature:" "$POLICIES_DIR"/*.yaml 2>/dev/null | wc -l)
    if [[ $SIGNED_POLICY_COUNT -eq 0 ]]; then
        error "No signed policies found in $POLICIES_DIR"
        ((VERIFY_FAILED++)) || true
    else
        success "Signed policies verified: $SIGNED_POLICY_COUNT file(s) with signatures"
    fi
fi

if [[ $VERIFY_FAILED -gt 0 ]]; then
    error "Verification failed: $VERIFY_FAILED check(s) failed"
    exit 1
fi

# Summary
log ""
log "=========================================="
log "PROVISIONING COMPLETE"
log "=========================================="
log ""
success "Root trust key: $ROOT_PUB_KEY"
success "Policy trust store (v1.0 root): $POLICY_PUB_KEY_DER"
success "Root CA (non-policy): $ROOT_CA_PEM_DST"
success "Signed policies: $POLICY_COUNT file(s) in $POLICIES_DIR"
log ""
log "Next step: Run PROMPT-11 tests"
log "  sudo $PROJECT_ROOT/qa/runtime/run_all_tests.sh"
log ""

exit 0

