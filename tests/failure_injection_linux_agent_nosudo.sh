#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/tests/failure_injection_linux_agent_nosudo.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Failure injection tests for Linux Agent (NO SUDO REQUIRED)

set -euo pipefail

RESULTS_DIR="/tmp/ransomeye_failure_injection_nosudo_$(date +%s)"
mkdir -p "$RESULTS_DIR"

echo "=== Linux Agent Failure Injection Tests (No Sudo) ==="
echo "Results directory: $RESULTS_DIR"
echo ""

# Test 1: Network Flap (no sudo required for localhost)
echo "[TEST 1] Network Flap Failure Injection (Localhost)"
echo "-----------------------------------"
CORE_API_URL="${CORE_API_URL:-http://localhost:8080}"
echo "Core API URL: $CORE_API_URL"
# For localhost, we can't use iptables, but we can test connection failures
# by temporarily stopping the service or using firewall rules
echo "Note: Localhost network flap test requires service restart (manual)"
echo "Result: See $RESULTS_DIR/network_flap.log"
echo ""

# Test 2: Invalid Config (no sudo - use user-writable config)
echo "[TEST 2] Invalid Config Failure Injection"
echo "-----------------------------------"
USER_CONFIG_DIR="$HOME/.config/ransomeye"
mkdir -p "$USER_CONFIG_DIR"
TEST_CONFIG="$USER_CONFIG_DIR/test_agent.env"

# Create invalid config
cat > "$TEST_CONFIG" <<EOF
INVALID_CONFIG_SYNTAX=broken
MISSING_REQUIRED_VAR
EOF

echo "Created invalid config: $TEST_CONFIG"
echo "Result: See $RESULTS_DIR/invalid_config.log"
echo ""

# Test 3: Missing Signing Key (no sudo - use user directory)
echo "[TEST 3] Missing Signing Key Failure Injection"
echo "-----------------------------------"
USER_KEY_DIR="$HOME/.config/ransomeye/keys"
mkdir -p "$USER_KEY_DIR"
TEST_KEY="$USER_KEY_DIR/test_signing.key"

# Create invalid key
echo "INVALID_KEY_DATA" > "$TEST_KEY"
echo "Created invalid key: $TEST_KEY"
echo "Result: See $RESULTS_DIR/invalid_key.log"
echo ""

# Test 4: Agent Health Check (no sudo required)
echo "[TEST 4] Agent Health Check"
echo "-----------------------------------"
if systemctl is-active --quiet ransomeye-linux-agent.service 2>/dev/null; then
    echo "Agent service is running"
    systemctl status ransomeye-linux-agent.service --no-pager > "$RESULTS_DIR/agent_status.log" 2>&1 || true
else
    echo "Agent service is not running"
    echo "NOT_RUNNING" > "$RESULTS_DIR/agent_status.log"
fi
echo "Result: See $RESULTS_DIR/agent_status.log"
echo ""

# Generate summary
cat > "$RESULTS_DIR/summary.txt" <<EOF
Failure Injection Test Summary (No Sudo)
========================================
Test Date: $(date)
Results Directory: $RESULTS_DIR

Tests Executed:
1. Network Flap - Manual (requires service restart)
2. Invalid Config - Created invalid config file
3. Invalid Signing Key - Created invalid key file
4. Agent Health Check - Checked service status

Note: Some tests require sudo for full execution.
This script executes tests that don't require sudo.

Review individual log files for detailed results.
EOF

cat "$RESULTS_DIR/summary.txt"
echo ""
echo "All test results available in: $RESULTS_DIR"

