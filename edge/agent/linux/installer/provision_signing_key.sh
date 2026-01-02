#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/installer/provision_signing_key.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Provisions Ed25519 signing key for Linux Agent - fail-closed, idempotent

set -euo pipefail

# Fail-closed: exit immediately on any error
set -o errexit
set -o nounset
set -o pipefail

KEY_DIR="/etc/ransomeye/keys"
KEY_FILE="$KEY_DIR/linux_agent_signing.key"
AGENT_ENV_FILE="/etc/ransomeye/agent.env"
RUN_USER="ransomeye-agent"
RUN_GROUP="ransomeye-agent"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    exit 1
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root (use sudo)"
fi

# ============================================================================
# STEP 1: Create keys directory
# ============================================================================
if [[ ! -d "$KEY_DIR" ]]; then
    mkdir -p "$KEY_DIR" || error "Failed to create keys directory: $KEY_DIR"
    chown "$RUN_USER:$RUN_GROUP" "$KEY_DIR" || error "Failed to set ownership on keys directory"
    chmod 750 "$KEY_DIR" || error "Failed to set permissions on keys directory"
    success "Created keys directory: $KEY_DIR"
else
    success "Keys directory already exists: $KEY_DIR"
fi

# ============================================================================
# STEP 2: Generate Ed25519 signing key (32 bytes raw format)
# ============================================================================
if [[ -f "$KEY_FILE" ]]; then
    # Verify existing key is valid (32 bytes)
    KEY_SIZE=$(stat -f%z "$KEY_FILE" 2>/dev/null || stat -c%s "$KEY_FILE" 2>/dev/null)
    if [[ "$KEY_SIZE" -eq 32 ]]; then
        success "Signing key already exists and is valid: $KEY_FILE"
    else
        error "Existing key file has invalid size ($KEY_SIZE bytes, expected 32). Remove it manually to regenerate."
    fi
else
    # Generate 32 bytes of cryptographically secure random data (Ed25519 private key)
    if command -v openssl &> /dev/null; then
        openssl rand -out "$KEY_FILE" 32 || error "Failed to generate Ed25519 key using openssl"
    elif command -v dd &> /dev/null && [[ -r /dev/urandom ]]; then
        dd if=/dev/urandom of="$KEY_FILE" bs=32 count=1 2>/dev/null || error "Failed to generate Ed25519 key using dd"
    else
        error "No suitable tool found to generate cryptographic key (requires openssl or dd with /dev/urandom)"
    fi
    
    # Verify key was generated correctly
    KEY_SIZE=$(stat -f%z "$KEY_FILE" 2>/dev/null || stat -c%s "$KEY_FILE" 2>/dev/null)
    if [[ "$KEY_SIZE" -ne 32 ]]; then
        rm -f "$KEY_FILE"
        error "Generated key has invalid size ($KEY_SIZE bytes, expected 32)"
    fi
    
    # Set ownership and permissions (600 = rw-------)
    # Key must be readable by ransomeye-agent user (agent runs as this user)
    chown "$RUN_USER:$RUN_GROUP" "$KEY_FILE" || error "Failed to set ownership on key file"
    chmod 600 "$KEY_FILE" || error "Failed to set permissions on key file"
    
    success "Generated Ed25519 signing key: $KEY_FILE (32 bytes)"
fi

# ============================================================================
# STEP 3: Add AGENT_SIGNING_KEY_PATH to agent.env
# ============================================================================
if [[ ! -f "$AGENT_ENV_FILE" ]]; then
    # Create agent.env if it doesn't exist
    mkdir -p "$(dirname "$AGENT_ENV_FILE")" || error "Failed to create /etc/ransomeye directory"
    touch "$AGENT_ENV_FILE" || error "Failed to create agent.env file"
    # agent.env should be readable by ransomeye-agent user
    chown root:"$RUN_GROUP" "$AGENT_ENV_FILE" || error "Failed to set ownership on agent.env"
    chmod 640 "$AGENT_ENV_FILE" || error "Failed to set permissions on agent.env"
    success "Created agent.env file: $AGENT_ENV_FILE"
fi

# Check if AGENT_SIGNING_KEY_PATH is already set
if grep -q "^AGENT_SIGNING_KEY_PATH=" "$AGENT_ENV_FILE" 2>/dev/null; then
    # Update existing entry
    if grep -q "^AGENT_SIGNING_KEY_PATH=$KEY_FILE" "$AGENT_ENV_FILE" 2>/dev/null; then
        success "AGENT_SIGNING_KEY_PATH already set correctly in agent.env"
    else
        # Update to correct path
        sed -i "s|^AGENT_SIGNING_KEY_PATH=.*|AGENT_SIGNING_KEY_PATH=$KEY_FILE|" "$AGENT_ENV_FILE" || \
            error "Failed to update AGENT_SIGNING_KEY_PATH in agent.env"
        success "Updated AGENT_SIGNING_KEY_PATH in agent.env"
    fi
else
    # Add new entry
    echo "AGENT_SIGNING_KEY_PATH=$KEY_FILE" >> "$AGENT_ENV_FILE" || \
        error "Failed to add AGENT_SIGNING_KEY_PATH to agent.env"
    success "Added AGENT_SIGNING_KEY_PATH to agent.env"
fi

# ============================================================================
# VERIFICATION
# ============================================================================
echo ""
echo "==========================================================================="
echo "Signing Key Provisioning Complete"
echo "==========================================================================="
echo ""
echo "Key file: $KEY_FILE"
echo "Key size: $(stat -f%z "$KEY_FILE" 2>/dev/null || stat -c%s "$KEY_FILE" 2>/dev/null) bytes"
echo "Permissions: $(stat -c%a "$KEY_FILE" 2>/dev/null || stat -f%OLp "$KEY_FILE" 2>/dev/null)"
echo "Owner: $(stat -c%U "$KEY_FILE" 2>/dev/null || stat -f%Su "$KEY_FILE" 2>/dev/null):$(stat -c%G "$KEY_FILE" 2>/dev/null || stat -f%Sg "$KEY_FILE" 2>/dev/null)"
echo ""
echo "Environment file: $AGENT_ENV_FILE"
echo "AGENT_SIGNING_KEY_PATH=$(grep "^AGENT_SIGNING_KEY_PATH=" "$AGENT_ENV_FILE" | cut -d'=' -f2)"
echo ""
echo "==========================================================================="

success "Signing key provisioning completed successfully"

