#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/installer/install.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Standalone Linux Agent installer - creates runtime layout, installs systemd unit, enables service
# CRITICAL: This is a STANDALONE module - NOT part of Core
# RUNTIME: Uses /opt/ransomeye-linux-agent (NOT /opt/ransomeye)
# USER: Creates and uses ransomeye-agent (NOT ransomeye)

set -euo pipefail

# Fail-closed: exit immediately on any error
set -o errexit
set -o nounset
set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="/opt/ransomeye-linux-agent"
SYSTEMD_DIR="/etc/systemd/system"
ENV_DIR="/etc/ransomeye-linux-agent"
LOG_FILE="/var/log/ransomeye-linux-agent/install.log"
RUN_USER="ransomeye-agent"
RUN_GROUP="ransomeye-agent"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE" 2>/dev/null || echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "$LOG_FILE" >&2
    exit 1
}

success() {
    echo -e "${GREEN}✓ $1${NC}" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}" | tee -a "$LOG_FILE"
}

# Create log directory
mkdir -p "$(dirname "$LOG_FILE")"

log "Starting RansomEye Linux Agent (STANDALONE) installation"

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    error "This installer must be run as root (use sudo)"
fi

# ============================================================================
# STEP 1: Create ransomeye-agent system user
# ============================================================================
log "Creating ransomeye-agent system user"

if id "$RUN_USER" &>/dev/null; then
    warning "User $RUN_USER already exists"
else
    useradd -r -s /usr/sbin/nologin -d "$INSTALL_DIR" -c "RansomEye Linux Agent" "$RUN_USER" || error "Failed to create user $RUN_USER"
    success "Created system user: $RUN_USER"
fi

# ============================================================================
# STEP 2: Create runtime directory structure
# ============================================================================
log "Creating runtime directory structure"

mkdir -p "$INSTALL_DIR"/{bin,config,lib,logs}
chown -R "$RUN_USER:$RUN_GROUP" "$INSTALL_DIR"
chmod 755 "$INSTALL_DIR"
chmod 750 "$INSTALL_DIR"/{bin,config,lib,logs}

success "Created runtime directory: $INSTALL_DIR"

# ============================================================================
# STEP 3: Install agent binary
# ============================================================================
log "Installing agent binary"

# Check if binary exists in module directory
BINARY_SOURCE="$MODULE_DIR/target/release/ransomeye_linux_agent"
if [[ ! -f "$BINARY_SOURCE" ]]; then
    # Try alternative locations
    BINARY_SOURCE="$MODULE_DIR/../target/release/ransomeye_linux_agent"
    if [[ ! -f "$BINARY_SOURCE" ]]; then
        error "Agent binary not found. Build the agent first: cd $MODULE_DIR && cargo build --release"
    fi
fi

cp "$BINARY_SOURCE" "$INSTALL_DIR/bin/ransomeye_linux_agent"
chown "$RUN_USER:$RUN_GROUP" "$INSTALL_DIR/bin/ransomeye_linux_agent"
chmod 550 "$INSTALL_DIR/bin/ransomeye_linux_agent"

success "Installed agent binary: $INSTALL_DIR/bin/ransomeye_linux_agent"

# ============================================================================
# STEP 4: Create environment file (canonical path: /etc/ransomeye/agent.env)
# ============================================================================
log "Creating environment configuration file"

# Use canonical path: /etc/ransomeye/agent.env
AGENT_ENV_DIR="/etc/ransomeye"
AGENT_ENV_FILE="$AGENT_ENV_DIR/agent.env"

mkdir -p "$AGENT_ENV_DIR"

# Only create/update if file doesn't exist or is missing required variables
if [[ ! -f "$AGENT_ENV_FILE" ]] || ! grep -q "^CORE_API_URL=" "$AGENT_ENV_FILE" 2>/dev/null; then
    cat > "$AGENT_ENV_FILE" << 'EOF'
# Path and File Name : /etc/ransomeye/agent.env
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Environment configuration for RansomEye Linux Agent

# Core API endpoint (configure as needed)
CORE_API_URL=http://localhost:8080

# Agent configuration
AGENT_ID=$(hostname)
AGENT_BUFFER_DIR=/var/lib/ransomeye-linux-agent/buffer

# Model directory (if using local models)
MODEL_DIR=/opt/ransomeye-linux-agent/models

# eBPF enablement
ENABLE_EBPF=true

# Certificate path (if using mTLS)
AGENT_CERT_PATH=/etc/ransomeye-linux-agent/certs/agent.pem
EOF

    chmod 640 "$AGENT_ENV_FILE"
    chown root:"$RUN_GROUP" "$AGENT_ENV_FILE"
    success "Created environment file: $AGENT_ENV_FILE"
else
    success "Environment file already exists: $AGENT_ENV_FILE"
fi

# ============================================================================
# STEP 5: Install systemd unit
# ============================================================================
log "Installing systemd unit"

SYSTEMD_SOURCE="$MODULE_DIR/systemd/ransomeye-linux-agent.service"
if [[ ! -f "$SYSTEMD_SOURCE" ]]; then
    error "Systemd unit not found: $SYSTEMD_SOURCE"
fi

cp "$SYSTEMD_SOURCE" "$SYSTEMD_DIR/ransomeye-linux-agent.service"
chmod 644 "$SYSTEMD_DIR/ransomeye-linux-agent.service"
chown root:root "$SYSTEMD_DIR/ransomeye-linux-agent.service"

# Reload systemd
systemctl daemon-reload

success "Installed systemd unit: $SYSTEMD_DIR/ransomeye-linux-agent.service"

# ============================================================================
# STEP 6: Enable service (but DO NOT start)
# ============================================================================
log "Enabling service (not starting)"

systemctl enable ransomeye-linux-agent.service || error "Failed to enable service"

success "Service enabled (not started as per requirement)"

# ============================================================================
# STEP 7: Provision signing key (Ed25519)
# ============================================================================
log "Provisioning Ed25519 signing key"

PROVISION_SCRIPT="$SCRIPT_DIR/provision_signing_key.sh"
if [[ -f "$PROVISION_SCRIPT" ]]; then
    # Run provisioning script (it's idempotent and fail-closed)
    bash "$PROVISION_SCRIPT" || error "Failed to provision signing key"
    success "Signing key provisioned"
else
    warning "Signing key provisioning script not found: $PROVISION_SCRIPT"
    warning "Agent will fail to start without AGENT_SIGNING_KEY_PATH set"
fi

# ============================================================================
# STEP 8: Create state directories
# ============================================================================
log "Creating state directories"

mkdir -p /var/lib/ransomeye-linux-agent/{buffer,state}
mkdir -p /run/ransomeye-linux-agent
chown -R "$RUN_USER:$RUN_GROUP" /var/lib/ransomeye-linux-agent
chown -R "$RUN_USER:$RUN_GROUP" /run/ransomeye-linux-agent

success "Created state directories"

# ============================================================================
# STEP 9: Verification
# ============================================================================
log "Verifying installation"

# Check binary exists
if [[ ! -f "$INSTALL_DIR/bin/ransomeye_linux_agent" ]]; then
    error "Binary verification failed"
fi

# Check systemd unit exists
if [[ ! -f "$SYSTEMD_DIR/ransomeye-linux-agent.service" ]]; then
    error "Systemd unit verification failed"
fi

# Check service is enabled
if ! systemctl is-enabled ransomeye-linux-agent.service &>/dev/null; then
    error "Service enablement verification failed"
fi

# Check service is NOT active
if systemctl is-active ransomeye-linux-agent.service &>/dev/null; then
    warning "Service is active (should not be started by installer)"
fi

success "Installation verification passed"

# ============================================================================
# SUMMARY
# ============================================================================
echo ""
echo "==========================================================================="
echo "RansomEye Linux Agent (STANDALONE) Installation Complete"
echo "==========================================================================="
echo ""
echo "Runtime directory: $INSTALL_DIR"
echo "Systemd unit: $SYSTEMD_DIR/ransomeye-linux-agent.service"
echo "Environment file: $AGENT_ENV_FILE"
echo "Service status: enabled (not started)"
echo ""
echo "To start the service:"
echo "  sudo systemctl start ransomeye-linux-agent.service"
echo ""
echo "To check status:"
echo "  sudo systemctl status ransomeye-linux-agent.service"
echo ""
echo "==========================================================================="

log "Installation completed successfully"
success "RansomEye Linux Agent (STANDALONE) installation complete"
