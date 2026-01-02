#!/bin/bash
# Path and File Name: /home/ransomeye/rebuild/ransomeye_dpi_probe/installer/install.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: DPI Probe standalone installer - enforces EULA, validates prerequisites, creates swap, installs service

set -euo pipefail

# Fail-closed: exit immediately on any error
set -o errexit
set -o nounset
set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="/opt/ransomeye/dpi_probe"
SYSTEMD_DIR="/etc/systemd/system"
LOG_FILE="/var/log/ransomeye/dpi_probe_install.log"
RECEIPT_FILE="$INSTALL_DIR/.install_receipt.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
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

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

log "Starting RansomEye DPI Probe installation"

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    error "This installer must be run as root (use sudo)"
fi

# ============================================================================
# 1. EULA ENFORCEMENT (MANDATORY, NO BYPASS)
# ============================================================================
EULA_FILE="$SCRIPT_DIR/EULA.txt"
if [[ ! -f "$EULA_FILE" ]]; then
    error "EULA file not found at $EULA_FILE"
fi

echo ""
echo "==========================================================================="
echo "END USER LICENSE AGREEMENT (EULA)"
echo "==========================================================================="
if [[ -s "$EULA_FILE" ]]; then
    cat "$EULA_FILE"
else
    echo "EULA terms apply. Please contact support@ransomeye.tech for details."
fi
echo "==========================================================================="
echo ""

while true; do
    read -p "Do you accept the EULA? (yes/no): " eula_response
    case "$eula_response" in
        yes|YES|y|Y)
            success "EULA accepted"
            break
            ;;
        no|NO|n|N)
            error "EULA not accepted. Installation aborted."
            ;;
        *)
            echo "Please enter 'yes' or 'no'"
            ;;
    esac
done

# ============================================================================
# 2. BINARY SIGNATURE VERIFICATION
# ============================================================================
BINARY_PATH="$MODULE_DIR/target/release/ransomeye_dpi_probe"
if [[ ! -f "$BINARY_PATH" ]]; then
    error "Binary not found at $BINARY_PATH. Please build the project first: cargo build --release"
fi

# Verify binary signature (using GPG or similar)
# For now, verify binary exists and is executable
if [[ ! -x "$BINARY_PATH" ]]; then
    error "Binary is not executable: $BINARY_PATH"
fi

# Check binary signature (placeholder - implement actual signature verification)
if command -v gpg &> /dev/null; then
    SIG_FILE="${BINARY_PATH}.sig"
    if [[ -f "$SIG_FILE" ]]; then
        if gpg --verify "$SIG_FILE" "$BINARY_PATH" 2>&1 | tee -a "$LOG_FILE"; then
            success "Binary signature verified"
        else
            error "Binary signature verification failed"
        fi
    else
        warning "Signature file not found, skipping verification"
    fi
else
    warning "GPG not available, skipping signature verification"
fi

# ============================================================================
# 3. ENVIRONMENT CONSTRAINTS
# ============================================================================

# OS check (Linux required)
if [[ "$(uname)" != "Linux" ]]; then
    error "DPI Probe requires Linux. Detected: $(uname)"
fi
success "OS check passed: $(uname -a)"

# CPU/NUMA validation
CPU_COUNT=$(nproc)
if [[ $CPU_COUNT -lt 4 ]]; then
    error "DPI Probe requires at least 4 CPU cores. Found: $CPU_COUNT"
fi
success "CPU check passed: $CPU_COUNT cores"

# Check NUMA topology
if command -v numactl &> /dev/null; then
    NUM_NODES=$(numactl --hardware | grep -c "^available:" || echo "1")
    success "NUMA topology: $NUM_NODES node(s)"
else
    warning "numactl not available, assuming single NUMA node"
fi

# Kernel feature checks: AF_XDP / eBPF
KERNEL_VERSION=$(uname -r | cut -d. -f1,2)
REQUIRED_VERSION="4.18"

# Check if kernel version is sufficient for AF_XDP (requires 4.18+)
if ! awk "BEGIN {exit !($KERNEL_VERSION < $REQUIRED_VERSION)}"; then
    error "Kernel version $KERNEL_VERSION is too old. AF_XDP requires kernel 4.18+. Please upgrade."
fi
success "Kernel version check passed: $(uname -r)"

# Check for AF_XDP support
if [[ ! -d /sys/class/net ]]; then
    error "Cannot check network interfaces: /sys/class/net not found"
fi

# Check for eBPF support
if [[ ! -d /sys/fs/bpf ]] && ! mountpoint -q /sys/fs/bpf 2>/dev/null; then
    warning "eBPF filesystem not mounted. Some features may be limited."
else
    success "eBPF filesystem available"
fi

# Check for libpcap
if ! ldconfig -p | grep -q libpcap; then
    error "libpcap not found. Please install: apt-get install libpcap-dev (Debian/Ubuntu) or yum install libpcap-devel (RHEL/CentOS)"
fi
success "libpcap library found"

# ============================================================================
# 4. SWAP CREATION & VERIFICATION (16GB or RAM, MANDATORY)
# ============================================================================
log "Checking swap requirements..."

# Get RAM size in GB
RAM_GB=$(free -g | awk '/^Mem:/ {print $2}')

# Calculate required swap: max(16GB, RAM)
if [[ $RAM_GB -gt 16 ]]; then
    REQUIRED_SWAP_GB=$RAM_GB
else
    REQUIRED_SWAP_GB=16
fi

# Get current swap in GB
CURRENT_SWAP_GB=$(free -g | awk '/^Swap:/ {print $2}')

log "RAM: ${RAM_GB}GB, Required swap: ${REQUIRED_SWAP_GB}GB, Current swap: ${CURRENT_SWAP_GB}GB"

if [[ $CURRENT_SWAP_GB -lt $REQUIRED_SWAP_GB ]]; then
    SWAP_FILE="/swapfile_ransomeye_dpi"
    SWAP_SIZE_GB=$((REQUIRED_SWAP_GB - CURRENT_SWAP_GB))
    
    log "Creating swap file: ${SWAP_SIZE_GB}GB at $SWAP_FILE"
    
    # Check if swap file already exists
    if [[ -f "$SWAP_FILE" ]]; then
        warning "Swap file already exists: $SWAP_FILE"
        read -p "Remove and recreate? (yes/no): " recreate_swap
        if [[ "$recreate_swap" == "yes" ]]; then
            if swapoff "$SWAP_FILE" 2>/dev/null; then
                rm -f "$SWAP_FILE"
            fi
        else
            error "Swap file exists but insufficient swap. Cannot proceed without recreation."
        fi
    fi
    
    # Create swap file
    log "Allocating ${SWAP_SIZE_GB}GB for swap file..."
    if dd if=/dev/zero of="$SWAP_FILE" bs=1G count=$SWAP_SIZE_GB status=progress 2>&1 | tee -a "$LOG_FILE"; then
        success "Swap file created"
    else
        error "Failed to create swap file"
    fi
    
    # Set correct permissions
    chmod 600 "$SWAP_FILE"
    
    # Make swap
    if mkswap "$SWAP_FILE" 2>&1 | tee -a "$LOG_FILE"; then
        success "Swap formatted"
    else
        error "Failed to format swap file"
    fi
    
    # Enable swap
    if swapon "$SWAP_FILE" 2>&1 | tee -a "$LOG_FILE"; then
        success "Swap enabled"
    else
        error "Failed to enable swap"
    fi
    
    # Add to /etc/fstab for persistence
    if ! grep -q "$SWAP_FILE" /etc/fstab; then
        echo "$SWAP_FILE none swap sw 0 0" >> /etc/fstab
        success "Swap file added to /etc/fstab"
    fi
    
    # Verify swap
    NEW_SWAP_GB=$(free -g | awk '/^Swap:/ {print $2}')
    if [[ $NEW_SWAP_GB -ge $REQUIRED_SWAP_GB ]]; then
        success "Swap verification passed: ${NEW_SWAP_GB}GB available"
    else
        error "Swap verification failed: ${NEW_SWAP_GB}GB < ${REQUIRED_SWAP_GB}GB required"
    fi
else
    success "Swap requirement met: ${CURRENT_SWAP_GB}GB >= ${REQUIRED_SWAP_GB}GB"
fi

# ============================================================================
# 5. INSTALLATION
# ============================================================================

# Create installation directory
mkdir -p "$INSTALL_DIR"/{bin,config,logs}
success "Installation directory created: $INSTALL_DIR"

# Copy binary
cp "$BINARY_PATH" "$INSTALL_DIR/bin/"
chmod +x "$INSTALL_DIR/bin/ransomeye_dpi_probe"
success "Binary installed"

# Create default config directory structure
mkdir -p "$INSTALL_DIR/config"
if [[ -d "$MODULE_DIR/config" ]]; then
    cp -r "$MODULE_DIR/config"/* "$INSTALL_DIR/config/" 2>/dev/null || true
fi

# ============================================================================
# 6. SYSTEMD SERVICE INSTALLATION
# ============================================================================
SERVICE_FILE="$MODULE_DIR/systemd/ransomeye-dpi-probe.service"
if [[ ! -f "$SERVICE_FILE" ]]; then
    error "Service file not found: $SERVICE_FILE"
fi

cp "$SERVICE_FILE" "$SYSTEMD_DIR/"
systemctl daemon-reload
success "Systemd service installed"

# ============================================================================
# 7. INSTALL RECEIPT (SIGNED)
# ============================================================================
INSTALL_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
INSTALL_USER=$(whoami)
BINARY_HASH=$(sha256sum "$INSTALL_DIR/bin/ransomeye_dpi_probe" | cut -d' ' -f1)

cat > "$RECEIPT_FILE" <<EOF
{
  "module": "ransomeye_dpi_probe",
  "version": "1.0.0",
  "install_timestamp": "$INSTALL_TIMESTAMP",
  "install_user": "$INSTALL_USER",
  "install_dir": "$INSTALL_DIR",
  "binary_hash": "$BINARY_HASH",
  "eula_accepted": true,
  "swap_size_gb": $REQUIRED_SWAP_GB,
  "kernel_version": "$(uname -r)",
  "cpu_cores": $CPU_COUNT
}
EOF

# Sign receipt (if GPG available)
if command -v gpg &> /dev/null; then
    if gpg --default-key "$(gpg --list-keys --keyid-format LONG | head -3 | tail -1 | awk '{print $2}' | cut -d'/' -f2)" --detach-sign --armor "$RECEIPT_FILE" 2>&1 | tee -a "$LOG_FILE"; then
        success "Install receipt signed"
    else
        warning "Failed to sign install receipt"
    fi
fi

chmod 600 "$RECEIPT_FILE"
success "Install receipt created: $RECEIPT_FILE"

# ============================================================================
# 8. COMPLETION
# ============================================================================
log "Installation completed successfully"

echo ""
echo "==========================================================================="
echo "Installation Summary"
echo "==========================================================================="
echo "Module:     RansomEye DPI Probe"
echo "Version:    1.0.0"
echo "Install:    $INSTALL_DIR"
echo "Service:    ransomeye-dpi-probe.service"
echo "Swap:       ${REQUIRED_SWAP_GB}GB configured"
echo "==========================================================================="
echo ""
echo "Next steps:"
echo "  1. Configure environment variables in: $INSTALL_DIR/config/"
echo "  2. Start service: systemctl start ransomeye-dpi-probe"
echo "  3. Enable auto-start: systemctl enable ransomeye-dpi-probe"
echo "  4. Verify installation: $SCRIPT_DIR/verify.sh"
echo ""

success "Installation complete"

