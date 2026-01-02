#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/edge/agent/linux/installer/uninstall.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Standalone Linux Agent uninstaller - removes systemd unit, runtime directory, env file
# CRITICAL: This is a STANDALONE module - does NOT touch Core files

set -euo pipefail

# Fail-closed: exit immediately on any error
set -o errexit
set -o nounset
set -o pipefail

INSTALL_DIR="/opt/ransomeye-linux-agent"
SYSTEMD_DIR="/etc/systemd/system"
ENV_DIR="/etc/ransomeye-linux-agent"
SERVICE_NAME="ransomeye-linux-agent.service"
RUN_USER="ransomeye-agent"
RUN_GROUP="ransomeye-agent"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

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

log "Starting RansomEye Linux Agent (STANDALONE) uninstallation"

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    error "This uninstaller must be run as root (use sudo)"
fi

# ============================================================================
# STEP 1: Stop and disable service
# ============================================================================
log "Stopping and disabling service"

if systemctl is-active "$SERVICE_NAME" &>/dev/null; then
    systemctl stop "$SERVICE_NAME" || warning "Failed to stop service (may not be running)"
fi

if systemctl is-enabled "$SERVICE_NAME" &>/dev/null; then
    systemctl disable "$SERVICE_NAME" || error "Failed to disable service"
fi

success "Service stopped and disabled"

# ============================================================================
# STEP 2: Remove systemd unit
# ============================================================================
log "Removing systemd unit"

SYSTEMD_FILE="$SYSTEMD_DIR/$SERVICE_NAME"
if [[ -f "$SYSTEMD_FILE" ]]; then
    rm -f "$SYSTEMD_FILE"
    systemctl daemon-reload
    success "Removed systemd unit: $SYSTEMD_FILE"
else
    warning "Systemd unit not found: $SYSTEMD_FILE"
fi

# ============================================================================
# STEP 3: Remove runtime directory
# ============================================================================
log "Removing runtime directory"

if [[ -d "$INSTALL_DIR" ]]; then
    rm -rf "$INSTALL_DIR"
    success "Removed runtime directory: $INSTALL_DIR"
else
    warning "Runtime directory not found: $INSTALL_DIR"
fi

# ============================================================================
# STEP 4: Remove environment file
# ============================================================================
log "Removing environment file"

ENV_FILE="$ENV_DIR/linux-agent.env"
if [[ -f "$ENV_FILE" ]]; then
    rm -f "$ENV_FILE"
    success "Removed environment file: $ENV_FILE"
else
    warning "Environment file not found: $ENV_FILE"
fi

# Remove env directory if empty
if [[ -d "$ENV_DIR" ]] && [[ -z "$(ls -A "$ENV_DIR")" ]]; then
    rmdir "$ENV_DIR" || warning "Failed to remove empty env directory"
fi

# ============================================================================
# STEP 5: Remove state directories (optional - ask user)
# ============================================================================
log "Checking state directories"

STATE_DIR="/var/lib/ransomeye-linux-agent"
RUNTIME_STATE_DIR="/run/ransomeye-linux-agent"

if [[ -d "$STATE_DIR" ]]; then
    read -p "Remove state directory $STATE_DIR? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$STATE_DIR"
        success "Removed state directory: $STATE_DIR"
    else
        warning "State directory preserved: $STATE_DIR"
    fi
fi

if [[ -d "$RUNTIME_STATE_DIR" ]]; then
    rm -rf "$RUNTIME_STATE_DIR" || warning "Failed to remove runtime state directory"
fi

# ============================================================================
# STEP 6: Remove system user (optional - ask user)
# ============================================================================
log "Checking system user"

if id "$RUN_USER" &>/dev/null; then
    read -p "Remove system user $RUN_USER? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        userdel "$RUN_USER" || warning "Failed to remove user (may be in use)"
        success "Removed system user: $RUN_USER"
    else
        warning "System user preserved: $RUN_USER"
    fi
else
    warning "System user not found: $RUN_USER"
fi

# ============================================================================
# SUMMARY
# ============================================================================
echo ""
echo "==========================================================================="
echo "RansomEye Linux Agent (STANDALONE) Uninstallation Complete"
echo "==========================================================================="
echo ""
echo "Removed:"
echo "  - Systemd unit: $SYSTEMD_DIR/$SERVICE_NAME"
echo "  - Runtime directory: $INSTALL_DIR"
echo "  - Environment file: $ENV_FILE"
echo ""
echo "Preserved (if not removed):"
echo "  - State directory: $STATE_DIR (if exists)"
echo "  - System user: $RUN_USER (if exists)"
echo ""
echo "==========================================================================="

log "Uninstallation completed successfully"
success "RansomEye Linux Agent (STANDALONE) uninstallation complete"
