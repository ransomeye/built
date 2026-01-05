#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/tests/failure_injection_linux_agent.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYylg8CMw1iGsq7gU
# Details of functionality of this file: Failure injection tests for Linux Agent (Phase A3)

set -euo pipefail

RESULTS_DIR="/tmp/ransomeye_failure_injection_$(date +%s)"
mkdir -p "$RESULTS_DIR"

echo "=== Linux Agent Failure Injection Tests ==="
echo "Results directory: $RESULTS_DIR"
echo ""

# Test 1: Disk Full
echo "[TEST 1] Disk Full Failure Injection"
echo "-----------------------------------"
BUFFER_DIR="/var/lib/ransomeye-linux-agent"
if [ -d "$BUFFER_DIR" ]; then
    # Fill disk in buffer directory
    dd if=/dev/zero of="$BUFFER_DIR/fill_disk_test" bs=1M count=100 2>&1 | tee "$RESULTS_DIR/disk_full.log" || true
    sleep 2
    # Check agent logs
    journalctl -u ransomeye-linux-agent.service --since "1 minute ago" | grep -i "disk\|space\|enospc" > "$RESULTS_DIR/disk_full_agent.log" || true
    # Cleanup
    rm -f "$BUFFER_DIR/fill_disk_test" 2>/dev/null || true
    echo "Result: See $RESULTS_DIR/disk_full.log and $RESULTS_DIR/disk_full_agent.log"
else
    echo "WARNING: Buffer directory not found: $BUFFER_DIR"
fi
echo ""

# Test 2: Network Flap
echo "[TEST 2] Network Flap Failure Injection"
echo "-----------------------------------"
CORE_API_URL="${CORE_API_URL:-http://localhost:8080}"
CORE_IP=$(echo "$CORE_API_URL" | sed -E 's|https?://([^:/]+).*|\1|')
if [ -n "$CORE_IP" ] && [ "$CORE_IP" != "localhost" ]; then
    # Block Core API
    iptables -A OUTPUT -d "$CORE_IP" -j DROP 2>&1 | tee "$RESULTS_DIR/network_flap_block.log" || true
    echo "Network blocked to $CORE_IP"
    sleep 5
    # Check agent logs
    journalctl -u ransomeye-linux-agent.service --since "1 minute ago" | grep -i "network\|connection\|timeout" > "$RESULTS_DIR/network_flap_agent.log" || true
    # Restore network
    iptables -D OUTPUT -d "$CORE_IP" -j DROP 2>&1 | tee "$RESULTS_DIR/network_flap_restore.log" || true
    echo "Network restored"
    echo "Result: See $RESULTS_DIR/network_flap_*.log"
else
    echo "WARNING: Could not determine Core IP from CORE_API_URL: $CORE_API_URL"
fi
echo ""

# Test 3: Clock Skew
echo "[TEST 3] Clock Skew Failure Injection"
echo "-----------------------------------"
ORIGINAL_DATE=$(date)
date -s "+1 hour" 2>&1 | tee "$RESULTS_DIR/clock_skew_set.log" || true
echo "Clock set forward by 1 hour"
sleep 2
# Check agent logs
journalctl -u ransomeye-linux-agent.service --since "1 minute ago" > "$RESULTS_DIR/clock_skew_agent.log" || true
# Restore clock (requires NTP or manual restore)
echo "WARNING: Clock must be restored manually or via NTP"
echo "Result: See $RESULTS_DIR/clock_skew_*.log"
echo ""

# Test 4: Invalid Config
echo "[TEST 4] Invalid Config Failure Injection"
echo "-----------------------------------"
CONFIG_FILE="/etc/ransomeye/agent.env"
if [ -f "$CONFIG_FILE" ]; then
    # Backup config
    cp "$CONFIG_FILE" "$CONFIG_FILE.backup.$$"
    # Corrupt config
    echo "INVALID_CONFIG_LINE=broken" >> "$CONFIG_FILE"
    # Restart agent
    systemctl restart ransomeye-linux-agent.service 2>&1 | tee "$RESULTS_DIR/invalid_config_restart.log" || true
    sleep 3
    # Check status
    systemctl status ransomeye-linux-agent.service > "$RESULTS_DIR/invalid_config_status.log" 2>&1 || true
    # Restore config
    mv "$CONFIG_FILE.backup.$$" "$CONFIG_FILE"
    systemctl restart ransomeye-linux-agent.service 2>&1 || true
    echo "Result: See $RESULTS_DIR/invalid_config_*.log"
else
    echo "WARNING: Config file not found: $CONFIG_FILE"
fi
echo ""

# Test 5: Invalid Signature
echo "[TEST 5] Invalid Signature Failure Injection"
echo "-----------------------------------"
SIGNING_KEY="${AGENT_SIGNING_KEY_PATH:-/etc/ransomeye/keys/linux_agent_signing.key}"
if [ -f "$SIGNING_KEY" ]; then
    # Backup key
    cp "$SIGNING_KEY" "$SIGNING_KEY.backup.$$"
    # Corrupt key
    echo "INVALID_KEY_DATA" > "$SIGNING_KEY"
    # Restart agent
    systemctl restart ransomeye-linux-agent.service 2>&1 | tee "$RESULTS_DIR/invalid_signature_restart.log" || true
    sleep 3
    # Check status
    systemctl status ransomeye-linux-agent.service > "$RESULTS_DIR/invalid_signature_status.log" 2>&1 || true
    # Restore key
    mv "$SIGNING_KEY.backup.$$" "$SIGNING_KEY"
    systemctl restart ransomeye-linux-agent.service 2>&1 || true
    echo "Result: See $RESULTS_DIR/invalid_signature_*.log"
else
    echo "WARNING: Signing key not found: $SIGNING_KEY"
fi
echo ""

# Generate summary
cat > "$RESULTS_DIR/summary.txt" <<EOF
Failure Injection Test Summary
=============================
Test Date: $(date)
Results Directory: $RESULTS_DIR

Tests Executed:
1. Disk Full - See disk_full*.log
2. Network Flap - See network_flap_*.log
3. Clock Skew - See clock_skew_*.log
4. Invalid Config - See invalid_config_*.log
5. Invalid Signature - See invalid_signature_*.log

Expected Behavior:
- Agent should halt safely on critical failures (config, signature)
- Agent should continue with graceful degradation on non-critical failures (network, disk)
- All errors must be explicitly logged (no silent drops)
- No undefined behavior or crashes

Review individual log files for detailed results.
EOF

cat "$RESULTS_DIR/summary.txt"
echo ""
echo "All test results available in: $RESULTS_DIR"

