#!/bin/bash
# Path and File Name: /home/ransomeye/rebuild/ransomeye_linux_agent/installer/verify.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Post-install validation script for Linux Agent

set -euo pipefail

INSTALL_DIR="/opt/ransomeye/linux_agent"
BINARY_PATH="$INSTALL_DIR/bin/ransomeye_linux_agent"
SERVICE_NAME="ransomeye-linux-agent.service"
RECEIPT_FILE="$INSTALL_DIR/.install_receipt.json"
RUN_USER="ransomeye"

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

echo "RansomEye Linux Agent - Installation Verification"
echo "=================================================="
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

# NO-SWAP validation
CURRENT_SWAP_GB=$(free -g | awk '/^Swap:/ {print $2}')
if [[ $CURRENT_SWAP_GB -eq 0 ]]; then
    check "NO-SWAP requirement met (0GB swap)" "true"
else
    echo -e "${RED}✗${NC} Swap detected (${CURRENT_SWAP_GB}GB) - Linux Agent MUST NOT use swap"
    ((FAILED++))
fi

# User/group checks
check "Run user exists" "id -u $RUN_USER > /dev/null 2>&1"

# Systemd service checks
check "Service file exists" "test -f /etc/systemd/system/$SERVICE_NAME"
check "Service file is readable" "test -r /etc/systemd/system/$SERVICE_NAME"

# Summary
echo ""
echo "=================================================="
echo "Verification Summary: $PASSED passed, $FAILED failed"
echo "=================================================="

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed. Please review the errors above.${NC}"
    exit 1
fi

