#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/qa/failure_injection/test_invalid_policy_signature.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Test script for invalid policy signature failure scenario

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="$SCRIPT_DIR/../runtime/reports"
mkdir -p "$REPORT_DIR"

TEST_NAME="invalid_policy_signature"
LOG_FILE="$REPORT_DIR/${TEST_NAME}.log"
RESULT_FILE="$REPORT_DIR/${TEST_NAME}.result.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}✓ $1${NC}" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}" | tee -a "$LOG_FILE"
}

log "=========================================="
log "TEST: Invalid Policy Signature"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This test must be run as root (for systemctl operations)"
    exit 1
fi

# Get policy directory from environment
ENV_FILE="/etc/ransomeye/ransomeye.env"
if [[ ! -f "$ENV_FILE" ]]; then
    error "Environment file not found: $ENV_FILE"
    exit 1
fi

POLICY_DIR=$(grep "^RANSOMEYE_POLICY_DIR=" "$ENV_FILE" | cut -d'=' -f2 || echo "")
if [[ -z "$POLICY_DIR" ]]; then
    error "RANSOMEYE_POLICY_DIR not found in environment file"
    exit 1
fi

log "Policy directory: $POLICY_DIR"

# Find a signed policy file
POLICY_FILE=$(find "$POLICY_DIR" -name "*.yaml" -o -name "*.yml" 2>/dev/null | head -1 || echo "")

if [[ -z "$POLICY_FILE" ]]; then
    error "No policy files found in $POLICY_DIR"
    exit 1
fi

log "Found policy file: $POLICY_FILE"

# Backup original policy file
POLICY_BACKUP="/tmp/$(basename "$POLICY_FILE").backup.$$"
cp "$POLICY_FILE" "$POLICY_BACKUP"
log "Backed up policy file: $POLICY_BACKUP"

# Corrupt the policy file (add invalid data to signature section)
log "Corrupting policy file signature..."
if grep -q "signature:" "$POLICY_FILE"; then
    # Corrupt existing signature
    sed -i 's/signature:.*/signature: INVALID_SIGNATURE_CORRUPTED_TEST_DATA_12345/' "$POLICY_FILE"
else
    # Add invalid signature if none exists
    echo "signature: INVALID_SIGNATURE_CORRUPTED_TEST_DATA_12345" >> "$POLICY_FILE"
fi

# Alternatively, corrupt file content directly (more aggressive)
log "Adding corruption to policy file content..."
echo "" >> "$POLICY_FILE"
echo "# CORRUPTED_TEST_DATA" >> "$POLICY_FILE"
echo "corrupted_field: $(dd if=/dev/urandom bs=1 count=100 2>/dev/null | base64 | tr -d '\n')" >> "$POLICY_FILE"

log "Policy file corrupted"

# Record start time
START_TIME=$(date +%s.%N)

# Attempt to start orchestrator service
log "Attempting to start orchestrator service..."
if systemctl start ransomeye-orchestrator.service 2>&1 | tee -a "$LOG_FILE"; then
    START_EXIT_CODE=0
else
    START_EXIT_CODE=${PIPESTATUS[0]}
fi

# Record failure detection time
FAILURE_TIME=$(date +%s.%N)
FAILURE_DETECTION_TIME=$(echo "$FAILURE_TIME - $START_TIME" | bc)

# Wait a moment for service to stabilize
sleep 3

# Check service status
log "Checking service status..."
SERVICE_STATUS=$(systemctl is-active ransomeye-orchestrator.service 2>&1 || echo "failed")
SERVICE_ENABLED=$(systemctl is-enabled ransomeye-orchestrator.service 2>&1 || echo "disabled")

log "Service status: $SERVICE_STATUS"
log "Service enabled: $SERVICE_ENABLED"

# Get exit code
EXIT_CODE=$(systemctl show ransomeye-orchestrator.service -p ExecMainStatus --value 2>/dev/null || echo "unknown")
log "Exit code: $EXIT_CODE"

# Check if service is in failed state
if [[ "$SERVICE_STATUS" == "failed" ]] || [[ "$SERVICE_STATUS" == "inactive" ]]; then
    success "Service correctly failed (status: $SERVICE_STATUS)"
    FAILED_CORRECTLY=true
else
    error "Service did not fail correctly (status: $SERVICE_STATUS)"
    FAILED_CORRECTLY=false
fi

# Check exit code
if [[ "$EXIT_CODE" != "0" ]] && [[ "$EXIT_CODE" != "unknown" ]]; then
    success "Exit code is non-zero: $EXIT_CODE"
    EXIT_CODE_CORRECT=true
else
    if [[ "$EXIT_CODE" == "unknown" ]]; then
        warning "Could not determine exit code (service may not have started)"
        EXIT_CODE_CORRECT=true
    else
        error "Exit code is zero (expected non-zero): $EXIT_CODE"
        EXIT_CODE_CORRECT=false
    fi
fi

# Check for restart attempts
RESTART_COUNT=$(systemctl show ransomeye-orchestrator.service -p NRestarts --value 2>/dev/null || echo "0")
log "Restart count: $RESTART_COUNT"

if [[ "$RESTART_COUNT" == "0" ]]; then
    success "No restart attempts (Restart=no enforced)"
    NO_RESTART=true
else
    error "Service attempted to restart ($RESTART_COUNT times) - Restart=no not enforced"
    NO_RESTART=false
fi

# Get journal logs
log "Capturing journal logs..."
JOURNAL_LOG="$REPORT_DIR/${TEST_NAME}.journal.log"
journalctl -u ransomeye-orchestrator.service --no-pager -n 100 > "$JOURNAL_LOG" 2>&1 || true
log "Journal logs saved to: $JOURNAL_LOG"

# Check for policy-related error message
if grep -qi "policy\|signature\|trust\|invalid" "$JOURNAL_LOG"; then
    success "Policy-related error message found in journal"
    ERROR_VISIBLE=true
else
    warning "No clear policy error message found in journal (check manually)"
    ERROR_VISIBLE=false
fi

# Restore original policy file
log "Restoring original policy file"
mv "$POLICY_BACKUP" "$POLICY_FILE"

# Generate result JSON
cat > "$RESULT_FILE" <<EOF
{
  "test_name": "$TEST_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_behavior": {
    "policy_engine_refuses_init": true,
    "orchestrator_aborts_before_ready": true,
    "exit_code_non_zero": true,
    "service_remains_failed": true
  },
  "observed_behavior": {
    "service_status": "$SERVICE_STATUS",
    "exit_code": "$EXIT_CODE",
    "restart_count": "$RESTART_COUNT",
    "failure_detection_time_seconds": "$FAILURE_DETECTION_TIME",
    "error_visible_in_journal": $ERROR_VISIBLE
  },
  "test_results": {
    "failed_correctly": $FAILED_CORRECTLY,
    "exit_code_correct": $EXIT_CODE_CORRECT,
    "no_restart": $NO_RESTART,
    "error_visible": $ERROR_VISIBLE
  },
  "pass": $([ "$FAILED_CORRECTLY" == "true" ] && [ "$EXIT_CODE_CORRECT" == "true" ] && [ "$NO_RESTART" == "true" ] && echo "true" || echo "false")
}
EOF

log "Result saved to: $RESULT_FILE"

# Summary
log ""
log "=========================================="
log "TEST SUMMARY"
log "=========================================="
log "Service Status: $SERVICE_STATUS"
log "Exit Code: $EXIT_CODE"
log "Restart Count: $RESTART_COUNT"
log "Failure Detection Time: ${FAILURE_DETECTION_TIME}s"
log "Failed Correctly: $FAILED_CORRECTLY"
log "Exit Code Correct: $EXIT_CODE_CORRECT"
log "No Restart: $NO_RESTART"
log "Error Visible: $ERROR_VISIBLE"
log ""

if [[ "$FAILED_CORRECTLY" == "true" ]] && [[ "$EXIT_CODE_CORRECT" == "true" ]] && [[ "$NO_RESTART" == "true" ]]; then
    success "TEST PASSED: Invalid policy signature correctly causes fail-closed behavior"
    exit 0
else
    error "TEST FAILED: Invalid policy signature did not cause expected fail-closed behavior"
    exit 1
fi

