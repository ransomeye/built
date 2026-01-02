#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/provision_artifacts_cert.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Generates and installs artifacts_signing.crt for intelligence service

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TRUST_WORK_DIR="$SCRIPT_DIR/ransomeye_trust"
RUNTIME_TRUST_DIR="/opt/ransomeye/modules/ransomeye_trust"

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
log "PROMPT-31B: Provisioning artifacts_signing.crt"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root (for certificate generation and installation)"
    exit 1
fi

# Step 1: Generate artifacts signing certificate if needed
log "Step 1: Checking/generating artifacts signing certificate..."
cd "$TRUST_WORK_DIR"

ARTIFACTS_KEY="$TRUST_WORK_DIR/keys/artifacts_signing.key"
ARTIFACTS_CERT="$TRUST_WORK_DIR/certs/artifacts_signing.crt"

if [[ ! -f "$ARTIFACTS_KEY" ]] || [[ ! -f "$ARTIFACTS_CERT" ]]; then
    log "Generating artifacts signing certificate..."
    python3 -c "
from root_ca_generator import RootCAGenerator
generator = RootCAGenerator('$TRUST_WORK_DIR')
private_key, cert, _, _ = generator.generate_signing_key('artifacts')
paths = generator.save_signing_key('artifacts', private_key, cert)
print('Artifacts signing certificate generated successfully')
print(f'Key: {paths[\"private_key\"]}')
print(f'Cert: {paths[\"certificate\"]}')
"
    success "Artifacts signing certificate generated"
else
    log "Artifacts signing certificate already exists"
    success "Using existing certificate"
fi

# Verify certificate exists and is valid
if [[ ! -f "$ARTIFACTS_CERT" ]] || [[ ! -s "$ARTIFACTS_CERT" ]]; then
    error "Artifacts signing certificate not found or empty: $ARTIFACTS_CERT"
    exit 1
fi

# Verify it's a valid PEM certificate
if ! grep -q "BEGIN CERTIFICATE" "$ARTIFACTS_CERT"; then
    error "Artifacts signing certificate is not a valid PEM certificate"
    exit 1
fi

success "Artifacts signing certificate verified: $ARTIFACTS_CERT ($(stat -c%s "$ARTIFACTS_CERT") bytes)"

# Step 2: Install certificate to runtime location
log ""
log "Step 2: Installing certificate to runtime location..."

# Create runtime trust directory structure
RUNTIME_CERTS_DIR="$RUNTIME_TRUST_DIR/certs"
mkdir -p "$RUNTIME_CERTS_DIR"
chmod 755 "$RUNTIME_TRUST_DIR"
chmod 755 "$RUNTIME_CERTS_DIR"

# Copy certificate
RUNTIME_CERT="$RUNTIME_CERTS_DIR/artifacts_signing.crt"
cp "$ARTIFACTS_CERT" "$RUNTIME_CERT"

# Set correct permissions and ownership
chmod 644 "$RUNTIME_CERT"
chown ransomeye:ransomeye "$RUNTIME_CERT" 2>/dev/null || {
    # If ransomeye user doesn't exist yet, set ownership to root:root
    chown root:root "$RUNTIME_CERT"
    warning "ransomeye user not found, certificate owned by root:root"
}

success "Certificate installed: $RUNTIME_CERT"

# Step 3: Also ensure root_ca.crt is present (required for chain verification)
log ""
log "Step 3: Ensuring root_ca.crt is present..."

ROOT_CA_CERT="$TRUST_WORK_DIR/certs/root_ca.crt"
RUNTIME_ROOT_CA="$RUNTIME_CERTS_DIR/root_ca.crt"

if [[ ! -f "$ROOT_CA_CERT" ]]; then
    error "Root CA certificate not found: $ROOT_CA_CERT"
    exit 1
fi

if [[ ! -f "$RUNTIME_ROOT_CA" ]]; then
    cp "$ROOT_CA_CERT" "$RUNTIME_ROOT_CA"
    chmod 644 "$RUNTIME_ROOT_CA"
    chown ransomeye:ransomeye "$RUNTIME_ROOT_CA" 2>/dev/null || chown root:root "$RUNTIME_ROOT_CA"
    success "Root CA certificate installed: $RUNTIME_ROOT_CA"
else
    success "Root CA certificate already present: $RUNTIME_ROOT_CA"
fi

# Step 4: Verify installation
log ""
log "Step 4: Verifying installation..."

VERIFY_FAILED=0

if [[ ! -f "$RUNTIME_CERT" ]] || [[ ! -s "$RUNTIME_CERT" ]]; then
    error "Runtime certificate verification failed: $RUNTIME_CERT"
    ((VERIFY_FAILED++)) || true
else
    CERT_SIZE=$(stat -c%s "$RUNTIME_CERT")
    if [[ $CERT_SIZE -lt 100 ]]; then
        error "Certificate file too small: $CERT_SIZE bytes (expected > 100 bytes)"
        ((VERIFY_FAILED++)) || true
    else
        success "Runtime certificate verified: $RUNTIME_CERT ($CERT_SIZE bytes)"
    fi
fi

if [[ ! -f "$RUNTIME_ROOT_CA" ]] || [[ ! -s "$RUNTIME_ROOT_CA" ]]; then
    error "Root CA certificate verification failed: $RUNTIME_ROOT_CA"
    ((VERIFY_FAILED++)) || true
else
    success "Root CA certificate verified: $RUNTIME_ROOT_CA"
fi

# Verify certificate is not a dummy/placeholder
if grep -qi "dummy\|placeholder\|example\|test" "$RUNTIME_CERT" 2>/dev/null; then
    error "Certificate appears to be a dummy/placeholder (contains dummy/placeholder/example/test)"
    ((VERIFY_FAILED++)) || true
else
    success "Certificate is not a dummy/placeholder"
fi

# Verify permissions
CERT_PERMS=$(stat -c "%a" "$RUNTIME_CERT" 2>/dev/null || echo "000")
if [[ "$CERT_PERMS" != "644" ]]; then
    warning "Certificate permissions are $CERT_PERMS (expected 644), fixing..."
    chmod 644 "$RUNTIME_CERT"
    success "Permissions fixed"
else
    success "Certificate permissions correct: $CERT_PERMS"
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
success "Artifacts signing certificate: $RUNTIME_CERT"
success "Root CA certificate: $RUNTIME_ROOT_CA"
log ""
log "Next step: Verify ransomeye-intelligence.service can start"
log "  sudo systemctl status ransomeye-intelligence.service"
log ""

exit 0

