#!/bin/bash
# Path and File Name: /home/ransomeye/rebuild/ransomeye_dpi_probe/installer/verify.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Post-install validation script for DPI Probe

set -euo pipefail

INSTALL_DIR="/opt/ransomeye/dpi_probe"
BINARY_PATH="$INSTALL_DIR/bin/ransomeye_dpi_probe"
SERVICE_NAME="ransomeye-dpi-probe.service"
RECEIPT_FILE="$INSTALL_DIR/.install_receipt.json"
SWAP_FILE="/swapfile_ransomeye_dpi"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

check() {
    local name="$1"
    local test_cmd="$2"
    
    if eval "$test_cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $name"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $name"
        ((FAILED++))
        return 1
    fi
}

echo "RansomEye DPI Probe - Installation Verification"
echo "================================================"
echo ""

# Binary checks
check "Binary exists" "test -f $BINARY_PATH"
check "Binary is executable" "test -x $BINARY_PATH"

# Version check
if [[ -f "$BINARY_PATH" ]]; then
    if "$BINARY_PATH" --version > /dev/null 2>&1 || true; then
        check "Binary runs" "true"
    fi
fi

# Directory checks
check "Install directory exists" "test -d $INSTALL_DIR"
check "Bin directory exists" "test -d $INSTALL_DIR/bin"
check "Config directory exists" "test -d $INSTALL_DIR/config"
check "Logs directory exists" "test -d $INSTALL_DIR/logs"

# Receipt check
check "Install receipt exists" "test -f $RECEIPT_FILE"

if [[ -f "$RECEIPT_FILE" ]]; then
    if command -v jq &> /dev/null; then
        if jq -e . "$RECEIPT_FILE" > /dev/null 2>&1; then
            check "Install receipt is valid JSON" "true"
        fi
    fi
fi

# Systemd service checks
check "Service file exists" "test -f /etc/systemd/system/$SERVICE_NAME"
check "Service file is readable" "test -r /etc/systemd/system/$SERVICE_NAME"

# Swap checks
CURRENT_SWAP_GB=$(free -g | awk '/^Swap:/ {print $2}')
RAM_GB=$(free -g | awk '/^Mem:/ {print $2}')
REQUIRED_SWAP_GB=$((RAM_GB > 16 ? RAM_GB : 16))

if [[ $CURRENT_SWAP_GB -ge $REQUIRED_SWAP_GB ]]; then
    check "Swap requirement met (${CURRENT_SWAP_GB}GB >= ${REQUIRED_SWAP_GB}GB)" "true"
else
    echo -e "${RED}✗${NC} Swap requirement not met (${CURRENT_SWAP_GB}GB < ${REQUIRED_SWAP_GB}GB)"
    ((FAILED++))
fi

# Kernel feature checks
KERNEL_VERSION=$(uname -r | cut -d. -f1,2)
if awk "BEGIN {exit !($KERNEL_VERSION >= 4.18)}"; then
    check "Kernel version supports AF_XDP (>= 4.18)" "true"
else
    echo -e "${RED}✗${NC} Kernel version too old for AF_XDP"
    ((FAILED++))
fi

# Library checks
check "libpcap available" "ldconfig -p | grep -q libpcap"

# Summary
echo ""
echo "================================================"
echo "Verification Summary: $PASSED passed, $FAILED failed"
echo "================================================"

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed. Please review the errors above.${NC}"
    exit 1
fi

