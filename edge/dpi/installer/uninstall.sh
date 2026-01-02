#!/bin/bash
# Path and File Name: /home/ransomeye/rebuild/ransomeye_dpi_probe/installer/uninstall.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: DPI Probe standalone uninstaller - clean removal with configurable log preservation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="/opt/ransomeye/dpi_probe"
SYSTEMD_DIR="/etc/systemd/system"
SERVICE_NAME="ransomeye-dpi-probe.service"
LOG_FILE="/var/log/ransomeye/dpi_probe_uninstall.log"
SWAP_FILE="/swapfile_ransomeye_dpi"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
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

mkdir -p "$(dirname "$LOG_FILE")"

log "Starting RansomEye DPI Probe uninstallation"

# Check root
if [[ $EUID -ne 0 ]]; then
    error "This uninstaller must be run as root (use sudo)"
fi

# Confirmation
echo ""
echo "This will uninstall RansomEye DPI Probe."
echo "WARNING: This will stop the service and remove all installed files."
echo ""

# Ask about log preservation
PRESERVE_LOGS="yes"
read -p "Preserve logs and configuration? (yes/no) [yes]: " preserve_response
if [[ -n "$preserve_response" ]]; then
    PRESERVE_LOGS="$preserve_response"
fi

# Final confirmation
read -p "Are you sure you want to proceed? (yes/no): " confirm
if [[ "$confirm" != "yes" ]]; then
    log "Uninstallation cancelled by user"
    exit 0
fi

# ============================================================================
# 1. STOP SERVICE
# ============================================================================
log "Stopping service..."

if systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    systemctl stop "$SERVICE_NAME"
    success "Service stopped"
else
    warning "Service was not running"
fi

if systemctl is-enabled --quiet "$SERVICE_NAME" 2>/dev/null; then
    systemctl disable "$SERVICE_NAME"
    success "Service disabled"
fi

# ============================================================================
# 2. REMOVE SYSTEMD SERVICE
# ============================================================================
log "Removing systemd service..."

if [[ -f "$SYSTEMD_DIR/$SERVICE_NAME" ]]; then
    rm -f "$SYSTEMD_DIR/$SERVICE_NAME"
    systemctl daemon-reload
    success "Systemd service removed"
else
    warning "Service file not found: $SYSTEMD_DIR/$SERVICE_NAME"
fi

# ============================================================================
# 3. REMOVE BINARIES
# ============================================================================
log "Removing binaries..."

if [[ -d "$INSTALL_DIR" ]]; then
    if [[ "$PRESERVE_LOGS" == "yes" ]]; then
        # Preserve logs and config, remove only binaries
        rm -rf "$INSTALL_DIR/bin"
        success "Binaries removed (logs and config preserved)"
    else
        # Remove everything
        rm -rf "$INSTALL_DIR"
        success "Installation directory removed"
    fi
else
    warning "Installation directory not found: $INSTALL_DIR"
fi

# ============================================================================
# 4. SWAP FILE REMOVAL (OPTIONAL)
# ============================================================================
log "Checking swap file..."

if [[ -f "$SWAP_FILE" ]]; then
    read -p "Remove swap file $SWAP_FILE? (yes/no) [no]: " remove_swap
    if [[ "$remove_swap" == "yes" ]]; then
        if swapoff "$SWAP_FILE" 2>/dev/null; then
            rm -f "$SWAP_FILE"
            # Remove from /etc/fstab
            sed -i "\|$SWAP_FILE|d" /etc/fstab 2>/dev/null || true
            success "Swap file removed"
        else
            warning "Failed to disable swap file (may be in use)"
        fi
    else
        warning "Swap file preserved: $SWAP_FILE"
    fi
fi

# ============================================================================
# 5. CLEANUP
# ============================================================================
log "Cleaning up..."

# Remove empty parent directories if they exist
if [[ -d "$(dirname "$INSTALL_DIR")" ]] && [[ -z "$(ls -A "$(dirname "$INSTALL_DIR")" 2>/dev/null)" ]]; then
    rmdir "$(dirname "$INSTALL_DIR")" 2>/dev/null || true
fi

success "Cleanup complete"

# ============================================================================
# 6. COMPLETION
# ============================================================================
log "Uninstallation completed"

echo ""
echo "==========================================================================="
echo "Uninstallation Summary"
echo "==========================================================================="
echo "Service:    Removed and disabled"
echo "Binaries:   Removed"
if [[ "$PRESERVE_LOGS" == "yes" ]]; then
    echo "Logs:       Preserved at $INSTALL_DIR"
fi
echo "==========================================================================="
echo ""

success "Uninstallation complete"

