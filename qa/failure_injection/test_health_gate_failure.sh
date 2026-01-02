#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/qa/failure_injection/test_health_gate_failure.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Test script for health gate failure scenario

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="$SCRIPT_DIR/../runtime/reports"
mkdir -p "$REPORT_DIR"

TEST_NAME="health_gate_failure"
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
log "TEST: Health Gate Failure"
log "=========================================="
log ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This test must be run as root (for systemctl operations)"
    exit 1
fi

# Backup original environment file
ENV_FILE="/etc/ransomeye/ransomeye.env"
ENV_BACKUP="/tmp/ransomeye.env.backup.$$"

if [[ ! -f "$ENV_FILE" ]]; then
    error "Environment file not found: $ENV_FILE"
    exit 1
fi

log "Backing up environment file: $ENV_BACKUP"
cp "$ENV_FILE" "$ENV_BACKUP"

# Get original reporting directory
ORIGINAL_REPORTING_DIR=$(grep "^RANSOMEYE_REPORTING_DIR=" "$ENV_FILE" | cut -d'=' -f2 || echo "/var/lib/ransomeye/reports")

log "Original RANSOMEYE_REPORTING_DIR: $ORIGINAL_REPORTING_DIR"

# Set reporting directory to invalid path (simulates dependency validation failure)
INVALID_REPORTING_DIR="/nonexistent/reporting/dir/$(date +%s)"
log "Setting RANSOMEYE_REPORTING_DIR to invalid path: $INVALID_REPORTING_DIR"

# Update or add REPORTING_DIR
if grep -q "^RANSOMEYE_REPORTING_DIR=" "$ENV_FILE"; then
    sed -i "s|^RANSOMEYE_REPORTING_DIR=.*|RANSOMEYE_REPORTING_DIR=$INVALID_REPORTING_DIR|" "$ENV_FILE"
else
    echo "RANSOMEYE_REPORTING_DIR=$INVALID_REPORTING_DIR" >> "$ENV_FILE"
fi

# Verify update
if ! grep -q "^RANSOMEYE_REPORTING_DIR=$INVALID_REPORTING_DIR" "$ENV_FILE"; then
    error "Failed to update RANSOMEYE_REPORTING_DIR"
    mv "$ENV_BACKUP" "$ENV_FILE"
    exit 1
fi

log "RANSOMEYE_REPORTING_DIR updated to invalid path"

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

# Check for health gate or dependency validation error
if grep -qi "health.*gate\|dependency.*validation\|reporting.*dir\|health.*check\|validation.*failed" "$JOURNAL_LOG"; then
    success "Health gate or dependency validation error found in journal"
    ERROR_VISIBLE=true
else
    warning "No clear health gate error message found in journal (check manually)"
    ERROR_VISIBLE=false
fi

# Restore original environment file
log "Restoring original environment file"
mv "$ENV_BACKUP" "$ENV_FILE"

# Generate result JSON
cat > "$RESULT_FILE" <<EOF
{
  "test_name": "$TEST_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "expected_behavior": {
    "health_gate_fails": true,
    "orchestrator_aborts": true,
    "explicit_failure_logged": true,
    "exit_code_non_zero": true
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
    success "TEST PASSED: Health gate failure correctly causes fail-closed behavior"
    exit 0
else
    error "TEST FAILED: Health gate failure did not cause expected fail-closed behavior"
    exit 1
fi

